//! [`IntegrityPlugin`]: the destruction pipeline over a graph of connected, health-bearing
//! nodes.
//!
//! The pipeline, all driven by observers over the [`components`](super::components):
//!
//! - collisions deal damage - a fast impact deals impulse/energy damage and a
//!   [`blast`](super::blast) sensor deals radial falloff damage, both funnelling through
//!   [`HealthApplyDamage`];
//! - a node whose health hits zero is disabled (`on_health_depleted_insert_disabled`);
//! - a disabled *leaf* - or a disabled [`IntegrityRoot`] - is destroyed
//!   ([`IntegrityDestroyMarker`], the public seam);
//! - destroying a node prunes it from its neighbours' [`ConnectedTo`] lists, re-deriving leaf
//!   markers and cascading the destruction through the structure.
//!
//! This file owns everything after the damage: disable, destroy, prune, cascade. The
//! collision-to-damage half lives in the private `damage` sibling module.
//!
//! The plugin never decides what "destroyed" looks like: it inserts
//! [`IntegrityDestroyMarker`] and prunes the graph, and the game observes that marker to
//! explode, spawn debris, or despawn. Building the graph ([`ConnectedTo`]/[`IntegrityRoot`])
//! is also the game's job. See `examples/15_integrity.rs`.

use bevy::prelude::*;

use super::{components::*, damage::*};
use crate::health::prelude::*;

pub mod prelude {
    pub use super::{IntegrityPlugin, IntegritySystems};
}

/// System set for the leaf-derivation system, so games can order graph edits around it.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntegritySystems;

/// Plugin wiring the destruction pipeline. Requires [`HealthPlugin`]
/// and avian's `PhysicsPlugins` for the collision-driven damage.
pub struct IntegrityPlugin;

impl Plugin for IntegrityPlugin {
    fn build(&self, app: &mut App) {
        debug!("IntegrityPlugin: build");

        app.add_observer(on_collider_of_spawn_insert_collision_events);
        app.add_observer(on_impact_collision_deal_damage);
        app.add_observer(on_blast_collision_deal_damage);
        app.add_observer(on_health_depleted_insert_disabled);
        app.add_observer(handle_destroy);
        app.add_observer(handle_chain_destroy);
        app.add_observer(handle_parent_destroy);
        app.add_observer(on_destroyed);

        app.add_systems(Update, derive_integrity_leaves.in_set(IntegritySystems));
    }
}

/// Disable a node the moment its health reaches zero.
fn on_health_depleted_insert_disabled(add: On<Add, HealthZeroMarker>, mut commands: Commands) {
    let entity = add.entity;
    trace!(
        "on_health_depleted_disable: entity {:?} health depleted, disabling",
        entity
    );

    commands.entity(entity).insert(IntegrityDisabledMarker);
}

/// Destroy a node that is disabled while already a leaf.
fn handle_destroy(
    add: On<Add, IntegrityDisabledMarker>,
    mut commands: Commands,
    q_disabled: Query<(), (With<IntegrityDisabledMarker>, With<IntegrityLeafMarker>)>,
) {
    let entity = add.entity;
    trace!("handle_destroy: entity {:?}", entity);

    let Ok(_) = q_disabled.get(entity) else {
        return;
    };

    debug!("handle_destroy: entity {:?} will be destroyed", entity);
    commands.entity(entity).insert(IntegrityDestroyMarker);
}

/// Destroy an already-disabled node that has just *become* a leaf - the chain reaction.
fn handle_chain_destroy(
    add: On<Add, IntegrityLeafMarker>,
    mut commands: Commands,
    q_destroyed: Query<(), (With<IntegrityDisabledMarker>, With<IntegrityLeafMarker>)>,
) {
    let entity = add.entity;
    trace!("handle_chain_destroy: entity {:?}", entity);

    let Ok(_) = q_destroyed.get(entity) else {
        return;
    };

    debug!(
        "handle_chain_destroy: entity {:?} became a disabled leaf, destroying",
        entity
    );
    commands.entity(entity).insert(IntegrityDestroyMarker);
}

/// Destroy a disabled [`IntegrityRoot`] outright: the whole structure dies with its root,
/// leaf or not.
fn handle_parent_destroy(
    add: On<Add, IntegrityDisabledMarker>,
    mut commands: Commands,
    q_destroyed: Query<(), (With<IntegrityDisabledMarker>, With<IntegrityRoot>)>,
) {
    let entity = add.entity;
    trace!("handle_parent_destroy: entity {:?}", entity);

    let Ok(_) = q_destroyed.get(entity) else {
        return;
    };

    commands.entity(entity).insert(IntegrityDestroyMarker);
}

/// When a node is destroyed, prune it from its neighbours' [`ConnectedTo`] lists. Mutating a
/// neighbour's list marks it `Changed`, so `derive_integrity_leaves` re-evaluates whether the
/// neighbour has become a leaf (which, if it is also disabled, drives the chain reaction via
/// `handle_chain_destroy`).
///
/// The destroyed node carries `IntegrityDestroyMarker`; its neighbours do not (a neighbour
/// that happens to be destroyed the same frame is skipped, which is harmless - it is going
/// away anyway). The disjoint `With`/`Without` filters keep the two `ConnectedTo` accesses
/// sound.
fn on_destroyed(
    add: On<Add, IntegrityDestroyMarker>,
    q_destroyed: Query<&ConnectedTo, With<IntegrityDestroyMarker>>,
    mut q_neighbors: Query<&mut ConnectedTo, Without<IntegrityDestroyMarker>>,
) {
    let entity = add.entity;
    trace!("on_destroyed: entity {:?}", entity);

    let Ok(connected) = q_destroyed.get(entity) else {
        return;
    };

    let neighbors = connected.0.clone();
    for neighbor in neighbors {
        if let Ok(mut neighbor_connections) = q_neighbors.get_mut(neighbor) {
            neighbor_connections.retain(|&node| node != entity);
        }
    }
}

