//! A tiny "fruit ninja" style game built entirely from procedural shapes.
//!
//! Octahedron "fruits" are launched up in a parabolic arc from below the view.
//! Hold the Left Mouse Button and swipe the cursor across a fruit to slice it:
//! the mesh is cut into flying fragments by `ExplodeMeshPlugin` and your score
//! goes up. Fruit you miss falls back off the bottom of the screen.
//!
//! Everything here is plain shapes and hand-rolled kinematics: no assets, no
//! physics engine. It reuses the crate's `TriangleMeshBuilder` (meshes),
//! `ExplodeMeshPlugin` (the slice effect), `TempEntityPlugin` (fragment
//! cleanup) and `StatusBarPlugin` (the score / FPS overlay).

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
use clap::Parser;
use rand::Rng;

#[derive(Parser)]
#[command(name = "06_fruitninja")]
#[command(version = "1.0.0")]
#[command(
    about = "Slice launched shapes with the mouse. Hold Left Mouse Button and swipe across a fruit to slice it.",
    long_about = None
)]
struct Cli;

/// Downward acceleration applied to fruit and fragments, units per second^2.
const GRAVITY: f32 = 18.0;

/// Z plane the whole game plays on; the static camera looks straight at it.
const PLAY_Z: f32 = 0.0;

/// Y below which a fruit counts as missed and is despawned.
const KILL_Y: f32 = -12.0;

/// Y a fruit is launched from, just under the visible area.
const SPAWN_Y: f32 = -10.0;

/// Radius of a fruit, in world units. The octahedron sphere is built at unit
/// radius, so this doubles as the slice hit radius.
const FRUIT_RADIUS: f32 = 1.0;

/// Seconds between fruit launches.
const SPAWN_INTERVAL: f32 = 0.9;

/// Fragments requested per slice.
const FRAGMENT_COUNT: usize = 10;

/// Speed fragments fly away from the slice point, units per second.
const FRAGMENT_SPEED: f32 = 5.0;

/// How long a fragment lives before it despawns, in seconds.
const FRAGMENT_LIFETIME: f32 = 3.0;

fn main() {
    let _ = Cli::parse();
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);

    // The debug inspector pulls in avian's debug-render systems, which need the
    // resources PhysicsPlugins installs; add it so `--features debug` boots even
    // though the game itself does no physics simulation.
    app.add_plugins(PhysicsPlugins::default());

    #[cfg(feature = "debug")]
    app.add_plugins(InspectorDebugPlugin);

    // FrameTimeDiagnosticsPlugin feeds the status bar's FPS item.
    if !app.is_plugin_added::<bevy::diagnostic::FrameTimeDiagnosticsPlugin>() {
        app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
    }

    app.add_plugins(ExplodeMeshPlugin);
    app.add_plugins(TempEntityPlugin);
    app.add_plugins(StatusBarPlugin);

    app.init_resource::<Score>();
    app.insert_resource(SpawnTimer(Timer::from_seconds(
        SPAWN_INTERVAL,
        TimerMode::Repeating,
    )));
    app.init_resource::<CursorTrail>();

    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            spawn_fruit,
            move_fruit,
            slice_fruit,
            move_fragments,
            update_score_text,
        ),
    );
    app.add_observer(on_fragments_spawned);

    app.run();
}

/// Running number of fruits sliced. Shown in the status bar.
#[derive(Resource, Default, Deref, DerefMut)]
struct Score(usize);

/// Ticks between fruit launches.
#[derive(Resource, Deref, DerefMut)]
struct SpawnTimer(Timer);

/// The cursor position on the play plane, remembered across frames so a slice
/// can be tested against the swipe segment (previous -> current), not a point.
#[derive(Resource, Default)]
struct CursorTrail {
    /// Cursor world position on the play plane last frame, if it was on screen.
    previous: Option<Vec3>,
}

/// Marker for the on-screen score HUD text.
#[derive(Component)]
struct ScoreText;

/// A slice-able fruit flying through the scene.
#[derive(Component)]
struct Fruit {
    /// Slice hit radius in world units.
    radius: f32,
}

/// Velocity carried by a flying fruit.
#[derive(Component)]
struct FruitMotion {
    velocity: Vec3,
}

/// Velocity carried by a flying fragment of a sliced fruit.
#[derive(Component)]
struct FragmentMotion {
    velocity: Vec3,
}

/// Shared render assets so fruit and fragments are cheap to spawn.
#[derive(Resource)]
struct FruitAssets {
    mesh: Handle<Mesh>,
    materials: Vec<Handle<StandardMaterial>>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // One centered octahedron sphere, reused by every fruit and every fragment.
    // Centered on the origin means any slice plane through the local origin
    // cuts it, so it always explodes.
    let mesh = meshes.add(TriangleMeshBuilder::new_octahedron(3).build());

