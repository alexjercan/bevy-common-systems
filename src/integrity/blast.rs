//! A radial blast-damage volume: a sensor that deals falloff damage to every body it
//! overlaps.
//!
//! Spawn the [`blast_damage`] bundle at a world position and the [`super::plugin`] observers
//! turn each overlap into a [`HealthApplyDamage`](crate::health::HealthApplyDamage) scaled by
//! distance from the blast centre (linear falloff to zero at `radius`). Pair it with a short
//! [`TempEntity`](crate::helpers::temp::TempEntity) so the volume cleans itself up after the
//! frame it fires.
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! # fn detonate(mut commands: Commands, at: Vec3) {
//! commands.spawn((
//!     blast_damage(BlastDamageConfig { radius: 6.0, max_damage: 80.0 }),
//!     Transform::from_translation(at),
//!     TempEntity(0.1),
//! ));
//! # }
//! ```

use avian3d::prelude::*;
use bevy::prelude::*;

pub mod prelude {
    pub use super::{blast_damage, BlastDamageConfig, BlastDamageMarker};
}

/// Configures a blast: everything within `radius` takes up to `max_damage`, falling off
/// linearly to zero at the edge.
// NOTE: linear falloff for now; other falloff models could be added later.
#[derive(Component, Debug, Clone, Reflect)]
pub struct BlastDamageConfig {
    /// Radius of the blast sensor. Bodies beyond this take no damage.
    pub radius: f32,
    /// Damage dealt at the blast centre (distance 0).
    pub max_damage: f32,
}

/// Marker for a blast-damage sensor entity, so the damage observers can tell the blast side
/// of a collision from the target side.
#[derive(Component, Debug, Clone, Reflect)]
pub struct BlastDamageMarker;

/// Bundle for a radial blast-damage volume. Spawn it with a [`Transform`] at the blast
/// centre; see the module docs.
pub fn blast_damage(config: BlastDamageConfig) -> impl Bundle {
    debug!(
        "blast_damage: radius {:.2}, max_damage {:.2}",
        config.radius, config.max_damage
    );

    (
        Name::new("BlastDamageArea"),
        BlastDamageMarker,
        config.clone(),
        RigidBody::Static,
        Collider::sphere(config.radius),
        Sensor,
        // The blast owns its collision events so it raises `CollisionStart` against every
        // collider it overlaps, instead of depending on each target having events enabled
        // (see `on_blast_collision_deal_damage`). Without this the blast only reaches bodies
        // that independently opted into collision events.
        CollisionEventsEnabled,
        Visibility::Visible,
    )
}
