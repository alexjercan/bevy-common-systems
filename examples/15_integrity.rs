//! 15_integrity - a destructible structure demo for the `integrity` module.
//!
//! A grid of connected blocks forms a wall. Press Space (or Left Mouse Button) to detonate a
//! blast at the centre of the surviving blocks: everything in range takes falloff damage,
//! blocks that hit zero health are disabled, and disabled *leaf* blocks are destroyed -
//! pruning them from their neighbours and cascading the collapse through the structure.
//!
//! This wires four promoted pieces together:
//! - `integrity` (IntegrityPlugin + `blast_damage` + `ConnectedTo`) drives the destruction;
//! - the destroy seam (`On<Add, IntegrityDestroyMarker>`) is hooked to the existing
//!   `ExplodeMeshPlugin`, so a destroyed block slices its own mesh into flying fragments;
//! - `ui/health_display` shows the wall's aggregate health (summed onto a core entity);
//! - `ui/objectives` shows a single objective that flips to done when the wall is gone.

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
use clap::Parser;

#[derive(Parser)]
#[command(name = "15_integrity")]
#[command(version = "1.0.0")]
#[command(
    about = "Destructible structure demo. Press Space / Left Mouse Button to detonate a blast.",
    long_about = None
)]
struct Cli;

/// Number of blocks across and up.
const COLS: usize = 6;
const ROWS: usize = 4;
/// Grid spacing; slightly larger than the 1.0 cube so blocks do not overlap at rest.
const SPACING: f32 = 1.3;

/// Blast radius and peak damage per detonation.
const BLAST_RADIUS: f32 = 3.2;
const BLAST_DAMAGE: f32 = 140.0;

/// Per-block health.
const BLOCK_HEALTH: f32 = 100.0;

/// Fragment flight after a block is sliced.
const FRAGMENT_SPEED: f32 = 5.0;
const FRAGMENT_LIFETIME: f32 = 3.0;
const FRAGMENT_COUNT: usize = 4;

fn main() {
    let _ = Cli::parse();
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(PhysicsPlugins::default());
    // The structure floats in place: no gravity, and the blast is a sensor (no push).
    app.insert_resource(Gravity(Vec3::ZERO));

    #[cfg(feature = "debug")]
    app.add_plugins(InspectorDebugPlugin);

    app.add_plugins(bevy_enhanced_input::EnhancedInputPlugin);
    app.add_plugins(WASDCameraPlugin);
    app.add_plugins(WASDCameraControllerPlugin);

    // The promoted pieces plus the two support plugins the seam leans on.
    app.add_plugins(HealthPlugin);
    app.add_plugins(IntegrityPlugin);
    app.add_plugins(ExplodeMeshPlugin);
    app.add_plugins(TempEntityPlugin);
    app.add_plugins(HealthDisplayPlugin);
    app.add_plugins(ObjectivesPlugin);

    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            detonate_on_input,
            aggregate_wall_health,
            update_objectives,
            move_fragments,
        ),
    );
    app.add_observer(darken_disabled_block);
    app.add_observer(explode_destroyed_block);
    app.add_observer(spawn_fragments);

    app.run();
}

/// A single destructible block of the wall.
#[derive(Component)]
struct Block;

/// The invisible core the wall's aggregate health is summed onto, for the health display.
#[derive(Component)]
struct WallCore;

/// Velocity carried by a flying fragment.
#[derive(Component)]
struct FragmentMotion {
    velocity: Vec3,
}