/// Re-derive leaf markers whenever a node's [`ConnectedTo`] changes (on initial build, or
/// when a neighbour is pruned by `on_destroyed`). A node with one or zero neighbours is a leaf.
fn derive_integrity_leaves(
    mut commands: Commands,
    q_nodes: Query<(Entity, &ConnectedTo), Changed<ConnectedTo>>,
) {
    for (entity, connected) in &q_nodes {
        if connected.len() <= 1 {
            commands.entity(entity).try_insert(IntegrityLeafMarker);
        } else {
            commands.entity(entity).try_remove::<IntegrityLeafMarker>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal app wired with the avian-free core of the integrity pipeline plus
    /// the health machinery, so tests can drive it from real damage.
    fn integrity_core_app() -> App {
        let mut app = App::new();
        app.add_plugins(HealthPlugin);
        app.add_observer(on_health_depleted_insert_disabled);
        app.add_observer(handle_destroy);
        app.add_observer(handle_chain_destroy);
        app.add_observer(handle_parent_destroy);
        app.add_observer(on_destroyed);
        app.add_systems(Update, derive_integrity_leaves);
        app
    }

    #[test]
    fn leaves_are_derived_from_the_connection_count() {
        let mut app = App::new();
        app.add_systems(Update, derive_integrity_leaves);

        let a = app.world_mut().spawn_empty().id();
        let b = app.world_mut().spawn_empty().id();
        let leaf = app.world_mut().spawn(ConnectedTo(vec![a])).id(); // 1 neighbor
        let hub = app.world_mut().spawn(ConnectedTo(vec![a, b])).id(); // 2 neighbors

        app.update();

        assert!(app.world().get::<IntegrityLeafMarker>(leaf).is_some());
        assert!(app.world().get::<IntegrityLeafMarker>(hub).is_none());

        app.world_mut().get_mut::<ConnectedTo>(hub).unwrap().0 = vec![a]; // now 1 neighbor
        app.update();
        assert!(app.world().get::<IntegrityLeafMarker>(hub).is_some());
    }

    #[test]
    fn a_disabled_leaf_is_marked_for_destruction() {
        let mut app = integrity_core_app();
        let node = app.world_mut().spawn(IntegrityLeafMarker).id();

        app.world_mut()
            .entity_mut(node)
            .insert(IntegrityDisabledMarker);
        app.update();

        assert!(app.world().get::<IntegrityDestroyMarker>(node).is_some());
    }

    #[test]
    fn a_disabled_non_leaf_is_not_destroyed() {
        let mut app = integrity_core_app();
        let node = app.world_mut().spawn_empty().id();

        app.world_mut()
            .entity_mut(node)
            .insert(IntegrityDisabledMarker);
        app.update();

        assert!(app.world().get::<IntegrityDestroyMarker>(node).is_none());
    }

    #[test]
    fn becoming_a_leaf_while_disabled_triggers_destruction() {
        let mut app = integrity_core_app();
        let node = app.world_mut().spawn(IntegrityDisabledMarker).id();

        app.world_mut().entity_mut(node).insert(IntegrityLeafMarker);
        app.update();

        assert!(app.world().get::<IntegrityDestroyMarker>(node).is_some());
    }

    #[test]
    fn a_disabled_root_is_destroyed_whole() {
        let mut app = integrity_core_app();
        let root = app.world_mut().spawn(IntegrityRoot).id();

        app.world_mut()
            .entity_mut(root)
            .insert(IntegrityDisabledMarker);
        app.update();

        assert!(app.world().get::<IntegrityDestroyMarker>(root).is_some());
    }

    #[test]
    fn damage_drives_a_leaf_from_full_health_to_destruction() {
        let mut app = integrity_core_app();
        let node = app
            .world_mut()
            .spawn((Health::new(50.0), ConnectedTo(vec![])))
            .id();

        app.update(); // no neighbors -> leaf
        assert!(app.world().get::<IntegrityLeafMarker>(node).is_some());
        assert!(app.world().get::<IntegrityDisabledMarker>(node).is_none());

        app.world_mut().trigger(HealthApplyDamage {
            entity: node,
            source: None,
            amount: 60.0,
        });
        app.update();

        assert!(app.world().get::<HealthZeroMarker>(node).is_some());
        assert!(app.world().get::<IntegrityDisabledMarker>(node).is_some());
        assert!(app.world().get::<IntegrityDestroyMarker>(node).is_some());
    }

    #[test]
    fn destruction_chains_through_a_connected_structure() {
        // NOTE: a disabled line A-B-C. A and C are leaves and go first; pruning them from B
        // leaves B a leaf too, so the whole structure comes apart from the ends in.
        let mut app = integrity_core_app();

        let a = app.world_mut().spawn(IntegrityDisabledMarker).id();
        let b = app.world_mut().spawn(IntegrityDisabledMarker).id();
        let c = app.world_mut().spawn(IntegrityDisabledMarker).id();
        app.world_mut().entity_mut(a).insert(ConnectedTo(vec![b]));
        app.world_mut()
            .entity_mut(b)
            .insert(ConnectedTo(vec![a, c]));
        app.world_mut().entity_mut(c).insert(ConnectedTo(vec![b]));

        for _ in 0..5 {
            app.update(); // let the leaf derivation and chain reaction settle
        }

        for node in [a, b, c] {
            assert!(
                app.world().get::<IntegrityDestroyMarker>(node).is_some(),
                "node {node:?} should have been destroyed in the chain"
            );
        }
    }
}
