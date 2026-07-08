//! 15_integrity - a destructible structure demo for the `integrity` module.
//!
//! A grid of blocks, wired together with `ConnectedTo`, floats as one structure (think a
//! space-station panel or an asteroid's shell). CLICK a block to detonate a blast there:
//!
//! - blocks in range take falloff damage and are tinted from intact (blue-grey) to
//!   critical (red) so you can see the wound;
//! - a block at zero health is *disabled*;
//! - a disabled block that is a graph *leaf* (one or zero surviving neighbours) is destroyed
//!   and sliced into flying fragments, then pruned from its neighbours - which can make
//!   *them* leaves, so a fully-disabled patch crumbles from its edges inward over a few
//!   frames. That cascade is the whole point of the `integrity` module.
//!
//! Undamaged blocks outside the blast keep standing, so a single click punches a crumbling
//! hole rather than levelling everything. It wires together four promoted pieces:
//! `integrity` (IntegrityPlugin + `blast_damage` + `ConnectedTo`), the destroy seam
//! (`On<Add, IntegrityDestroyMarker>`) hooked to the existing `ExplodeMeshPlugin`, plus
//! `ui/health_display` (aggregate structure health) and `ui/objectives`.

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
use clap::Parser;

#[derive(Parser)]
#[command(name = "15_integrity")]
#[command(version = "1.0.0")]
#[command(
    about = "Destructible structure demo. Click a block to detonate a blast there.",
    long_about = None
)]
struct Cli;

/// Blocks across and up.
const COLS: usize = 7;
const ROWS: usize = 5;
/// Grid spacing; slightly larger than the 1.0 cube so blocks do not overlap at rest.
const SPACING: f32 = 1.25;

/// Per-block health.
const BLOCK_HEALTH: f32 = 100.0;

/// Blast radius and peak damage. Tuned so the core of the blast *disables* a whole patch
/// (peak damage well above block health, out to a good fraction of the radius), which is
/// what makes the destruction cascade instead of just punching a single hole.
const BLAST_RADIUS: f32 = 2.6;
const BLAST_DAMAGE: f32 = 260.0;

/// Fragment flight after a block is sliced.
const FRAGMENT_SPEED: f32 = 4.5;
const FRAGMENT_LIFETIME: f32 = 3.5;
const FRAGMENT_COUNT: usize = 5;

/// Block tint at full health and at zero health; damage lerps between them.
const HEALTHY_COLOR: Srgba = Srgba::new(0.5, 0.58, 0.72, 1.0);
const CRITICAL_COLOR: Srgba = Srgba::new(0.75, 0.12, 0.1, 1.0);

fn main() {
    let _ = Cli::parse();
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(PhysicsPlugins::default());
    // A structure floating in space: no gravity. The blast is a sensor, so it never pushes
    // the blocks around; only the sliced fragments move, under their own velocity.
    app.insert_resource(Gravity(Vec3::ZERO));

    #[cfg(feature = "debug")]
    app.add_plugins(InspectorDebugPlugin);

    // The promoted pieces plus the two support plugins the destroy seam leans on.
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
            detonate_on_click,
            tint_blocks_by_health,
            aggregate_structure_health,
            update_objectives,
        ),
    );
    app.add_observer(explode_destroyed_block);
    app.add_observer(spawn_fragments);

    app.run();
}

/// A single destructible block of the structure.
#[derive(Component)]
struct Block;

/// The invisible entity the structure's aggregate health is summed onto, for the readout.
#[derive(Component)]
struct StructureCore;