/// Shared render assets so blocks and fragments look consistent.
#[derive(Resource)]
struct WallAssets {
    block_mesh: Handle<Mesh>,
    block_material: Handle<StandardMaterial>,
    disabled_material: Handle<StandardMaterial>,
    fragment_material: Handle<StandardMaterial>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut objectives: ResMut<GameObjectives>,
) {
    let block_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let block_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.6, 0.7),
        perceptual_roughness: 0.8,
        ..default()
    });
    let disabled_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.15, 0.17),
        perceptual_roughness: 0.95,
        ..default()
    });
    let fragment_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.4, 0.3),
        ..default()
    });

    // Spawn the blocks, remembering each one's grid coordinate so we can connect neighbours.
    let mut grid: Vec<(Entity, usize, usize)> = Vec::new();
    for row in 0..ROWS {
        for col in 0..COLS {
            let x = (col as f32 - (COLS as f32 - 1.0) / 2.0) * SPACING;
            let y = (row as f32 - (ROWS as f32 - 1.0) / 2.0) * SPACING + 2.0;
            let entity = commands
                .spawn((
                    Name::new(format!("Block {col},{row}")),
                    Block,
                    Mesh3d(block_mesh.clone()),
                    MeshMaterial3d(block_material.clone()),
                    Transform::from_xyz(x, y, 0.0),
                    // A destructible physics body: Health + density, visible.
                    destructible_body(BLOCK_HEALTH, 1.0),
                    RigidBody::Dynamic,
                    Collider::cuboid(1.0, 1.0, 1.0),
                    // Own our collision events so the blast sensor always reaches us.
                    CollisionEventsEnabled,
                ))
                .id();
            grid.push((entity, col, row));
        }
    }

    // Connect 4-neighbours in the grid: each block's ConnectedTo lists the adjacent blocks.
    // A block with <= 1 surviving neighbour is a leaf, so the collapse eats inward from the
    // edges of the blasted hole.
    for &(entity, col, row) in &grid {
        let neighbors: Vec<Entity> = grid
            .iter()
            .filter(|&&(_, c, r)| grid_adjacent((col, row), (c, r)))
            .map(|&(other, _, _)| other)
            .collect();
        commands.entity(entity).insert(ConnectedTo(neighbors));
    }

    commands.insert_resource(WallAssets {
        block_mesh,
        block_material,
        disabled_material,
        fragment_material,
    });

    // An invisible core carrying the wall's aggregate health, tracked by the health display.
    let core = commands
        .spawn((Name::new("WallCore"), WallCore, Health::new(1.0)))
        .id();
    commands.spawn(health_display(HealthDisplayConfig { target: Some(core) }));

    commands.spawn(objectives_panel(ObjectivesPanelConfig::default()));
    objectives.objectives = vec![Objective::new(
        "demolish",
        "Demolish the structure (Space to detonate)",
    )];

    commands.spawn((
        Name::new("Main Camera"),
        Camera3d::default(),
        WASDCameraController,
        Transform::from_xyz(0.0, 2.0, 14.0).looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, 0.6, 0.0)),
    ));
}

/// True when two grid cells are 4-connected neighbours (adjacent in exactly one axis).
fn grid_adjacent(a: (usize, usize), b: (usize, usize)) -> bool {
    let dc = a.0.abs_diff(b.0);
    let dr = a.1.abs_diff(b.1);
    (dc + dr) == 1
}

/// On input, detonate a blast at the centroid of the surviving blocks.
fn detonate_on_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    q_blocks: Query<&Transform, With<Block>>,
) {
    if !(keys.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left)) {
        return;
    }

    let mut centroid = Vec3::ZERO;
    let mut count = 0.0;
    for transform in &q_blocks {
        centroid += transform.translation;
        count += 1.0;
    }
    if count == 0.0 {
        return;
    }
    centroid /= count;

    info!("detonate: blast at {:?}", centroid);
    commands.spawn((
        blast_damage(BlastDamageConfig {
            radius: BLAST_RADIUS,
            max_damage: BLAST_DAMAGE,
        }),
        Transform::from_translation(centroid),
        // The sensor only needs to exist for one physics step to fire its overlaps.
        TempEntity(0.1),
    ));
}

/// Keep the wall core's health equal to the sum of the surviving blocks, so the health
/// display tracks real demolition progress. This is the game-side aggregation seam (the
/// integrity core deals in per-node health, not a whole-structure total).
fn aggregate_wall_health(
    mut q_core: Query<&mut Health, With<WallCore>>,
    q_blocks: Query<&Health, (With<Block>, Without<WallCore>)>,
) {
    let Ok(mut core) = q_core.single_mut() else {
        return;
    };
    let mut current = 0.0;
    let mut max = 0.0;
    for health in &q_blocks {
        current += health.current;
        max += health.max;
    }
    // Keep max positive so the display never divides by zero once the wall is gone.
    core.max = max.max(1.0);
    core.current = current;
}

