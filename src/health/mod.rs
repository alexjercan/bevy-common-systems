//! Health component and related systems for Bevy games.
//!
//! This plugin provides a simple health system for game entities.
//!
//! Features:
//! - `Health` component to track current and maximum health.
//! - `HealthApplyDamage` event to apply damage to entities.
//! - `HealthZeroMarker` component added when an entity's health reaches zero.
//!
//! Usage:
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! # fn demo(mut commands: Commands, entity: Entity, player_entity: Entity) {
//! commands.spawn((
//!     Health::new(100.0),
//! ));
//!
//! // Apply damage from some system
//! commands.trigger(HealthApplyDamage {
//!     entity,
//!     source: Some(player_entity),
//!     amount: 25.0,
//! });
//! # }
//! ```

use bevy::prelude::*;

pub mod prelude {
    pub use super::{
        Health, HealthApplyDamage, HealthPlugin, HealthPluginSystems, HealthZeroMarker,
    };
}

/// Component representing the health of an entity.
///
/// Contains current and maximum health values. Health cannot exceed `max`
/// and should not drop below 0.
#[derive(Component, Clone, Debug, Reflect)]
pub struct Health {
    /// Current health value.
    pub current: f32,

    /// Maximum health value.
    pub max: f32,
}

impl Health {
    /// Create a new Health component with `current` equal to `max`.
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }
}

/// Marker component indicating that an entity has been destroyed.
///
/// This is automatically added by the `on_damage` system when an entity's
/// health reaches zero. You can use this marker to trigger destruction logic
/// like removing the entity, playing effects, or spawning loot.
#[derive(Component, Clone, Debug, Reflect)]
pub struct HealthZeroMarker;

/// Event to apply damage to a target entity.
///
/// `amount` is subtracted from the target's current health. If health reaches
/// zero or below, the `HealthZeroMarker` is added.
#[derive(EntityEvent, Clone, Debug)]
#[entity_event(propagate, auto_propagate)]
pub struct HealthApplyDamage {
    /// The entity receiving damage.
    pub entity: Entity,

    /// TODO: Maybe make this `source` more configurable? - what if we can also specify stuff like
    /// damage type, critical hit, etc.?
    /// Optional source entity causing the damage.
    pub source: Option<Entity>,

    /// Amount of damage to apply.
    pub amount: f32,
}

/// System set for the Health plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum HealthPluginSystems {
    /// Systems responsible for syncing health and applying damage.
    Sync,
}

/// Plugin that enables the Health component and related systems.
#[derive(Default)]
pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        debug!("HealthPlugin: build");

        // Listen for damage events and apply them to entities
        app.add_observer(on_damage);
    }
}