/// Shared assets that outlive `setup`: the fragment look and the blast flash.
#[derive(Resource)]
struct DemoAssets {
    fragment_material: Handle<StandardMaterial>,
    blast_mesh: Handle<Mesh>,
    blast_material: Handle<StandardMaterial>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut objectives: ResMut<GameObjectives>,
) {
    let block_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    // Spawn the blocks, each with its OWN material so it can be tinted by damage
    // independently. Remember each one's grid coordinate so we can connect neighbours.
    let mut grid: Vec<(Entity, usize, usize)> = Vec::new();
    for row in 0..ROWS {
        for col in 0..COLS {
            let x = (col as f32 - (COLS as f32 - 1.0) / 2.0) * SPACING;
            let y = (row as f32 - (ROWS as f32 - 1.0) / 2.0) * SPACING;
            let material = materials.add(StandardMaterial {
                base_color: HEALTHY_COLOR.into(),
                perceptual_roughness: 0.8,
                ..default()
            });
            let entity = commands
                .spawn((
                    Name::new(format!("Block {col},{row}")),
                    Block,
                    Mesh3d(block_mesh.clone()),
                    MeshMaterial3d(material),
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

    commands.insert_resource(DemoAssets {
        fragment_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.45, 0.35),
            ..default()
        }),
        blast_mesh: meshes.add(Sphere::new(BLAST_RADIUS)),
        blast_material: materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.55, 0.2, 0.22),
            emissive: LinearRgba::rgb(1.0, 0.4, 0.1),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
    });

    // An invisible core carrying the structure's aggregate health, tracked by the readout.
    let core = commands
        .spawn((Name::new("StructureCore"), StructureCore, Health::new(1.0)))
        .id();
    commands.spawn(health_display(HealthDisplayConfig { target: Some(core) }));

    commands.spawn(objectives_panel(ObjectivesPanelConfig::default()));
    objectives.objectives = vec![
        Objective::new("how", "Click a block to detonate a blast"),
        Objective::new("goal", "Break the structure apart"),
    ];

    // A fixed camera looking straight down the -Z axis at the structure, so a click maps
    // cleanly onto the z = 0 plane the blocks sit on.
    commands.spawn((
        Name::new("Main Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 16.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 9_000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.5, 0.0)),
    ));
}

/// True when two grid cells are 4-connected neighbours (adjacent in exactly one axis).
fn grid_adjacent(a: (usize, usize), b: (usize, usize)) -> bool {
    let dc = a.0.abs_diff(b.0);
    let dr = a.1.abs_diff(b.1);
    (dc + dr) == 1
}

/// On a left click, cast the cursor onto the z = 0 structure plane and detonate a blast
/// there, with a brief flash so the hit reads.
fn detonate_on_click(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    assets: Res<DemoAssets>,
    q_window: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = q_window.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = q_camera.single() else {
        return;
    };
    let Some(point) = pointer_on_plane(
        camera,
        camera_transform,
        cursor,
        Vec3::ZERO,
        InfinitePlane3d::new(Vec3::Z),
    ) else {
        return;
    };

    info!("detonate: blast at {:?}", point);

    // The damage sensor: exists for one physics step (TempEntity), long enough to fire its
    // overlaps.
    commands.spawn((
        blast_damage(BlastDamageConfig {
            radius: BLAST_RADIUS,
            max_damage: BLAST_DAMAGE,
        }),
        Transform::from_translation(point),
        TempEntity(0.1),
    ));

    // A purely cosmetic flash showing the blast extent.
    commands.spawn((
        Name::new("Blast Flash"),
        Mesh3d(assets.blast_mesh.clone()),
        MeshMaterial3d(assets.blast_material.clone()),
        Transform::from_translation(point),
        TempEntity(0.2),
    ));
}

/// Tint each block from healthy to critical as it loses health, so damage is visible even on
/// blocks that survive the blast.
fn tint_blocks_by_health(
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_blocks: Query<(&Health, &MeshMaterial3d<StandardMaterial>), (With<Block>, Changed<Health>)>,
) {
    for (health, material) in &q_blocks {
        let Some(mut material) = materials.get_mut(&material.0) else {
            continue;
        };
        let frac = (health.current / health.max).clamp(0.0, 1.0);
        material.base_color = CRITICAL_COLOR.mix(&HEALTHY_COLOR, frac).into();
    }
}

/// Keep the structure core's health equal to the sum of the surviving blocks, so the readout
/// tracks real demolition progress. This is the game-side aggregation seam (the integrity
/// core deals in per-node health, not a whole-structure total).
fn aggregate_structure_health(
    mut q_core: Query<&mut Health, With<StructureCore>>,
    q_blocks: Query<&Health, (With<Block>, Without<StructureCore>)>,
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
    // Keep max positive so the readout never divides by zero once the structure is gone.
    core.max = max.max(1.0);
    core.current = current;
}

/// Flip the goal objective to done once every block is gone.
fn update_objectives(mut objectives: ResMut<GameObjectives>, q_blocks: Query<(), With<Block>>) {
    let cleared = q_blocks.is_empty();
    let showing_done = objectives
        .objectives
        .iter()
        .any(|o| o.id == "goal" && o.message.contains("destroyed"));
    if cleared && !showing_done {
        objectives.objectives = vec![Objective::new("goal", "Structure destroyed!")];
    }
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
    // Drop Block so it stops counting toward the structure and cannot be re-processed, and
    // hand it to the slicer.
    commands
        .entity(entity)
        .remove::<Block>()
        .insert(ExplodeMesh {
            fragment_count: FRAGMENT_COUNT,
        });
}

/// When the slicer produces fragments, launch each as a short-lived flying body (real avian
/// motion in zero-g: outward velocity plus a tumble), then despawn the sliced shell.
fn spawn_fragments(
    insert: On<Insert, ExplodeFragments>,
    mut commands: Commands,
    q_fragments: Query<(&ExplodeFragments, &Transform)>,
    assets: Res<DemoAssets>,
) {
    let entity = insert.entity;
    let Ok((fragments, transform)) = q_fragments.get(entity) else {
        return;
    };
    let origin = transform.translation;

    for fragment in fragments.iter() {
        // A deterministic tumble axis from the slice direction (no rng needed).
        let spin = fragment.direction.cross(Vec3::Y).normalize_or(Vec3::X) * 6.0;
        commands.spawn((
            Name::new("Fragment"),
            Mesh3d(fragment.mesh.clone()),
            MeshMaterial3d(assets.fragment_material.clone()),
            Transform::from_translation(origin + fragment.direction * 0.4),
            RigidBody::Dynamic,
            LinearVelocity(fragment.direction * FRAGMENT_SPEED),
            AngularVelocity(spin),
            TempEntity(FRAGMENT_LIFETIME),
        ));
    }

    commands.entity(entity).despawn();
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
