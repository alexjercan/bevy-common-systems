//! A Bevy plugin for short-lived floating "+N" popup text.
//!
//! ## Overview
//!
//! `PopupPlugin` animates a screen-space UI text label that rises up the screen
//! and fades out over a lifetime, then despawns itself -- the floating "+N"
//! score / "+FUEL" pickup / "-10" damage text that almost every game shows on a
//! pickup or a hit.
//!
//! The popup is a plain Bevy UI `Text` node with a [`Popup`] component; the
//! plugin advances it. Because it is screen-space, the caller decides *where* on
//! screen it appears: for a world event, project the world position to a
//! viewport point yourself (see [`camera::project::world_to_screen`]) and pass
//! that in; for a fixed banner, pass a constant screen point. Keeping the
//! projection in the caller is deliberate -- it varies per game, so the module
//! stays free of a camera handle.
//!
//! [`camera::project::world_to_screen`]: crate::camera::project::world_to_screen
//!
//! The component split follows the crate convention:
//!
//! 1. [`Popup`] - config: lifetime, rise speed, base color.
//! 2. `PopupState` - private per-popup age.
//!
//! Spawn the common case with the [`popup`] bundle builder; for a custom layout
//! (a centered banner, a different rise) put a [`Popup`] on your own `Text` /
//! `Node` instead.
//!
//! ## Usage
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! # fn on_pickup(mut commands: Commands, viewport_pos: Vec2) {
//! // A "+10" that rises from a projected screen point and fades out.
//! commands.spawn(popup(viewport_pos, "+10", 28.0, Color::WHITE));
//! # }
//! ```
//!
//! The rise/fade is a bespoke lerp for now; once the `tween` module exists it
//! should be backed by that instead.

use bevy::prelude::*;

pub mod prelude {
    pub use super::{popup, Popup, PopupPlugin, PopupSystems};
}

/// Configuration for a floating popup label on a UI text entity.
///
/// Put this on any UI `Text` node (the [`popup`] builder does it for you) and
/// [`PopupPlugin`] will rise it up the screen, ramp its `TextColor` alpha to
/// zero over `lifetime`, and despawn it when the lifetime is up.
#[derive(Component, Debug, Clone, Reflect)]
#[require(PopupState)]
pub struct Popup {
    /// Total lifetime in seconds; the popup despawns once its age reaches this.
    pub lifetime: f32,

    /// Upward screen speed in pixels per second. Applied to the node's `top`
    /// (only when `top` is a `Val::Px`).
    pub rise_speed: f32,

    /// Base color. Its alpha is ramped from its starting value down to zero as
    /// the popup ages, so a translucent base color fades from that alpha.
    pub base_color: Color,
}

impl Default for Popup {
    fn default() -> Self {
        Self {
            lifetime: 0.8,
            rise_speed: 70.0,
            base_color: Color::WHITE,
        }
    }
}

/// Private per-popup age, in seconds since spawn. Managed by the plugin.
#[derive(Component, Default, Debug, Reflect)]
struct PopupState {
    age: f32,
}

/// System set for [`PopupPlugin`].
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PopupSystems {
    /// Advances popup age, rises and fades them, and despawns expired ones.
    Animate,
}

/// Plugin that animates and despawns floating [`Popup`] text.
pub struct PopupPlugin;

impl Plugin for PopupPlugin {
    fn build(&self, app: &mut App) {
        debug!("PopupPlugin: build");

        app.register_type::<Popup>().register_type::<PopupState>();

        app.add_systems(Update, animate_popups.in_set(PopupSystems::Animate));
    }
}

/// The fade ramp: fraction of the base alpha remaining at `age` of `lifetime`.
/// Linear from `base_alpha` at age 0 to 0 at (and past) the lifetime.
fn popup_alpha(age: f32, lifetime: f32, base_alpha: f32) -> f32 {
    if lifetime <= 0.0 {
        return 0.0;
    }
    let ramp = (1.0 - age / lifetime).clamp(0.0, 1.0);
    base_alpha * ramp
}