/// Flip the objective to done once every block is gone.
fn update_objectives(mut objectives: ResMut<GameObjectives>, q_blocks: Query<(), With<Block>>) {
    let cleared = q_blocks.is_empty();
    let showing_done = objectives
        .objectives
        .first()
        .is_some_and(|o| o.id == "cleared");
    if cleared && !showing_done {
        objectives.objectives = vec![Objective::new("cleared", "Structure demolished!")];
    }
}

/// A disabled-but-not-yet-destroyed block is deactivated: darken it so the dead-but-standing
/// blocks read differently from the live ones. (The destroyed ones are handled by the slicer.)
fn darken_disabled_block(
    add: On<Add, IntegrityDisabledMarker>,
    mut commands: Commands,
    assets: Res<WallAssets>,
    q_block: Query<(), (With<Block>, With<IntegrityDisabledMarker>)>,
) {
    let entity = add.entity;
    if !q_block.contains(entity) {
        return;
    }
    commands
        .entity(entity)
        .insert(MeshMaterial3d(assets.disabled_material.clone()));
}

/// The destroy seam: when the integrity core marks a block destroyed, slice its mesh with the
/// existing ExplodeMeshPlugin. The block has a real Mesh3d, so the slicer can fragment it.
fn explode_destroyed_block(
    add: On<Add, IntegrityDestroyMarker>,
    mut commands: Commands,
    q_block: Query<(), (With<Block>, With<Mesh3d>)>,
) {
    let entity = add.entity;
    if !q_block.contains(entity) {
        return;
    }
    // Drop Block so it stops counting toward the wall and cannot be re-processed, and hand
    // it to the slicer.
    commands
        .entity(entity)
        .remove::<Block>()
        .insert(ExplodeMesh {
            fragment_count: FRAGMENT_COUNT,
        });
}

/// When the slicer produces fragments, launch each as a short-lived flying body, then despawn
/// the sliced shell.
fn spawn_fragments(
    insert: On<Insert, ExplodeFragments>,
    mut commands: Commands,
    q_fragments: Query<(&ExplodeFragments, &Transform)>,
    assets: Res<WallAssets>,
) {
    let entity = insert.entity;
    let Ok((fragments, transform)) = q_fragments.get(entity) else {
        return;
    };
    let origin = transform.translation;

    for fragment in fragments.iter() {
        commands.spawn((
            Name::new("Fragment"),
            Mesh3d(fragment.mesh.clone()),
            MeshMaterial3d(assets.fragment_material.clone()),
            Transform::from_translation(origin),
            FragmentMotion {
                velocity: fragment.direction * FRAGMENT_SPEED + Vec3::Y * 1.5,
            },
            TempEntity(FRAGMENT_LIFETIME),
        ));
    }

    commands.entity(entity).despawn();
}

/// Drift and tumble fragments; TempEntity despawns them after their lifetime.
fn move_fragments(time: Res<Time>, mut q_fragments: Query<(&mut Transform, &mut FragmentMotion)>) {
    let dt = time.delta_secs();
    for (mut transform, mut motion) in &mut q_fragments {
        motion.velocity.y -= 4.0 * dt;
        transform.translation += motion.velocity * dt;
        transform.rotate_local_x(dt * 3.0);
        transform.rotate_local_y(dt * 2.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_adjacency_is_four_connected() {
        // Orthogonal neighbours are adjacent...
        assert!(grid_adjacent((1, 1), (1, 2)));
        assert!(grid_adjacent((1, 1), (2, 1)));
        // ...diagonals and self and gaps are not.
        assert!(!grid_adjacent((1, 1), (2, 2)));
        assert!(!grid_adjacent((1, 1), (1, 1)));
        assert!(!grid_adjacent((1, 1), (1, 3)));
    }
}