    // A handful of "fruit" colors picked at spawn time.
    let palette = [
        Color::srgb(0.85, 0.20, 0.20),
        Color::srgb(0.95, 0.65, 0.15),
        Color::srgb(0.30, 0.75, 0.35),
        Color::srgb(0.55, 0.35, 0.80),
        Color::srgb(0.95, 0.85, 0.25),
    ];
    let materials = palette
        .into_iter()
        .map(|color| {
            materials.add(StandardMaterial {
                base_color: color,
                ..default()
            })
        })
        .collect();

    commands.insert_resource(FruitAssets { mesh, materials });

    // Static camera looking straight down the -Z axis at the play plane.
    commands.spawn((
        Name::new("Main Camera"),
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 22.0).looking_at(Vec3::new(0.0, 0.0, PLAY_Z), Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.6, -0.4, 0.0)),
        GlobalTransform::default(),
    ));

    // On-screen score HUD: a large text element in the top-left corner. The
    // status bar below carries FPS only, so the score has a single home.
    commands.spawn((
        Name::new("Score HUD"),
        ScoreText,
        Text::new(score_label(0)),
        TextFont {
            font_size: 40.0,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.85, 0.25)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(16.0),
            ..default()
        },
    ));

    // Status bar: FPS only (the score now lives in the HUD above).
    commands.spawn((status_bar(StatusBarRootConfig::default()),));

    commands.spawn((status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: status_fps_value_fn(),
        color_fn: status_fps_color_fn(),
        prefix: "".to_string(),
        suffix: "fps".to_string(),
    }),));
}

/// Text shown by the score HUD for a given score.
fn score_label(score: usize) -> String {
    format!("Score: {score}")
}

/// Refresh the score HUD text whenever the score changes.
fn update_score_text(score: Res<Score>, mut q_text: Query<&mut Text, With<ScoreText>>) {
    if !score.is_changed() {
        return;
    }

    for mut text in q_text.iter_mut() {
        **text = score_label(score.0);
    }
}

/// Launch a fresh fruit from below the view on a repeating timer.
fn spawn_fruit(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<SpawnTimer>,
    assets: Res<FruitAssets>,
) {
    if !timer.tick(time.delta()).just_finished() {
        return;
    }

    let mut rng = rand::rng();

    // Spawn somewhere along the bottom and aim up-and-inward so the arc peaks
    // in view. Fruit near the edges gets nudged back toward the center.
    let x = rng.random_range(-6.0..6.0);
    let vx = rng.random_range(-2.5..2.5) - x * 0.25;
    let vy = rng.random_range(17.0..21.0);

    let material = assets.materials[rng.random_range(0..assets.materials.len())].clone();

    commands.spawn((
        Name::new("Fruit"),
        Fruit {
            radius: FRUIT_RADIUS,
        },
        Mesh3d(assets.mesh.clone()),
        MeshMaterial3d(material),
        Transform::from_xyz(x, SPAWN_Y, PLAY_Z),
        FruitMotion {
            velocity: Vec3::new(vx, vy, 0.0),
        },
    ));
}

/// Advance fruit along their arc under gravity, tumble them, and despawn any
/// that fall past the bottom (a miss).
fn move_fruit(
    mut commands: Commands,
    time: Res<Time>,
    mut q_fruit: Query<(Entity, &mut Transform, &mut FruitMotion)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut motion) in q_fruit.iter_mut() {
        motion.velocity.y -= GRAVITY * dt;
        transform.translation += motion.velocity * dt;
        transform.rotate_local_x(dt * 1.5);
        transform.rotate_local_y(dt * 2.0);

        if transform.translation.y < KILL_Y {
            commands.entity(entity).despawn();
        }
    }
}

/// Slice any fruit the swipe segment passes through this frame.
///
/// Cursor tracking and slicing live in one system on purpose: the swipe is the
/// segment from last frame's cursor to this frame's, so the read (previous),
/// the test, and the store (current) must happen in a fixed order. Splitting
/// them into two `Update` systems that share `CursorTrail` would let the store
/// race ahead of the read and collapse the segment to a point.
fn slice_fruit(
    mut commands: Commands,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut trail: ResMut<CursorTrail>,
    mut score: ResMut<Score>,
    q_fruit: Query<(Entity, &Transform, &Fruit)>,
) {
    // Releasing the button ends the swipe, so the next press starts a fresh
    // segment instead of jumping across the screen from a stale point.
    if !mouse.pressed(MouseButton::Left) {
        trail.previous = None;
        return;
    }

    let (camera, camera_transform) = *camera;
    let Some(current) = cursor_on_play_plane(&window, camera, camera_transform) else {
        return;
    };

    // On the first frame of a press there is no previous point yet; treat the
    // segment as degenerate (a point) so a stationary click can still slice.
    let previous = trail.previous.unwrap_or(current);
    trail.previous = Some(current);

    for (entity, transform, fruit) in q_fruit.iter() {
        if segment_hits_circle(
            previous.truncate(),
            current.truncate(),
            transform.translation.truncate(),
            fruit.radius,
        ) {
            // Drop the Fruit marker so it cannot be sliced twice while its
            // fragments are being generated, then trigger the explosion.
            commands
                .entity(entity)
                .remove::<Fruit>()
                .remove::<FruitMotion>()
                .insert(ExplodeMesh {
                    fragment_count: FRAGMENT_COUNT,
                });
            **score += 1;
        }
    }
}

