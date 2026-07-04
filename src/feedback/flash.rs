//! A Bevy plugin for a short material "hit flash".
//!
//! ## Overview
//!
//! `FlashPlugin` briefly overrides an entity's `StandardMaterial` color (its
//! emissive by default, or its base color) with a flash color, then eases it
//! back to the original over a duration and cleans up -- the white / red pop an
//! enemy or the player shows the instant it is hit.
//!
//! The interesting problem is doing this without corrupting a *shared* material:
//! many entities usually share one material handle, so mutating that asset in
//! place would flash all of them. Instead, on [`Flash`] insert the plugin
//! **clones** the entity's material into a fresh per-entity asset, swaps the
//! entity onto the clone, animates the clone, and on completion swaps the
//! original handle back and frees the clone. A remove observer frees the clone
//! even if the entity despawns mid-flash, so nothing leaks. Do not swap the
//! entity's material yourself while a flash is active: on completion the plugin
//! restores the handle it captured and would clobber your change. Re-inserting
//! [`Flash`] on an already-flashing entity restarts the flash from full.
//!
//! The component split follows the crate convention:
//!
//! 1. [`Flash`] - config: flash color, duration, which channel to flash.
//! 2. `FlashState` - private: the original + clone handles and elapsed time.
//!
//! ## Usage
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! # fn on_hit(mut commands: Commands, entity: Entity) {
//! // Flash the entity's emissive red for a quarter second, then ease back.
//! commands.entity(entity).insert(Flash {
//!     color: Color::srgb(1.0, 0.2, 0.2),
//!     duration: 0.25,
//!     ..default()
//! });
//! # }
//! ```
//!
//! The ease-back is a linear lerp for now; once the `tween` module exists it
//! should be backed by that instead.

use bevy::prelude::*;

pub mod prelude {
    pub use super::{Flash, FlashChannel, FlashPlugin, FlashSystems};
}

/// Which `StandardMaterial` channel a [`Flash`] overrides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub enum FlashChannel {
    /// Flash the emissive color (the default). Reads as a glow and blooms under
    /// `camera/post`. The material must be lit (not `unlit`) for emissive to
    /// show.
    #[default]
    Emissive,
    /// Flash the base color.
    BaseColor,
}

/// A short material hit-flash on an entity with a `StandardMaterial`.
///
/// Insert it on the hit entity; [`FlashPlugin`] overrides the chosen channel
/// with `color`, eases it back to the material's original value over `duration`,
/// and removes itself when done. The entity must carry a
/// `MeshMaterial3d<StandardMaterial>`; flashing a parent that only holds mesh
/// children has no effect (put the `Flash` on the child that owns the material).
#[derive(Component, Debug, Clone, Reflect)]
pub struct Flash {
    /// The color flashed onto the chosen channel at the start of the flash.
    pub color: Color,

    /// How long the flash takes to ease back to the original, in seconds.
    pub duration: f32,

    /// Which material channel to flash. Defaults to [`FlashChannel::Emissive`].
    pub channel: FlashChannel,
}

impl Default for Flash {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            duration: 0.2,
            channel: FlashChannel::Emissive,
        }
    }
}

/// Private per-flash state: the original (shared) material handle to restore,
/// the per-entity clone being animated, and elapsed time.
#[derive(Component, Debug, Reflect)]
struct FlashState {
    /// The material the entity had before the flash; restored on completion.
    original: Handle<StandardMaterial>,
    /// The per-entity clone being animated; freed when the flash ends.
    clone: Handle<StandardMaterial>,
    /// Seconds since the flash started.
    elapsed: f32,
}

/// System set for [`FlashPlugin`].
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum FlashSystems {
    /// Advances the flash, eases the clone back, and restores/cleans up.
    Animate,
}

/// Plugin that runs material hit-flashes.
pub struct FlashPlugin;

impl Plugin for FlashPlugin {
    fn build(&self, app: &mut App) {
        debug!("FlashPlugin: build");

        app.register_type::<Flash>()
            .register_type::<FlashChannel>()
            .register_type::<FlashState>();

        app.add_observer(on_insert_flash);
        app.add_observer(on_remove_flash_state);

        app.add_systems(Update, animate_flash.in_set(FlashSystems::Animate));
    }
}

