//! The physics half of the integrity pipeline: turn collisions into
//! [`HealthApplyDamage`](crate::health::HealthApplyDamage).
//!
//! Two damage sources, both observers over avian's `CollisionStart`:
//!
//! - a fast impact between two rigid bodies deals impulse/energy damage scaled by their
//!   relative velocity and effective mass (`on_impact_collision_deal_damage`);
//! - a [`blast`](super::blast) sensor deals radial damage falling off linearly to zero at its
//!   radius (`on_blast_collision_deal_damage`).
//!
//! Everything downstream of the damage - disabling a node at zero health, destroying leaves,
//! cascading through the graph - lives in [`super::plugin`]. [`IntegrityPlugin`] registers
//! both halves.
//!
//! [`IntegrityPlugin`]: super::plugin::IntegrityPlugin

use avian3d::prelude::*;
use bevy::prelude::*;

use super::blast::*;
use crate::health::prelude::*;

const RESTITUTION_COEFFICIENT: f32 = 0.5;
const IMPULSE_DAMAGE_MODIFIER: f32 = 0.1;
const ENERGY_DAMAGE_MODIFIER: f32 = 0.05;

/// Opt a health-bearing collider into collision events, so impacts against it are reported.
pub(super) fn on_collider_of_spawn_insert_collision_events(
    add: On<Add, ColliderOf>,
    mut commands: Commands,
    q_collider: Query<Entity, (With<ColliderOf>, With<Health>)>,
) {
    let entity = add.entity;
    trace!("on_collider_of_spawn: entity {:?}", entity);

    let Ok(_) = q_collider.get(entity) else {
        trace!(
            "on_collider_of_spawn: entity {:?} not found in q_collider",
            entity
        );
        return;
    };

    debug!(
        "on_collider_of_spawn: adding CollisionEventsEnabled to entity {:?}",
        entity
    );
    commands.entity(entity).insert(CollisionEventsEnabled);
}

/// Damage a body from the impulse and energy lost in a fast impact against another body.
pub(super) fn on_impact_collision_deal_damage(
    collision: On<CollisionStart>,
    mut commands: Commands,
    q_body: Query<(&LinearVelocity, &ComputedMass), With<RigidBody>>,
    // NOTE: excluding BlastDamageMarker keeps a blast overlap from also dealing impact damage.
    q_other: Query<(&LinearVelocity, &ComputedMass), (With<RigidBody>, Without<BlastDamageMarker>)>,
) {
    trace!(
        "on_impact_collision_event: collision between {:?} and {:?}",
        collision.body1,
        collision.body2
    );

    let collider1 = collision.collider1;
    let collider2 = collision.collider2;

    let Some(body) = collision.body1 else {
        return;
    };
    let Some(other) = collision.body2 else {
        return;
    };

    let Ok((velocity1, mass1)) = q_body.get(body) else {
        return;
    };
    let Ok((velocity2, mass2)) = q_other.get(other) else {
        return;
    };

    let relative_velocity = **velocity1 - **velocity2;
    if relative_velocity.length_squared() < 0.1 {
        return;
    }

    let effective_mass = (mass1.value() * mass2.value()) / (mass1.value() + mass2.value());
    let impulse = effective_mass * (1.0 + RESTITUTION_COEFFICIENT) * relative_velocity.length();
    let energy_lost = 0.5
        * effective_mass
        * (1.0 - RESTITUTION_COEFFICIENT.powi(2))
        * relative_velocity.length_squared();

    let damage = impulse * IMPULSE_DAMAGE_MODIFIER + energy_lost * ENERGY_DAMAGE_MODIFIER;
    if damage <= f32::EPSILON {
        return;
    }
    debug!(
        "on_impact_collision_event: collider {:?} (body {:?}) hit by collider {:?} (other {:?}) for damage {:.2}",
        collider1, body, collider2, other, damage
    );
    commands.trigger(HealthApplyDamage {
        entity: collider1,
        source: Some(collider2),
        amount: damage,
    });
}