/// Spawn each fragment of a sliced fruit as a flying, self-despawning piece.
fn on_fragments_spawned(
    insert: On<Insert, ExplodeFragments>,
    mut commands: Commands,
    q_fragments: Query<(
        &ExplodeFragments,
        &Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let entity = insert.entity;

    let Ok((fragments, transform, material)) = q_fragments.get(entity) else {
        return;
    };

    let origin = transform.translation;
    // The sliced shell still carries its material, so fragments burst in the
    // same color as the fruit they came from.
    let material = material.0.clone();

    for fragment in fragments.iter() {
        commands.spawn((
            Name::new("Fragment"),
            Mesh3d(fragment.mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(origin),
            FragmentMotion {
                // Push outward along the slice direction, with a little lift.
                velocity: fragment.direction * FRAGMENT_SPEED + Vec3::Y * 1.5,
            },
            TempEntity(FRAGMENT_LIFETIME),
        ));
    }

    // Remove the sliced shell; TempEntity retires the fragments later.
    commands.entity(entity).despawn();
}

/// Move fragments along their velocity under gravity, and tumble them.
fn move_fragments(time: Res<Time>, mut q_fragments: Query<(&mut Transform, &mut FragmentMotion)>) {
    let dt = time.delta_secs();

    for (mut transform, mut motion) in q_fragments.iter_mut() {
        motion.velocity.y -= GRAVITY * dt;
        transform.translation += motion.velocity * dt;
        transform.rotate_local_x(dt * 4.0);
        transform.rotate_local_y(dt * 3.0);
    }
}

/// World position where the cursor ray meets the play plane, if the cursor is
/// on screen and its ray actually crosses the plane.
fn cursor_on_play_plane(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec3> {
    let cursor = window.cursor_position()?;

    let ray = camera.viewport_to_world(camera_transform, cursor).ok()?;
    let plane = InfinitePlane3d::new(Vec3::Z);
    let distance = ray.intersect_plane(Vec3::new(0.0, 0.0, PLAY_Z), plane)?;

    Some(ray.get_point(distance))
}

/// True when the segment `a -> b` passes within `radius` of `center`.
///
/// This is the swipe/slice hit test: the closest point on the segment to the
/// fruit center is found, then compared against the fruit radius. A degenerate
/// segment (a == b) reduces to a point-in-circle test.
fn segment_hits_circle(a: Vec2, b: Vec2, center: Vec2, radius: f32) -> bool {
    let ab = b - a;
    let len_sq = ab.length_squared();

    let closest = if len_sq <= f32::EPSILON {
        a
    } else {
        let t = ((center - a).dot(ab) / len_sq).clamp(0.0, 1.0);
        a + ab * t
    };

    closest.distance_squared(center) <= radius * radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_inside_circle_hits() {
        // Degenerate segment (a == b) that sits inside the circle.
        assert!(segment_hits_circle(
            Vec2::new(0.5, 0.0),
            Vec2::new(0.5, 0.0),
            Vec2::ZERO,
            1.0
        ));
    }

    #[test]
    fn point_outside_circle_misses() {
        assert!(!segment_hits_circle(
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::ZERO,
            1.0
        ));
    }

    #[test]
    fn swipe_crossing_circle_hits() {
        // A horizontal swipe well left-to-right that passes through the center.
        assert!(segment_hits_circle(
            Vec2::new(-5.0, 0.0),
            Vec2::new(5.0, 0.0),
            Vec2::ZERO,
            1.0
        ));
    }

    #[test]
    fn swipe_grazing_within_radius_hits() {
        // Passes above the center but still within the radius (0.5 < 1.0).
        assert!(segment_hits_circle(
            Vec2::new(-5.0, 0.5),
            Vec2::new(5.0, 0.5),
            Vec2::ZERO,
            1.0
        ));
    }

    #[test]
    fn swipe_missing_by_a_margin_misses() {
        // Parallel swipe further out than the radius.
        assert!(!segment_hits_circle(
            Vec2::new(-5.0, 1.5),
            Vec2::new(5.0, 1.5),
            Vec2::ZERO,
            1.0
        ));
    }

    #[test]
    fn swipe_endpoints_short_of_circle_misses() {
        // The infinite line would cross the circle, but the segment stops short.
        assert!(!segment_hits_circle(
            Vec2::new(-5.0, 0.0),
            Vec2::new(-3.0, 0.0),
            Vec2::ZERO,
            1.0
        ));
    }
}
