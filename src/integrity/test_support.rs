//! Shared harness for the physics-level integrity tests in `plugin.rs`.
//!
//! The avian-free core of the pipeline is unit-tested directly in that module (driving
//! `ConnectedTo` / `HealthApplyDamage` by hand). These helpers cover the other half: the
//! physics-driven *inputs* - collision and blast damage - which need a real avian world to
//! produce `ColliderOf` links and `ComputedMass`.

use core::time::Duration;

use avian3d::prelude::*;
use bevy::{prelude::*, time::TimeUpdateStrategy};

use super::plugin::IntegrityPlugin;
use crate::health::prelude::*;

/// A headless avian app wired with the full integrity pipeline.
///
/// Mirrors avian's own test harness (`MinimalPlugins` + `TransformPlugin` + `AssetPlugin` +
/// `MeshPlugin` + `PhysicsPlugins`); `MeshPlugin` is required because avian's
/// `collider-from-mesh` feature reads `AssetEvent<Mesh>` and panics on a
/// `Messages<AssetEvent<Mesh>>` that was never initialized. A fixed manual timestep makes
/// stepping deterministic, and gravity is zeroed so a body stays exactly where the test puts
/// it.
pub(crate) fn integrity_physics_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        bevy::asset::AssetPlugin::default(),
        bevy::mesh::MeshPlugin,
        PhysicsPlugins::default(),
        HealthPlugin,
        IntegrityPlugin,
    ));
    app.insert_resource(Gravity(Vec3::ZERO));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
        1.0 / 60.0,
    )));
    app.finish();
    app
}

/// Step the app enough times for avian to link colliders (`ColliderOf`) and finalize masses
/// (`ComputedMass`). A single update is not enough - mass is computed over the first few
/// steps, and reading it too early yields `NaN`.
pub(crate) fn settle(app: &mut App) {
    for _ in 0..4 {
        app.update();
    }
}