/// Linearly mix from `original` toward `flash` by `k` (`k = 1` is fully the
/// flash color, `k = 0` is fully the original), preserving the original alpha.
fn flash_mix(original: LinearRgba, flash: LinearRgba, k: f32) -> LinearRgba {
    let k = k.clamp(0.0, 1.0);
    LinearRgba {
        red: original.red + (flash.red - original.red) * k,
        green: original.green + (flash.green - original.green) * k,
        blue: original.blue + (flash.blue - original.blue) * k,
        alpha: original.alpha,
    }
}

/// Reads the flashed channel's current value off a material.
fn channel_value(material: &StandardMaterial, channel: FlashChannel) -> LinearRgba {
    match channel {
        FlashChannel::Emissive => material.emissive,
        FlashChannel::BaseColor => material.base_color.to_linear(),
    }
}

/// Writes a value into the flashed channel of a material.
fn set_channel_value(material: &mut StandardMaterial, channel: FlashChannel, value: LinearRgba) {
    match channel {
        FlashChannel::Emissive => material.emissive = value,
        FlashChannel::BaseColor => material.base_color = Color::from(value),
    }
}

/// On [`Flash`] insert, clone the entity's material into a per-entity asset and
/// swap the entity onto it, so the shared original is never mutated. Uses
/// `On<Insert>` (not `On<Add>`) so a repeat flash on an already-flashing entity
/// restarts the animation from full rather than being ignored.
fn on_insert_flash(
    insert: On<Insert, Flash>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut q_state: Query<&mut FlashState>,
    q_material: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    let entity = insert.entity;
    trace!("on_insert_flash: entity {:?}", entity);

    // Already flashing: re-pop from full using the existing clone/original.
    if let Ok(mut state) = q_state.get_mut(entity) {
        state.elapsed = 0.0;
        return;
    }

    let Ok(mesh_material) = q_material.get(entity) else {
        // No StandardMaterial to flash: drop the Flash so it does not linger.
        commands.entity(entity).remove::<Flash>();
        return;
    };

    let original = mesh_material.0.clone();
    // Own the source material before `add` so the immutable borrow ends first.
    let Some(source) = materials.get(&original).cloned() else {
        commands.entity(entity).remove::<Flash>();
        return;
    };

    let clone = materials.add(source);
    commands.entity(entity).insert((
        MeshMaterial3d(clone.clone()),
        FlashState {
            original,
            clone,
            elapsed: 0.0,
        },
    ));
}

/// When a flash ends (or the entity despawns), free the per-entity clone so it
/// does not leak in `Assets`.
fn on_remove_flash_state(
    remove: On<Remove, FlashState>,
    q_state: Query<&FlashState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Ok(state) = q_state.get(remove.entity) {
        trace!(
            "on_remove_flash_state: freeing clone for {:?}",
            remove.entity
        );
        materials.remove(&state.clone);
    }
}

