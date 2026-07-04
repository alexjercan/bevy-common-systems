//! A Bevy plugin for a full-screen "screen flash" damage / hit overlay.
//!
//! ## Overview
//!
//! `ScreenFlashPlugin` drives a full-screen UI overlay whose alpha spikes to a
//! peak and decays back to transparent -- the red flash a game slams across the
//! whole screen when the player takes a hit or the run ends.
//!
//! The overlay is a plain full-screen `Node` with a `BackgroundColor`; the
//! [`ScreenFlash`] component animates its alpha. The overlay's *color* lives in
//! the `BackgroundColor` (the plugin only touches the alpha channel, preserving
//! the RGB), so the caller picks the tint and the plugin fades it.
//!
//! Two usage shapes fall out of one primitive, both covered by the same linear
//! intensity decay:
//!
//! - **Spawn-and-fade** (a one-shot flash on death): spawn a [`screen_flash`]
//!   overlay; it starts at full intensity, fades over `1 / decay` seconds and
//!   despawns itself (`despawn_on_end: true`).
//! - **Spike-and-decay** (a reusable hit overlay): spawn a persistent overlay
//!   once (transparent, `despawn_on_end: false`) and re-insert [`ScreenFlash`]
//!   on each hit to re-spike it to peak. It never despawns, so it is always
//!   ready for the next hit.
//!
//! Re-inserting [`ScreenFlash`] on an already-flashing entity restarts the flash
//! from full intensity (it uses `On<Insert>`), which is exactly how the
//! spike-and-decay shape re-triggers.
//!
//! The component split follows the crate convention:
//!
//! 1. [`ScreenFlash`] - config: peak alpha, decay rate, whether to despawn.
//! 2. `ScreenFlashState` - private: the current intensity (1 at peak, 0 faded).
//!
//! ## Usage
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! # fn on_death(mut commands: Commands) {
//! // A red flash that fills the screen, fades over ~0.35s, then despawns.
//! commands.spawn(screen_flash(Color::srgb(0.9, 0.1, 0.1), 0.5, 1.0 / 0.35));
//! # }
//! ```
//!
//! The fade is a linear ramp for now; once the `tween` module exists it should
//! be backed by that instead.

use bevy::prelude::*;

pub mod prelude {
    pub use super::{
        screen_flash, screen_flash_node, ScreenFlash, ScreenFlashPlugin, ScreenFlashSystems,
    };
}

/// A full-screen flash overlay: fades an entity's `BackgroundColor` alpha from a
/// peak down to zero.
///
/// Put it on a full-screen UI `Node` that carries a `BackgroundColor` (the
/// [`screen_flash`] builder does both for you). [`ScreenFlashPlugin`] decays the
/// intensity each frame and writes `peak_alpha * intensity` into the overlay's
/// alpha, leaving the RGB untouched. Inserting (or re-inserting) it spikes the
/// intensity back to full.
#[derive(Component, Debug, Clone, Reflect)]
pub struct ScreenFlash {
    /// The overlay alpha at full intensity (intensity 1). The alpha ramps from
    /// here to zero as the flash decays.
    pub peak_alpha: f32,

    /// Intensity lost per second (linear). The flash goes from full to
    /// transparent in `1 / decay` seconds.
    pub decay: f32,

    /// Despawn the overlay entity once the intensity reaches zero. Leave it
    /// `true` for a one-shot spawn-and-fade flash; set it `false` for a
    /// persistent overlay that is re-spiked on later hits.
    pub despawn_on_end: bool,
}

impl Default for ScreenFlash {
    fn default() -> Self {
        Self {
            peak_alpha: 0.5,
            decay: 3.0,
            despawn_on_end: true,
        }
    }
}

/// Private per-flash state: the current intensity, 1 at the peak and 0 when
/// fully faded. Managed by the plugin.
#[derive(Component, Default, Debug, Reflect)]
struct ScreenFlashState {
    /// Current intensity in `[0, 1]`; the overlay alpha is `peak_alpha * this`.
    intensity: f32,
}

/// System set for [`ScreenFlashPlugin`].
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScreenFlashSystems {
    /// Decays each flash, drives the overlay alpha, and despawns finished
    /// one-shot flashes.
    Animate,
}

