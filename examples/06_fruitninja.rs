//! A tiny "fruit ninja" style game built entirely from procedural shapes.
//!
//! Boot into a main menu, click to play. Octahedron "fruits" are launched up in
//! a parabolic arc from below the view; hold the Left Mouse Button and swipe the
//! cursor across one to slice it into flying fragments (via `ExplodeMeshPlugin`)
//! and score a point. Dark "bombs" are mixed in: slicing a bomb deals lethal
//! damage to the player through the crate's health system and ends the run at
//! the game-over screen. Fruit you miss just falls off the bottom.
//!
//! Everything here is plain shapes and hand-rolled kinematics: no assets, no
//! physics engine. It reuses the crate's `TriangleMeshBuilder` (meshes),
//! `ExplodeMeshPlugin` (the slice effect), `TempEntityPlugin` (fragment
//! cleanup), `HealthPlugin` (the lose condition) and `StatusBarPlugin` (the FPS
//! overlay); the menu / states use Bevy's own state machine.

use std::collections::VecDeque;

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

/// Maximum number of cursor points kept for the blade trail.
const BLADE_TRAIL_LEN: usize = 16;

/// How long a floating "+N" popup lives before it despawns, in seconds.
const POPUP_LIFETIME: f32 = 0.8;

/// How fast a floating popup rises up the screen, in pixels per second.
const POPUP_RISE_SPEED: f32 = 70.0;

/// Chance that a launched object is a bomb rather than a fruit.
const BOMB_CHANCE: f64 = 0.2;

/// Player health at the start of a run. Slicing a bomb deals lethal damage,
/// so this is effectively a single life; it is still a real `Health` value so
/// the example drives the crate's health system end to end.
const PLAYER_HEALTH: f32 = 1.0;

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
    app.add_plugins(HealthPlugin);

    app.init_state::<GameState>();

    app.init_resource::<Score>();
    app.insert_resource(SpawnTimer(Timer::from_seconds(
        SPAWN_INTERVAL,
        TimerMode::Repeating,
    )));
    app.init_resource::<CursorTrail>();
    app.init_resource::<BladeTrail>();

    // Persistent scene: camera, light and the FPS status bar live for the whole
    // run, independent of game state.
    app.add_systems(Startup, setup);

    // Main menu.
    app.add_systems(OnEnter(GameState::Menu), spawn_menu);
    app.add_systems(Update, menu_click.run_if(in_state(GameState::Menu)));

    // Playing: reset the run, spawn the player + HUD, then run the gameplay loop.
    app.add_systems(
        OnEnter(GameState::Playing),
        (start_game, spawn_player, spawn_hud),
    );
    app.add_systems(
        Update,
        (
            spawn_projectile,
            move_projectiles,
            slice_objects,
            move_fragments,
            update_score_text,
            draw_blade_trail,
            animate_floating_text,
            giveup_on_escape,
        )
            .run_if(in_state(GameState::Playing)),
    );

    // Game over screen.
    app.add_systems(OnEnter(GameState::GameOver), spawn_game_over);
    app.add_systems(Update, gameover_click.run_if(in_state(GameState::GameOver)));

    app.add_observer(on_fragments_spawned);
    app.add_observer(on_player_died);

    app.run();
}

/// Top-level game flow: the menu, the playable run, and the game-over screen.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

/// Running number of fruits sliced. Shown in the score HUD.
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

/// Recent cursor world positions along the current swipe, newest last. Drawn as
/// a fading "blade" and cleared when the button is released.
#[derive(Resource, Default)]
struct BladeTrail {
    points: VecDeque<Vec3>,
}

/// Marker for the on-screen score HUD text.
#[derive(Component)]
struct ScoreText;

/// A short-lived UI text that rises and fades out (a "+N" or combo popup).
#[derive(Component)]
struct FloatingText {
    /// Seconds since the popup was spawned.
    age: f32,
    /// Total lifetime in seconds; the popup despawns once `age` reaches it.
    lifetime: f32,
    /// Upward screen speed in pixels per second.
    rise_speed: f32,
    /// Base color; its alpha is ramped down as the popup ages.
    color: Color,
}

