//! Components describing a destructible structure as a graph of connected nodes.
//!
//! A structure is a set of node entities, each carrying [`ConnectedTo`] (its structural
//! neighbours), all descending from a single body marked [`IntegrityRoot`]. The plugin in
//! [`super::plugin`] drives the destruction lifecycle over these components:
//! health depletion inserts [`IntegrityDisabledMarker`], a disabled leaf (or a disabled
//! root) inserts [`IntegrityDestroyMarker`], and destroying a node prunes it from its
//! neighbours, which can turn them into leaves and cascade.
//!
//! The graph is deliberately node-local (each node owns its own neighbour list) rather than
//! a central adjacency map on the root, so a game builds it however it likes - a grid, a
//! hand-authored list, a single lone node - without this module knowing the layout.

use bevy::prelude::*;

pub mod prelude {
    pub use super::{
        ConnectedTo, IntegrityDestroyMarker, IntegrityDisabledMarker, IntegrityLeafMarker,
        IntegrityRoot,
    };
}

/// Marks the body that owns an integrity structure (the root of a multi-node structure, or a
/// lone body such as a boulder). Its integrity nodes are its descendants that carry
/// [`ConnectedTo`].
///
/// The root just needs to be identifiable so the plugin can find it for aggregate concerns
/// (whole-body destruction when the root is disabled). Games typically add it to the
/// `RigidBody` entity whose colliders are the nodes.
#[derive(Component, Debug, Default, Reflect)]
pub struct IntegrityRoot;

/// The integrity neighbours of a node, i.e. the adjacent nodes it is structurally connected
/// to. Lives on the node itself (a collider, a structural cell) rather than in a central
/// graph.
///
/// A node with one or zero neighbours is a leaf ([`IntegrityLeafMarker`]); removing a node
/// prunes it from its neighbours' lists, which can turn them into leaves and drive the
/// destruction chain reaction. A lone node gets an empty list, so it is a leaf and is
/// destroyed as soon as it is disabled.
#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct ConnectedTo(pub Vec<Entity>);

/// Marker component for leaf nodes in the integrity graph (one or zero neighbours).
///
/// Derived automatically from [`ConnectedTo`] by the plugin. When a node that is disabled
/// becomes a leaf, it is destroyed (the chain reaction).
#[derive(Component, Debug, Default, Reflect)]
pub struct IntegrityLeafMarker;

/// Marker component for nodes that are disabled due to having zero health.
///
/// Inserted automatically when a node gains [`HealthZeroMarker`](crate::health::HealthZeroMarker).
/// A disabled interior node is merely deactivated; a disabled *leaf* is destroyed.
#[derive(Component, Debug, Default, Reflect)]
pub struct IntegrityDisabledMarker;

/// Marker component inserted on a node the frame it is destroyed.
///
/// This is the public seam of the destruction pipeline: a game reacts to
/// `On<Add, IntegrityDestroyMarker>` to play effects, spawn debris, slice the mesh (see the
/// integrity example, which hooks it to [`ExplodeMesh`](crate::mesh::explode::ExplodeMesh)),
/// or despawn the entity. The plugin itself only inserts the marker and prunes the graph; it
/// never decides what "destroyed" looks like.
#[derive(Component, Debug, Default, Reflect)]
pub struct IntegrityDestroyMarker;