/// Advance floating popups: age them, rise them up the screen, fade them out,
/// and despawn them at the end of their lifetime.
///
/// `Node` and `TextColor` are optional so an expired popup is despawned even if
/// it lacks either (rise needs the node, fade needs the text color).
fn animate_popups(
    mut commands: Commands,
    time: Res<Time>,
    mut q_popup: Query<(
        Entity,
        &Popup,
        &mut PopupState,
        Option<&mut Node>,
        Option<&mut TextColor>,
    )>,
) {
    let dt = time.delta_secs();

    for (entity, popup, mut state, node, text_color) in q_popup.iter_mut() {
        trace!("animate_popups: entity {:?} age {}", entity, state.age);

        state.age += dt;
        if state.age >= popup.lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        if let Some(mut node) = node {
            if let Val::Px(top) = node.top {
                node.top = Val::Px(top - popup.rise_speed * dt);
            }
        }

        if let Some(mut text_color) = text_color {
            let alpha = popup_alpha(state.age, popup.lifetime, popup.base_color.alpha());
            text_color.0 = popup.base_color.with_alpha(alpha);
        }
    }
}

/// Bundle builder for the common floating "+N" popup: a screen-space `Text`
/// label anchored at an absolute viewport `position` (logical px), sized and
/// colored, that rises and fades with the default [`Popup`] feel.
///
/// Chain scoping onto the spawn, e.g.
/// `commands.spawn(popup(pos, "+10", 28.0, color)).insert(DespawnOnExit(state))`.
pub fn popup(position: Vec2, text: impl Into<String>, font_size: f32, color: Color) -> impl Bundle {
    (
        Name::new("Popup"),
        Popup {
            base_color: color,
            ..default()
        },
        Text::new(text.into()),
        TextFont {
            font_size: FontSize::Px(font_size),
            ..default()
        },
        TextColor(color),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(position.x),
            top: Val::Px(position.y),
            ..default()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_alpha_ramps_from_base_to_zero() {
        // Full base alpha at birth.
        assert!((popup_alpha(0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
        // Half base alpha at half life.
        assert!((popup_alpha(0.5, 1.0, 1.0) - 0.5).abs() < 1e-6);
        // Zero at (and past) the lifetime, clamped.
        assert_eq!(popup_alpha(1.0, 1.0, 1.0), 0.0);
        assert_eq!(popup_alpha(2.0, 1.0, 1.0), 0.0);
    }

    #[test]
    fn popup_alpha_respects_base_alpha() {
        // A translucent base color fades from its own alpha, not from 1.0.
        assert!((popup_alpha(0.0, 1.0, 0.6) - 0.6).abs() < 1e-6);
        assert!((popup_alpha(0.5, 1.0, 0.6) - 0.3).abs() < 1e-6);
    }

    #[test]
    fn popup_alpha_handles_zero_lifetime() {
        assert_eq!(popup_alpha(0.0, 0.0, 1.0), 0.0);
    }

    fn step(app: &mut App, dt_ms: u64) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(dt_ms));
        app.update();
    }

    #[test]
    fn popup_rises_fades_and_despawns() {
        let mut app = App::new();
        app.init_resource::<Time>();
        app.add_plugins(PopupPlugin);

        let start_top = 200.0;
        let ent = app
            .world_mut()
            .spawn((
                Popup {
                    lifetime: 0.5,
                    rise_speed: 100.0,
                    base_color: Color::WHITE,
                },
                Node {
                    top: Val::Px(start_top),
                    ..default()
                },
                TextColor(Color::WHITE),
            ))
            .id();

        // One 100ms step: the popup rose (top decreased) and faded (alpha < 1).
        step(&mut app, 100);
        let node = app.world().get::<Node>(ent).unwrap();
        let Val::Px(top) = node.top else {
            panic!("top should be Px");
        };
        assert!(
            top < start_top,
            "popup should rise (top {top} < {start_top})"
        );
        let alpha = app.world().get::<TextColor>(ent).unwrap().0.alpha();
        assert!(
            alpha < 1.0 && alpha > 0.0,
            "alpha should be fading: {alpha}"
        );

        // Past the lifetime the popup despawns itself.
        for _ in 0..6 {
            step(&mut app, 100);
        }
        assert!(
            app.world().get_entity(ent).is_err(),
            "popup should have despawned after its lifetime"
        );
    }

    #[test]
    fn bare_popup_without_node_or_text_despawns() {
        // The animate system queries Node/TextColor optionally so a Popup with
        // neither still ages and despawns (rather than leaking).
        let mut app = App::new();
        app.init_resource::<Time>();
        app.add_plugins(PopupPlugin);

        let ent = app
            .world_mut()
            .spawn(Popup {
                lifetime: 0.3,
                ..default()
            })
            .id();

        for _ in 0..5 {
            step(&mut app, 100);
        }
        assert!(
            app.world().get_entity(ent).is_err(),
            "a Node/TextColor-less popup should still despawn after its lifetime"
        );
    }
}