/// A slice-able object (fruit or bomb) flying through the scene.
#[derive(Component)]
struct Sliceable {
    /// Slice hit radius in world units.
    radius: f32,
}

/// Marker for a bomb. Slicing one is an instant loss; a plain `Sliceable`
/// without this marker is fruit.
#[derive(Component)]
struct Bomb;

/// Marker for the player entity that owns the run's `Health`.
#[derive(Component)]
struct Player;

/// Velocity carried by a flying projectile (fruit or bomb).
#[derive(Component)]
struct Projectile {
    velocity: Vec3,
}

/// Velocity carried by a flying fragment of a sliced fruit.
#[derive(Component)]
struct FragmentMotion {
    velocity: Vec3,
}

/// Shared render assets so fruit, bombs and fragments are cheap to spawn.
#[derive(Resource)]
struct FruitAssets {
    mesh: Handle<Mesh>,
    materials: Vec<Handle<StandardMaterial>>,
    bomb_material: Handle<StandardMaterial>,
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
    let fruit_materials = palette
        .into_iter()
        .map(|color| {
            materials.add(StandardMaterial {
                base_color: color,
                ..default()
            })
        })
        .collect();

    // Bombs reuse the same mesh in a dark, unmistakable material.
    let bomb_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.08, 0.10),
        perceptual_roughness: 0.3,
        metallic: 0.8,
        ..default()
    });

    commands.insert_resource(FruitAssets {
        mesh,
        materials: fruit_materials,
        bomb_material,
    });

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

    // Status bar: FPS only (the score lives in the in-game HUD, spawned per
    // run in `spawn_hud`).
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

/// Spawn the player entity that owns the run's health, scoped to `Playing`.
fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Name::new("Player"),
        Player,
        Health::new(PLAYER_HEALTH),
        DespawnOnExit(GameState::Playing),
    ));
}

/// When the player's health hits zero (a sliced bomb), end the run.
fn on_player_died(
    add: On<Add, HealthZeroMarker>,
    q_player: Query<(), With<Player>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if q_player.contains(add.entity) {
        next.set(GameState::GameOver);
    }
}

/// Spawn a floating "+N" / combo popup at a viewport position, scoped to
/// `Playing`. It rises and fades out via `animate_floating_text`.
fn spawn_floating_text(
    commands: &mut Commands,
    viewport_pos: Vec2,
    text: impl Into<String>,
    size: f32,
    color: Color,
) {
    commands.spawn((
        Name::new("Floating Text"),
        FloatingText {
            age: 0.0,
            lifetime: POPUP_LIFETIME,
            rise_speed: POPUP_RISE_SPEED,
            color,
        },
        DespawnOnExit(GameState::Playing),
        Text::new(text.into()),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(viewport_pos.x),
            top: Val::Px(viewport_pos.y),
            ..default()
        },
    ));
}

/// Advance floating popups: rise up the screen, fade out, and despawn at the
/// end of their lifetime.
fn animate_floating_text(
    mut commands: Commands,
    time: Res<Time>,
    mut q_text: Query<(Entity, &mut FloatingText, &mut Node, &mut TextColor)>,
) {
    let dt = time.delta_secs();

    for (entity, mut floating, mut node, mut text_color) in q_text.iter_mut() {
        floating.age += dt;
        if floating.age >= floating.lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        if let Val::Px(top) = node.top {
            node.top = Val::Px(top - floating.rise_speed * dt);
        }

        let alpha = 1.0 - floating.age / floating.lifetime;
        text_color.0 = floating.color.with_alpha(alpha);
    }
}

/// A full-screen, centered UI column used by the menu and game-over screens.
fn centered_screen() -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        row_gap: Val::Px(16.0),
        ..default()
    }
}

/// One line of menu / game-over text at the given size and color.
fn screen_text(text: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(text.into()),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
    )
}