/// Plugin that animates full-screen [`ScreenFlash`] overlays.
pub struct ScreenFlashPlugin;

impl Plugin for ScreenFlashPlugin {
    fn build(&self, app: &mut App) {
        debug!("ScreenFlashPlugin: build");

        app.register_type::<ScreenFlash>()
            .register_type::<ScreenFlashState>();

        app.add_observer(on_insert_screen_flash);

        app.add_systems(
            Update,
            animate_screen_flash.in_set(ScreenFlashSystems::Animate),
        );
    }
}

/// The overlay alpha at a given intensity: `peak_alpha * intensity`, with
/// intensity clamped to `[0, 1]`.
fn flash_alpha(intensity: f32, peak_alpha: f32) -> f32 {
    peak_alpha * intensity.clamp(0.0, 1.0)
}

/// On [`ScreenFlash`] insert, spike the intensity to full. Uses `On<Insert>`
/// (not `On<Add>`) so re-inserting on an already-flashing overlay re-triggers
/// the flash instead of being ignored -- the spike-and-decay usage shape.
fn on_insert_screen_flash(
    insert: On<Insert, ScreenFlash>,
    mut commands: Commands,
    mut q_state: Query<&mut ScreenFlashState>,
) {
    let entity = insert.entity;
    trace!("on_insert_screen_flash: entity {:?}", entity);

    if let Ok(mut state) = q_state.get_mut(entity) {
        state.intensity = 1.0;
    } else {
        commands
            .entity(entity)
            .insert(ScreenFlashState { intensity: 1.0 });
    }
}

/// Decay each flash's intensity, drive the overlay's `BackgroundColor` alpha
/// from it (preserving the RGB tint), and despawn a finished one-shot flash.
///
/// `BackgroundColor` is optional so the flash still decays (and a one-shot still
/// despawns) even if the overlay is missing its background.
fn animate_screen_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut q_flash: Query<(
        Entity,
        &ScreenFlash,
        &mut ScreenFlashState,
        Option<&mut BackgroundColor>,
    )>,
) {
    let dt = time.delta_secs();

    for (entity, flash, mut state, background) in q_flash.iter_mut() {
        trace!(
            "animate_screen_flash: entity {:?} intensity {}",
            entity,
            state.intensity
        );

        state.intensity = (state.intensity - flash.decay * dt).max(0.0);

        if let Some(mut background) = background {
            let alpha = flash_alpha(state.intensity, flash.peak_alpha);
            background.0 = background.0.with_alpha(alpha);
        }

        if state.intensity <= 0.0 && flash.despawn_on_end {
            commands.entity(entity).despawn();
        }
    }
}

/// A full-screen absolute UI node covering the whole viewport, the shape every
/// screen-flash overlay uses. Spawn it with a `BackgroundColor` (and a
/// [`ScreenFlash`]) to make an overlay; the [`screen_flash`] builder does this
/// for the common one-shot case, while a persistent overlay can pair this with
/// its own `BackgroundColor` / marker / `GlobalZIndex`.
pub fn screen_flash_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    }
}