/// Apply radial blast damage to a body that overlaps a blast sensor.
///
/// The blast sensor is the "self" side of the event (`collider1`/`body1`): it carries
/// `CollisionEventsEnabled` (see `blast_damage`), so avian raises `CollisionStart` with the
/// blast as `body1` against every collider it overlaps. This is why the blast owns its events
/// rather than relying on each target - a body only takes blast damage if *some* collider in
/// the pair has events enabled, and keying that on the blast means it reaches every overlapped
/// body regardless of the target's own configuration.
///
/// avian raises the swapped `{body1 = target, body2 = blast}` event too whenever the target
/// also has events enabled (e.g. a health-bearing node, for impact damage). We ignore that
/// ordering here - `q_blast.get(body1)` fails when `body1` is the target - so each overlap
/// deals damage exactly once and never double-dips.
pub(super) fn on_blast_collision_deal_damage(
    collision: On<CollisionStart>,
    mut commands: Commands,
    q_blast: Query<(&Transform, &BlastDamageConfig), With<BlastDamageMarker>>,
    // NOTE: distance is measured between body origins, not the nearest points of the colliders,
    // so a large body is judged by its centre.
    q_body: Query<&Transform, With<RigidBody>>,
) {
    trace!(
        "on_blast_collision_event: collision between {:?} and {:?}",
        collision.body1,
        collision.body2
    );

    let blast_collider = collision.collider1;
    let target_collider = collision.collider2;

    let Some(blast) = collision.body1 else {
        return;
    };
    let Some(target) = collision.body2 else {
        return;
    };

    let Ok((blast_transform, blast_config)) = q_blast.get(blast) else {
        return;
    };
    let Ok(target_transform) = q_body.get(target) else {
        return;
    };

    let distance = blast_transform
        .translation
        .distance(target_transform.translation);
    let damage = calculate_blast_damage(distance, blast_config);
    if damage <= f32::EPSILON {
        return;
    };

    debug!(
        "on_blast_collision_start_event: applying blast damage {:.2} to collider {:?} (body {:?}) from blast collider {:?} (blast {:?})",
        damage, target_collider, target, blast_collider, blast
    );
    commands.trigger(HealthApplyDamage {
        entity: target_collider,
        source: Some(blast_collider),
        amount: damage,
    });
}