/// Spawn the main menu (title + prompt), scoped to the `Menu` state.
fn spawn_menu(mut commands: Commands) {
    commands.spawn((
        Name::new("Main Menu"),
        DespawnOnExit(GameState::Menu),
        centered_screen(),
        children![
            screen_text("FRUIT NINJA", 72.0, Color::srgb(0.95, 0.85, 0.25)),
            screen_text("Click to play", 32.0, Color::WHITE),
            screen_text(
                "swipe to slice - avoid the bombs - Esc to give up",
                20.0,
                Color::srgb(0.7, 0.7, 0.7),
            ),
        ],
    ));
}

/// Start the game on a click from the menu.
fn menu_click(mouse: Res<ButtonInput<MouseButton>>, mut next: ResMut<NextState<GameState>>) {
    if mouse.just_pressed(MouseButton::Left) {
        next.set(GameState::Playing);
    }
}

/// Reset per-run state when a new game starts.
fn start_game(
    mut score: ResMut<Score>,
    mut timer: ResMut<SpawnTimer>,
    mut trail: ResMut<CursorTrail>,
    mut blade: ResMut<BladeTrail>,
) {
    score.0 = 0;
    timer.reset();
    trail.previous = None;
    // Clear any trail left over from a swipe that ended the previous run so the
    // new run does not flash a stale blade.
    blade.points.clear();
}

/// Spawn the in-game HUD (score), scoped to the `Playing` state.
fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        Name::new("Score HUD"),
        ScoreText,
        DespawnOnExit(GameState::Playing),
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
}

/// Give up the current run with Escape (a stand-in lose trigger until bombs
/// provide the real one).
fn giveup_on_escape(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next.set(GameState::GameOver);
    }
}

/// Spawn the game-over screen with the final score, scoped to `GameOver`.
fn spawn_game_over(mut commands: Commands, score: Res<Score>) {
    commands.spawn((
        Name::new("Game Over"),
        DespawnOnExit(GameState::GameOver),
        centered_screen(),
        children![
            screen_text("GAME OVER", 72.0, Color::srgb(0.9, 0.25, 0.25)),
            screen_text(score_label(score.0), 40.0, Color::srgb(0.95, 0.85, 0.25)),
            screen_text("Click to return to menu", 28.0, Color::WHITE),
        ],
    ));
}

/// Return to the menu on a click from the game-over screen.
fn gameover_click(mouse: Res<ButtonInput<MouseButton>>, mut next: ResMut<NextState<GameState>>) {
    if mouse.just_pressed(MouseButton::Left) {
        next.set(GameState::Menu);
    }
}

/// Launch a fresh fruit or bomb from below the view on a repeating timer.
fn spawn_projectile(
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
    // in view. Objects near the edges get nudged back toward the center.
    let x = rng.random_range(-6.0..6.0);
    let vx = rng.random_range(-2.5..2.5) - x * 0.25;
    let vy = rng.random_range(17.0..21.0);

    let is_bomb = rng.random_bool(BOMB_CHANCE);
    let material = if is_bomb {
        assets.bomb_material.clone()
    } else {
        assets.materials[rng.random_range(0..assets.materials.len())].clone()
    };

    let mut object = commands.spawn((
        Name::new(if is_bomb { "Bomb" } else { "Fruit" }),
        Sliceable {
            radius: FRUIT_RADIUS,
        },
        DespawnOnExit(GameState::Playing),
        Mesh3d(assets.mesh.clone()),
        MeshMaterial3d(material),
        Transform::from_xyz(x, SPAWN_Y, PLAY_Z),
        Projectile {
            velocity: Vec3::new(vx, vy, 0.0),
        },
    ));

    if is_bomb {
        object.insert(Bomb);
    }
}

/// Advance projectiles along their arc under gravity, tumble them, and despawn
/// any that fall past the bottom (a miss; harmless for both fruit and bombs).
fn move_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut q_projectiles: Query<(Entity, &mut Transform, &mut Projectile)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut motion) in q_projectiles.iter_mut() {
        motion.velocity.y -= GRAVITY * dt;
        transform.translation += motion.velocity * dt;
        transform.rotate_local_x(dt * 1.5);
        transform.rotate_local_y(dt * 2.0);

        if transform.translation.y < KILL_Y {
            commands.entity(entity).despawn();
        }
    }
}