/// Bundle builder for a one-shot spawn-and-fade screen flash: a full-screen
/// overlay tinted `color`, starting at `peak_alpha`, fading to transparent over
/// `1 / decay` seconds and despawning itself.
///
/// Chain scoping onto the spawn, e.g.
/// `commands.spawn(screen_flash(red, 0.5, 3.0)).insert(DespawnOnExit(state))`.
pub fn screen_flash(color: Color, peak_alpha: f32, decay: f32) -> impl Bundle {
    (
        Name::new("Screen Flash"),
        ScreenFlash {
            peak_alpha,
            decay,
            despawn_on_end: true,
        },
        screen_flash_node(),
        BackgroundColor(color.with_alpha(peak_alpha)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flash_alpha_scales_and_clamps() {
        // Full intensity -> full peak alpha.
        assert!((flash_alpha(1.0, 0.5) - 0.5).abs() < 1e-6);
        // Half intensity -> half peak alpha.
        assert!((flash_alpha(0.5, 0.5) - 0.25).abs() < 1e-6);
        // Zero intensity -> transparent.
        assert_eq!(flash_alpha(0.0, 0.5), 0.0);
        // Intensity past 1 clamps to the peak (never brighter than the peak).
        assert!((flash_alpha(2.0, 0.5) - 0.5).abs() < 1e-6);
    }

    fn flash_app() -> App {
        let mut app = App::new();
        app.init_resource::<Time>();
        app.add_plugins(ScreenFlashPlugin);
        app
    }

    fn step(app: &mut App, dt_ms: u64) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(dt_ms));
        app.update();
    }

    #[test]
    fn insert_spikes_intensity_to_full() {
        let mut app = flash_app();
        let red = Color::srgb(0.9, 0.1, 0.1);
        let ent = app
            .world_mut()
            .spawn((
                ScreenFlash {
                    peak_alpha: 0.5,
                    // Zero decay so intensity holds at the spike for this check.
                    decay: 0.0,
                    despawn_on_end: false,
                },
                BackgroundColor(red.with_alpha(0.0)),
            ))
            .id();
        // Flush the insert observer + one animate step.
        step(&mut app, 16);

        // Intensity spiked to full, so alpha is the full peak...
        let bg = app.world().get::<BackgroundColor>(ent).unwrap().0;
        assert!(
            (bg.alpha() - 0.5).abs() < 1e-6,
            "insert should spike to peak alpha, got {}",
            bg.alpha()
        );
        // ...and the RGB tint is preserved (only alpha is animated).
        let lin = bg.to_linear();
        let red_lin = red.to_linear();
        assert!((lin.red - red_lin.red).abs() < 1e-6);
        assert!((lin.green - red_lin.green).abs() < 1e-6);
        assert!((lin.blue - red_lin.blue).abs() < 1e-6);
    }

    #[test]
    fn spawn_and_fade_decays_and_despawns() {
        let mut app = flash_app();
        let ent = app
            .world_mut()
            .spawn(screen_flash(Color::srgb(0.9, 0.1, 0.1), 0.5, 2.0))
            .id();

        // Halfway through the 0.5s life: intensity ~0.5, alpha ~0.25.
        step(&mut app, 250);
        let bg = app.world().get::<BackgroundColor>(ent).unwrap().0;
        assert!(
            (bg.alpha() - 0.25).abs() < 0.05,
            "mid-fade alpha should be ~0.25, got {}",
            bg.alpha()
        );
        assert!(
            app.world().get_entity(ent).is_ok(),
            "overlay should still exist mid-fade"
        );

        // Past the full life: intensity hits zero and despawn_on_end despawns it.
        step(&mut app, 400);
        assert!(
            app.world().get_entity(ent).is_err(),
            "a faded one-shot flash should despawn itself"
        );
    }

    #[test]
    fn persistent_overlay_survives_and_respikes() {
        let mut app = flash_app();
        let red = Color::srgb(0.85, 0.05, 0.05);
        let ent = app
            .world_mut()
            .spawn((
                ScreenFlash {
                    peak_alpha: 0.4,
                    decay: 3.0,
                    despawn_on_end: false,
                },
                BackgroundColor(red.with_alpha(0.0)),
            ))
            .id();

        // Let it decay fully to transparent; it must NOT despawn.
        for _ in 0..10 {
            step(&mut app, 100);
        }
        assert!(
            app.world().get_entity(ent).is_ok(),
            "a persistent overlay must not despawn when faded"
        );
        let faded = app.world().get::<BackgroundColor>(ent).unwrap().0.alpha();
        assert!(faded < 0.02, "should be near transparent, got {}", faded);

        // Re-insert to re-spike: alpha jumps back to the peak.
        app.world_mut().entity_mut(ent).insert(ScreenFlash {
            peak_alpha: 0.4,
            decay: 3.0,
            despawn_on_end: false,
        });
        step(&mut app, 16);
        let spiked = app.world().get::<BackgroundColor>(ent).unwrap().0.alpha();
        assert!(
            spiked > 0.35,
            "re-insert should re-spike toward peak, got {}",
            spiked
        );
    }
}