/// System to handle `HealthApplyDamage` events.
///
/// Reduces the target's current health by the damage amount. If health
/// reaches zero, adds `HealthZeroMarker`.
///
/// The event still bubbles up the hierarchy so parent entities (e.g. an aggregate
/// hull that sums its sections) react to a hit on a child. Crucially, we bubble up
/// only the amount that *actually landed* on this node, not the raw incoming amount:
/// `damage.amount` is clamped to the node's remaining health before propagation
/// continues. This is what stops overkill on a child from teleporting into a parent
/// aggregate - a 1000-damage hit on a 100 hp section costs the parent 100, not 1000.
/// Entity-event propagation reuses this same event instance for every ancestor, so
/// mutating `damage.amount` here is what the next node up sees.
///
/// A node that is already destroyed or at zero health absorbs nothing, so it zeroes
/// the propagated amount: hitting a corpse must not charge its parents.
fn on_damage(
    mut damage: On<HealthApplyDamage>,
    mut commands: Commands,
    mut q_health: Query<(Entity, &mut Health, Has<HealthZeroMarker>)>,
) {
    let target = damage.entity;
    trace!("on_damage: target {:?}, damage {:?}", target, damage.amount);

    let Ok((entity, mut health, destroyed)) = q_health.get_mut(target) else {
        // No `Health` on this node (e.g. an intermediate transform parent). Leave the
        // amount unchanged so it keeps bubbling to the next ancestor that does have one.
        trace!("on_damage: entity {:?} not found in q_health", target);
        return;
    };

    if destroyed {
        trace!("on_damage: entity {:?} is already destroyed", entity);
        damage.amount = 0.0;
        return;
    }

    if health.current <= 0.0 {
        trace!("on_damage: entity {:?} health is already zero", entity);
        damage.amount = 0.0;
        return;
    }

    // Apply at most the node's remaining health, and propagate only what landed.
    let applied = damage.amount.min(health.current);
    health.current -= applied;
    damage.amount = applied;
    if health.current <= 0.0 {
        health.current = 0.0;
        commands.entity(entity).insert(HealthZeroMarker);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn health_app() -> App {
        let mut app = App::new();
        app.add_plugins(HealthPlugin);
        app
    }

    /// Overkill on a child must not teleport into its parent aggregate: the parent
    /// is charged only what the child could actually absorb (its remaining health).
    #[test]
    fn overkill_on_a_child_only_costs_the_parent_the_childs_remaining_health() {
        let mut app = health_app();
        let parent = app.world_mut().spawn(Health::new(200.0)).id();
        let child = app
            .world_mut()
            .spawn((Health::new(100.0), ChildOf(parent)))
            .id();

        // A 1000-damage hit on the 100 hp child.
        app.world_mut().trigger(HealthApplyDamage {
            entity: child,
            source: None,
            amount: 1000.0,
        });
        app.world_mut().flush();

        // The child is destroyed...
        assert_eq!(app.world().get::<Health>(child).unwrap().current, 0.0);
        assert!(app.world().get::<HealthZeroMarker>(child).is_some());

        // ...but the parent loses only the child's 100 and survives.
        assert_eq!(app.world().get::<Health>(parent).unwrap().current, 100.0);
        assert!(app.world().get::<HealthZeroMarker>(parent).is_none());
    }

    /// The clamp must not break fatal propagation: when the parent aggregate equals
    /// the dying child, the lethal hit still bubbles up and zeroes the parent.
    #[test]
    fn a_lethal_hit_still_bubbles_to_zero_a_matching_parent() {
        let mut app = health_app();
        let parent = app.world_mut().spawn(Health::new(100.0)).id();
        let child = app
            .world_mut()
            .spawn((Health::new(100.0), ChildOf(parent)))
            .id();

        app.world_mut().trigger(HealthApplyDamage {
            entity: child,
            source: None,
            amount: 1000.0,
        });
        app.world_mut().flush();

        assert_eq!(app.world().get::<Health>(parent).unwrap().current, 0.0);
        assert!(app.world().get::<HealthZeroMarker>(child).is_some());
        assert!(app.world().get::<HealthZeroMarker>(parent).is_some());
    }

    /// Hitting an already-dead child must not charge the parent at all: a corpse
    /// absorbs nothing, so the propagated amount is zero.
    #[test]
    fn hitting_a_destroyed_child_does_not_charge_the_parent() {
        let mut app = health_app();
        let parent = app.world_mut().spawn(Health::new(200.0)).id();
        let child = app
            .world_mut()
            .spawn((Health::new(100.0), ChildOf(parent)))
            .id();

        // Kill the child exactly; parent drops by the child's 100.
        app.world_mut().trigger(HealthApplyDamage {
            entity: child,
            source: None,
            amount: 100.0,
        });
        app.world_mut().flush();
        assert_eq!(app.world().get::<Health>(parent).unwrap().current, 100.0);

        // A second hit on the now-dead child leaves the parent untouched.
        app.world_mut().trigger(HealthApplyDamage {
            entity: child,
            source: None,
            amount: 50.0,
        });
        app.world_mut().flush();
        assert_eq!(app.world().get::<Health>(parent).unwrap().current, 100.0);
    }
}