/// Slice any object (fruit or bomb) the swipe segment passes through this frame.
///
/// Cursor tracking and slicing live in one system on purpose: the swipe is the
/// segment from last frame's cursor to this frame's, so the read (previous),
/// the test, and the store (current) must happen in a fixed order. Splitting
/// them into two `Update` systems that share `CursorTrail` would let the store
/// race ahead of the read and collapse the segment to a point.
fn slice_objects(
    mut commands: Commands,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    player: Single<Entity, With<Player>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut trail: ResMut<CursorTrail>,
    mut blade: ResMut<BladeTrail>,
    mut score: ResMut<Score>,
    q_sliceable: Query<(Entity, &Transform, &Sliceable, Has<Bomb>)>,
) {
    // Releasing the button ends the swipe, so the next press starts a fresh
    // segment instead of jumping across the screen from a stale point, and the
    // blade trail is cleared so it does not linger.
    if !mouse.pressed(MouseButton::Left) {
        trail.previous = None;
        blade.points.clear();
        return;
    }

    let (camera, camera_transform) = *camera;
    let Some(current) = cursor_on_play_plane(&window, camera, camera_transform) else {
        return;
    };

    // Record the cursor point for the blade trail, dropping the oldest once the
    // trail is at its cap.
    blade.points.push_back(current);
    while blade.points.len() > BLADE_TRAIL_LEN {
        blade.points.pop_front();
    }

    // On the first frame of a press there is no previous point yet; treat the
    // segment as degenerate (a point) so a stationary click can still slice.
    let previous = trail.previous.unwrap_or(current);
    trail.previous = Some(current);

    for (entity, transform, sliceable, is_bomb) in q_sliceable.iter() {
        if !segment_hits_circle(
            previous.truncate(),
            current.truncate(),
            transform.translation.truncate(),
            sliceable.radius,
        ) {
            continue;
        }

        // Drop the Sliceable marker so the same object cannot be sliced twice
        // while its fragments are being generated, then trigger the explosion.
        commands
            .entity(entity)
            .remove::<Sliceable>()
            .remove::<Projectile>()
            .insert(ExplodeMesh {
                fragment_count: FRAGMENT_COUNT,
            });

        if is_bomb {
            // Slicing a bomb is an instant loss: deal lethal damage to the
            // player, which trips HealthZeroMarker -> GameOver.
            commands.trigger(HealthApplyDamage {
                entity: *player,
                source: Some(entity),
                amount: PLAYER_HEALTH,
            });
        } else {
            **score += 1;

            // Pop a rising "+1" at the fruit's screen position for feedback.
            if let Ok(viewport_pos) =
                camera.world_to_viewport(camera_transform, transform.translation)
            {
                spawn_floating_text(
                    &mut commands,
                    viewport_pos,
                    "+1",
                    30.0,
                    Color::srgb(0.95, 0.85, 0.25),
                );
            }
        }
    }
}

/// Draw the current swipe as a blade trail: fading line segments from the
/// oldest (faint) cursor point to the newest (bright).
fn draw_blade_trail(blade: Res<BladeTrail>, mut gizmos: Gizmos) {
    let count = blade.points.len();
    if count < 2 {
        return;
    }

    // Lift the trail slightly toward the camera so it draws in front of fruit.
    let lift = Vec3::Z * 0.5;

    for (i, (a, b)) in blade
        .points
        .iter()
        .zip(blade.points.iter().skip(1))
        .enumerate()
    {
        // t ramps 0 -> 1 from tail to head; alpha follows so the blade looks
        // like it is trailing the cursor.
        let t = (i + 1) as f32 / (count - 1) as f32;
        let color = Color::srgba(0.7, 0.95, 1.0, t);
        gizmos.line(*a + lift, *b + lift, color);
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
            DespawnOnExit(GameState::Playing),
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