/// Ease each flashing material's clone from the flash color back to the original
/// over the duration; on completion restore the original handle and drop the
/// flash (which frees the clone via [`on_remove_flash_state`]).
fn animate_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut q_flash: Query<(Entity, &Flash, &mut FlashState)>,
) {
    let dt = time.delta_secs();

    for (entity, flash, mut state) in q_flash.iter_mut() {
        trace!(
            "animate_flash: entity {:?} elapsed {}",
            entity,
            state.elapsed
        );

        state.elapsed += dt;

        if flash.duration <= 0.0 || state.elapsed >= flash.duration {
            // Restore the shared original; removing FlashState frees the clone.
            commands
                .entity(entity)
                .try_insert(MeshMaterial3d(state.original.clone()))
                .remove::<(Flash, FlashState)>();
            continue;
        }

        // Fraction of the flash remaining: 1 at the start, 0 at the end.
        let k = 1.0 - state.elapsed / flash.duration;

        // Read the original channel value first (a Copy), then mutate the clone,
        // so the two Assets borrows do not overlap.
        let Some(original) = materials.get(&state.original) else {
            continue;
        };
        let original_value = channel_value(original, flash.channel);
        let flash_value = LinearRgba::from(flash.color);

        let Some(mut clone) = materials.get_mut(&state.clone) else {
            continue;
        };
        set_channel_value(
            &mut clone,
            flash.channel,
            flash_mix(original_value, flash_value, k),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flash_mix_endpoints_and_midpoint() {
        let orig = LinearRgba::new(0.1, 0.2, 0.3, 1.0);
        let flash = LinearRgba::new(1.0, 0.0, 0.0, 1.0);
        // k = 1 -> fully flash (alpha kept from original).
        let hot = flash_mix(orig, flash, 1.0);
        assert!((hot.red - 1.0).abs() < 1e-6 && hot.green.abs() < 1e-6);
        // k = 0 -> fully original.
        let cold = flash_mix(orig, flash, 0.0);
        assert!((cold.red - 0.1).abs() < 1e-6 && (cold.blue - 0.3).abs() < 1e-6);
        // k = 0.5 -> halfway on each channel.
        let mid = flash_mix(orig, flash, 0.5);
        assert!((mid.red - 0.55).abs() < 1e-6);
        assert!((mid.green - 0.1).abs() < 1e-6);
    }

    #[test]
    fn flash_mix_clamps_and_keeps_alpha() {
        let orig = LinearRgba::new(0.0, 0.0, 0.0, 0.5);
        let flash = LinearRgba::new(1.0, 1.0, 1.0, 1.0);
        // k past 1 clamps to the flash color; original alpha (0.5) is preserved.
        let v = flash_mix(orig, flash, 2.0);
        assert!((v.red - 1.0).abs() < 1e-6);
        assert!((v.alpha - 0.5).abs() < 1e-6);
    }

    fn flash_app() -> App {
        let mut app = App::new();
        app.add_plugins(AssetPlugin::default());
        app.init_asset::<StandardMaterial>();
        app.init_resource::<Time>();
        app.add_plugins(FlashPlugin);
        app
    }

    fn step(app: &mut App, dt_ms: u64) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(dt_ms));
        app.update();
    }

    #[test]
    fn flash_clones_the_material_leaving_shared_users_untouched() {
        let mut app = flash_app();

        // One shared material handle on two entities.
        let shared = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                emissive: LinearRgba::new(0.0, 0.0, 0.0, 1.0),
                ..default()
            });

        let flashed = app.world_mut().spawn(MeshMaterial3d(shared.clone())).id();
        let bystander = app.world_mut().spawn(MeshMaterial3d(shared.clone())).id();

        // Flash the first entity's emissive white.
        app.world_mut().entity_mut(flashed).insert(Flash {
            color: Color::WHITE,
            duration: 0.5,
            channel: FlashChannel::Emissive,
        });
        // Flush the on_add observer + run one animate step.
        step(&mut app, 50);

        // The flashed entity is now on a distinct clone handle...
        let flashed_handle = app
            .world()
            .get::<MeshMaterial3d<StandardMaterial>>(flashed)
            .unwrap()
            .0
            .clone();
        assert_ne!(
            flashed_handle, shared,
            "flashed entity should be on a clone"
        );
        // ...whose emissive has been pushed toward the flash color.
        let mats = app.world().resource::<Assets<StandardMaterial>>();
        assert!(
            mats.get(&flashed_handle).unwrap().emissive.red > 0.5,
            "flashed clone should be lit toward white"
        );

        // The bystander still points at the shared material, and it is untouched.
        let bystander_handle = app
            .world()
            .get::<MeshMaterial3d<StandardMaterial>>(bystander)
            .unwrap()
            .0
            .clone();
        assert_eq!(
            bystander_handle, shared,
            "bystander should keep the shared handle"
        );
        assert_eq!(
            mats.get(&shared).unwrap().emissive,
            LinearRgba::new(0.0, 0.0, 0.0, 1.0),
            "shared material must not be mutated"
        );
    }

    #[test]
    fn flash_restores_original_and_frees_clone_when_done() {
        let mut app = flash_app();

        let shared = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial::default());
        let flashed = app.world_mut().spawn(MeshMaterial3d(shared.clone())).id();

        app.world_mut().entity_mut(flashed).insert(Flash {
            color: Color::WHITE,
            duration: 0.2,
            channel: FlashChannel::Emissive,
        });
        step(&mut app, 50);
        let clone = app
            .world()
            .get::<MeshMaterial3d<StandardMaterial>>(flashed)
            .unwrap()
            .0
            .clone();
        assert_ne!(clone, shared);
        assert!(app
            .world()
            .resource::<Assets<StandardMaterial>>()
            .get(&clone)
            .is_some());

        // Run past the duration.
        for _ in 0..5 {
            step(&mut app, 100);
        }

        // The original handle is restored and Flash/FlashState are gone.
        let restored = app
            .world()
            .get::<MeshMaterial3d<StandardMaterial>>(flashed)
            .unwrap()
            .0
            .clone();
        assert_eq!(
            restored, shared,
            "original shared handle should be restored"
        );
        assert!(
            app.world().get::<Flash>(flashed).is_none(),
            "Flash should be removed"
        );
        // The clone asset is freed (no leak).
        assert!(
            app.world()
                .resource::<Assets<StandardMaterial>>()
                .get(&clone)
                .is_none(),
            "clone material should be freed"
        );
    }

    #[test]
    fn reflashing_restarts_the_animation() {
        let mut app = flash_app();

        let shared = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial::default());
        let flashed = app.world_mut().spawn(MeshMaterial3d(shared.clone())).id();

        app.world_mut().entity_mut(flashed).insert(Flash {
            color: Color::WHITE,
            duration: 0.5,
            channel: FlashChannel::Emissive,
        });
        // Age the flash most of the way through (elapsed ~0.4 of 0.5).
        for _ in 0..4 {
            step(&mut app, 100);
        }
        let clone = app
            .world()
            .get::<MeshMaterial3d<StandardMaterial>>(flashed)
            .unwrap()
            .0
            .clone();

        // Re-flash: On<Insert> must reset elapsed so it re-pops from full and
        // reuses the same clone (no leak of a second clone).
        app.world_mut().entity_mut(flashed).insert(Flash {
            color: Color::WHITE,
            duration: 0.5,
            channel: FlashChannel::Emissive,
        });
        step(&mut app, 50);

        // Same clone handle (not a fresh one).
        let after = app
            .world()
            .get::<MeshMaterial3d<StandardMaterial>>(flashed)
            .unwrap()
            .0
            .clone();
        assert_eq!(after, clone, "re-flash should reuse the existing clone");
        // Still bright: at elapsed ~0.05 of 0.5 the emissive is near-full white,
        // which it would NOT be had elapsed stayed at ~0.45 (near restored).
        let emissive = app
            .world()
            .resource::<Assets<StandardMaterial>>()
            .get(&after)
            .unwrap()
            .emissive;
        assert!(
            emissive.red > 0.8,
            "re-flash should restart near full brightness, got {}",
            emissive.red
        );
    }

    #[test]
    fn flash_without_material_is_dropped() {
        let mut app = flash_app();
        // An entity with no MeshMaterial3d: the Flash must not linger.
        let ent = app.world_mut().spawn(Flash::default()).id();
        app.update();
        assert!(
            app.world().get::<Flash>(ent).is_none(),
            "a Flash on a material-less entity should be removed"
        );
    }

    #[test]
    fn despawn_mid_flash_frees_clone() {
        let mut app = flash_app();

        let shared = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial::default());
        let flashed = app.world_mut().spawn(MeshMaterial3d(shared.clone())).id();
        app.world_mut().entity_mut(flashed).insert(Flash {
            duration: 1.0,
            ..default()
        });
        step(&mut app, 50);
        let clone = app
            .world()
            .get::<MeshMaterial3d<StandardMaterial>>(flashed)
            .unwrap()
            .0
            .clone();
        assert!(app
            .world()
            .resource::<Assets<StandardMaterial>>()
            .get(&clone)
            .is_some());

        // Despawn while still flashing; the remove observer must free the clone.
        app.world_mut().entity_mut(flashed).despawn();
        app.update();
        assert!(
            app.world()
                .resource::<Assets<StandardMaterial>>()
                .get(&clone)
                .is_none(),
            "clone material should be freed on despawn"
        );
    }
}
