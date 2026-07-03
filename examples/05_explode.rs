use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
use clap::Parser;

#[derive(Parser)]
#[command(name = "05_explode")]
#[command(version = "1.0.0")]
#[command(
    about = "Slice a mesh into fragments with the explode plugin. Press Left Mouse Button to explode.",
    long_about = None
)]
struct Cli;

/// Speed, in units per second, that fragments fly away from the origin.
const FRAGMENT_SPEED: f32 = 6.0;

/// How long a fragment lives before it despawns, in seconds.
const FRAGMENT_LIFETIME: f32 = 4.0;

/// Number of fragments to request per explosion.
const FRAGMENT_COUNT: usize = 12;

fn main() {
    let _ = Cli::parse();
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(PhysicsPlugins::default());

    #[cfg(feature = "debug")]
    app.add_plugins(InspectorDebugPlugin);

    app.add_plugins(bevy_enhanced_input::EnhancedInputPlugin);
    app.add_plugins(WASDCameraPlugin);
    app.add_plugins(WASDCameraControllerPlugin);

    // ExplodeMeshPlugin does the slicing; TempEntityPlugin cleans up the
    // fragments after their lifetime so repeated explosions do not pile up.
    app.add_plugins(ExplodeMeshPlugin);
    app.add_plugins(TempEntityPlugin);

    app.add_systems(Startup, setup);
    app.add_systems(Update, (explode_on_lmb, move_fragments));
    app.add_observer(on_fragments_spawned);

    app.run();
}

/// Marker for the current intact target that Left Mouse Button will explode.
#[derive(Component)]
struct Target;

/// Velocity carried by a flying fragment.
#[derive(Component)]
struct FragmentMotion {
    velocity: Vec3,
}

/// Shared render assets so every target and fragment looks the same.
#[derive(Resource)]
struct ExplodeAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // A sphere-ish octahedron centered at the origin: any plane through the
    // origin cuts it, so it always explodes.
    let mesh = meshes.add(TriangleMeshBuilder::new_octahedron(3).build());
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.7, 0.6),
        ..default()
    });

    commands.insert_resource(ExplodeAssets {
        mesh: mesh.clone(),
        material: material.clone(),
    });

    spawn_target(&mut commands, &mesh, &material);

    commands.spawn((
        Name::new("Main Camera"),
        Camera3d::default(),
        WASDCameraController,
        Transform::from_xyz(0.0, 4.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_2,
            0.0,
            0.0,
        )),
        GlobalTransform::default(),
    ));
}

/// Spawn a fresh intact target at the origin.
fn spawn_target(commands: &mut Commands, mesh: &Handle<Mesh>, material: &Handle<StandardMaterial>) {
    commands.spawn((
        Name::new("Target"),
        Target,
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

/// On Left Mouse Button, explode the current target by inserting `ExplodeMesh`.
fn explode_on_lmb(
    mut commands: Commands,
    input: Res<ButtonInput<MouseButton>>,
    q_target: Query<Entity, With<Target>>,
) {
    if !input.just_pressed(MouseButton::Left) {
        return;
    }

    for entity in q_target.iter() {
        // Drop the Target marker so the same entity cannot be exploded twice
        // while the fragments are being generated.
        commands
            .entity(entity)
            .remove::<Target>()
            .insert(ExplodeMesh {
                fragment_count: FRAGMENT_COUNT,
            });
    }
}

/// When the explode plugin produces `ExplodeFragments`, spawn each fragment as
/// a flying entity, remove the original, and queue up a fresh target.
fn on_fragments_spawned(
    insert: On<Insert, ExplodeFragments>,
    mut commands: Commands,
    q_fragments: Query<(&ExplodeFragments, &Transform)>,
    assets: Res<ExplodeAssets>,
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
            MeshMaterial3d(assets.material.clone()),
            Transform::from_translation(origin),
            FragmentMotion {
                // Push outward along the slice direction, with a little lift.
                velocity: fragment.direction * FRAGMENT_SPEED + Vec3::Y * 2.0,
            },
            TempEntity(FRAGMENT_LIFETIME),
        ));
    }

    // Remove the exploded shell and spawn a new intact target to explode next.
    commands.entity(entity).despawn();
    spawn_target(&mut commands, &assets.mesh, &assets.material);
}

/// Move fragments along their velocity under a little gravity, and tumble them.
fn move_fragments(time: Res<Time>, mut q_fragments: Query<(&mut Transform, &mut FragmentMotion)>) {
    let dt = time.delta_secs();

    for (mut transform, mut motion) in q_fragments.iter_mut() {
        motion.velocity.y -= 9.8 * dt;
        transform.translation += motion.velocity * dt;
        transform.rotate_local_x(dt * 3.0);
        transform.rotate_local_y(dt * 2.0);
    }
}
