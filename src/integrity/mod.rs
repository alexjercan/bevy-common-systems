//! Integrity: a destruction pipeline for structures built from connected, health-bearing
//! nodes.
//!
//! Model a destructible object as a graph: each node is a health-bearing collider carrying
//! [`ConnectedTo`](components::ConnectedTo) (its structural neighbours), all under one body
//! marked [`IntegrityRoot`](components::IntegrityRoot). [`IntegrityPlugin`](plugin::IntegrityPlugin)
//! then turns collisions and [`blast`] volumes into damage, disables nodes at zero health,
//! destroys disabled leaves, and cascades the destruction as nodes are pruned from the graph.
//!
//! The game owns the two ends: it builds the graph (how nodes connect) and it decides what a
//! destroyed node does, by observing `On<Add, IntegrityDestroyMarker>`. Everything in between
//! is this module. See `examples/15_integrity.rs`, which builds a grid, damages it with a
//! blast, and hooks the destroy marker to the mesh slicer.

#[cfg(test)]
mod test_support;

pub mod blast;
pub mod components;
pub mod plugin;

pub use plugin::IntegrityPlugin;

pub mod prelude {
    pub use super::{blast::prelude::*, components::prelude::*, plugin::prelude::*};
}