/// Blast damage at `distance` from the centre: `max_damage` falling off linearly to zero at
/// `radius`, and zero beyond it.
fn calculate_blast_damage(distance: f32, config: &BlastDamageConfig) -> f32 {
    if distance >= config.radius {
        0.0
    } else {
        let falloff = 1.0 - (distance / config.radius);
        config.max_damage * falloff
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blast_damage_falls_off_linearly_to_the_radius() {
        let config = BlastDamageConfig {
            radius: 10.0,
            max_damage: 100.0,
        };
        assert_eq!(calculate_blast_damage(0.0, &config), 100.0);
        assert!((calculate_blast_damage(5.0, &config) - 50.0).abs() < 1e-3);
        assert_eq!(calculate_blast_damage(10.0, &config), 0.0);
        assert_eq!(calculate_blast_damage(20.0, &config), 0.0);
    }
}

/// Physics-level tests for the collision-driven damage observers. Unlike the cascade tests in
/// [`super::plugin`] (which drive the avian-free core by hand), these run a real headless
/// avian world so the observers read genuine `ComputedMass` / `Transform` / `ColliderOf` state.
#[cfg(test)]
mod physics_tests {
    use super::*;
    use crate::integrity::test_support::{integrity_physics_app, settle};

    /// Spawn a dynamic rigid body with a unit-sphere collider child that carries `Health`.
    /// Returns `(body, collider)`. Placement is far from the origin so bodies never actually
    /// touch - the tests inject the `CollisionStart` themselves for determinism.
    fn spawn_body(app: &mut App, at: Vec3) -> (Entity, Entity) {
        let body = app
            .world_mut()
            .spawn((RigidBody::Dynamic, Transform::from_translation(at)))
            .id();
        let collider = app
            .world_mut()
            .spawn((
                ChildOf(body),
                Collider::sphere(1.0),
                ColliderDensity(1.0),
                Health::new(1000.0),
            ))
            .id();
        (body, collider)
    }

    fn health(app: &App, entity: Entity) -> f32 {
        app.world().get::<Health>(entity).unwrap().current
    }

    #[test]
    fn an_impact_applies_damage_from_relative_velocity_and_mass() {
        // NOTE: a real collision is left to the solver, which zeroes the contact velocity before
        // the observer can read it, so sim-driven damage would be timing-dependent. Instead avian
        // computes real masses and the test injects the contact event at a known velocity.
        let mut app = integrity_physics_app();
        let (b1, c1) = spawn_body(&mut app, Vec3::new(-100.0, 0.0, 0.0));
        let (b2, c2) = spawn_body(&mut app, Vec3::new(100.0, 0.0, 0.0));
        settle(&mut app);

        app.world_mut().get_mut::<LinearVelocity>(b1).unwrap().0 = Vec3::new(20.0, 0.0, 0.0);
        app.world_mut().get_mut::<LinearVelocity>(b2).unwrap().0 = Vec3::new(-20.0, 0.0, 0.0);

        let m1 = app.world().get::<ComputedMass>(b1).unwrap().value();
        let m2 = app.world().get::<ComputedMass>(b2).unwrap().value();
        assert!(m1.is_finite() && m1 > 0.0, "mass should be finalized: {m1}");

        app.world_mut().trigger(CollisionStart {
            collider1: c1,
            collider2: c2,
            body1: Some(b1),
            body2: Some(b2),
        });
        app.update();

        // NOTE: the oracle is recomputed from the real mass and the module's own constants, so
        // this checks the wiring + physics state rather than a hard-coded magic number.
        let rel = 40.0_f32; // head-on closing velocity, 20 - -20
        let effective_mass = (m1 * m2) / (m1 + m2);
        let impulse = effective_mass * (1.0 + RESTITUTION_COEFFICIENT) * rel;
        let energy = 0.5 * effective_mass * (1.0 - RESTITUTION_COEFFICIENT.powi(2)) * rel * rel;
        let expected = impulse * IMPULSE_DAMAGE_MODIFIER + energy * ENERGY_DAMAGE_MODIFIER;

        // NOTE: damage lands on collider1 and only there - collider2 is the source, not a target.
        assert!((health(&app, c1) - (1000.0 - expected)).abs() < 1e-2);
        assert_eq!(health(&app, c2), 1000.0);
    }

    #[test]
    fn a_near_stationary_contact_applies_no_impact_damage() {
        // NOTE: the velocity gate is what keeps resting stacks of debris from grinding each
        // other away, so a graze below it must deal nothing.
        let mut app = integrity_physics_app();
        let (b1, c1) = spawn_body(&mut app, Vec3::new(-100.0, 0.0, 0.0));
        let (b2, c2) = spawn_body(&mut app, Vec3::new(100.0, 0.0, 0.0));
        settle(&mut app);

        app.world_mut().get_mut::<LinearVelocity>(b1).unwrap().0 = Vec3::new(0.01, 0.0, 0.0);
        app.world_mut().get_mut::<LinearVelocity>(b2).unwrap().0 = Vec3::ZERO;

        app.world_mut().trigger(CollisionStart {
            collider1: c1,
            collider2: c2,
            body1: Some(b1),
            body2: Some(b2),
        });
        app.update();

        assert_eq!(health(&app, c1), 1000.0);
    }

    /// Spawn a blast sensor via the production `blast_damage` bundle at `at`.
    fn spawn_blast(app: &mut App, at: Vec3, radius: f32, max_damage: f32) -> Entity {
        app.world_mut()
            .spawn((
                blast_damage(BlastDamageConfig { radius, max_damage }),
                Transform::from_translation(at),
            ))
            .id()
    }

    #[test]
    fn a_blast_sensor_overlap_applies_falloff_damage() {
        // NOTE: unlike the impact case a sensor overlap raises a real, deterministic
        // `CollisionStart` (no solver to zero it out), so this drives the whole path through
        // avian: overlap detection, both transforms, then the falloff.
        let mut app = integrity_physics_app();
        let (_body, target_collider) = spawn_body(&mut app, Vec3::ZERO);
        spawn_blast(&mut app, Vec3::new(4.0, 0.0, 0.0), 10.0, 100.0);

        settle(&mut app);

        let expected = calculate_blast_damage(
            4.0,
            &BlastDamageConfig {
                radius: 10.0,
                max_damage: 100.0,
            },
        );
        assert!((expected - 60.0).abs() < 1e-3, "sanity: {expected}");
        assert!((health(&app, target_collider) - (1000.0 - expected)).abs() < 1e-2);
    }

    #[test]
    fn a_blast_reaches_a_target_that_has_no_collision_events() {
        // BUG: regression for an ordering bug - the blast must not depend on the target having
        // `CollisionEventsEnabled`. Before the fix the only event raised was the target's, which
        // a target like this one never raises, so no damage landed.
        let mut app = integrity_physics_app();
        let body = app
            .world_mut()
            .spawn((RigidBody::Dynamic, Transform::default()))
            .id();
        let target_collider = app
            .world_mut()
            .spawn((ChildOf(body), Collider::sphere(1.0), ColliderDensity(1.0)))
            .id(); // no Health at ColliderOf time, so no events get enabled
        settle(&mut app);
        assert!(
            app.world()
                .get::<CollisionEventsEnabled>(target_collider)
                .is_none(),
            "target must not have opted into collision events for this regression to be meaningful"
        );
        // NOTE: adding Health now does NOT enable events - that observer keys on ColliderOf.
        app.world_mut()
            .entity_mut(target_collider)
            .insert(Health::new(1000.0));

        spawn_blast(&mut app, Vec3::new(4.0, 0.0, 0.0), 10.0, 100.0);
        settle(&mut app);

        assert!(
            (health(&app, target_collider) - 940.0).abs() < 1e-2,
            "blast should reach a target that has no collision events of its own"
        );
    }

    #[test]
    fn a_blast_deals_damage_only_once_when_the_target_also_has_events() {
        // NOTE: with events on both sides avian raises both orderings of the pair; the observer
        // acts only on the blast-as-self ordering, so the target takes 60, not 120.
        let mut app = integrity_physics_app();
        let (_body, target_collider) = spawn_body(&mut app, Vec3::ZERO);
        settle(&mut app);
        assert!(
            app.world()
                .get::<CollisionEventsEnabled>(target_collider)
                .is_some(),
            "a Health-bearing target should have its own collision events"
        );

        spawn_blast(&mut app, Vec3::new(4.0, 0.0, 0.0), 10.0, 100.0);
        settle(&mut app);

        assert!((health(&app, target_collider) - 940.0).abs() < 1e-2);
    }

    #[test]
    fn a_body_outside_the_blast_takes_no_damage() {
        let mut app = integrity_physics_app();
        let (_body, target_collider) = spawn_body(&mut app, Vec3::ZERO);
        // NOTE: radius 5 centred 8 away leaves the unit-sphere target ~7 out, clear of the
        // sensor, so avian never raises a collision at all.
        spawn_blast(&mut app, Vec3::new(8.0, 0.0, 0.0), 5.0, 100.0);

        settle(&mut app);

        assert_eq!(health(&app, target_collider), 1000.0);
    }
}
