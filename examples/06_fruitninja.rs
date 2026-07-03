//! A tiny "fruit ninja" style game built entirely from procedural shapes.
//!
//! Boot into a main menu, click to play. Octahedron "fruits" are launched up in
//! a parabolic arc from below the view; hold the Left Mouse Button and swipe the
//! cursor across one to slice it into flying fragments (via `ExplodeMeshPlugin`)
//! and score a point. A bright blade trail follows the swipe, and each slice
//! pops a rising "+N". Slicing several fruit in one continuous swipe builds a
//! combo: the Nth fruit is worth N points and a "COMBO xN" banner flashes. The
//! combo runs on a short time window, so it survives slow swipes and separate
//! strokes as long as you keep landing hits. Dark
//! "bombs" are mixed in: slicing a bomb deals lethal damage to the player
//! through the crate's health system and ends the run at the game-over screen.
//! Fruit you miss just falls off the bottom.
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

/// Seconds between launches at the start of a run, and the floor it ramps down
/// to as the run goes on.
const SPAWN_INTERVAL: f32 = 0.9;
const SPAWN_INTERVAL_FLOOR: f32 = 0.35;

/// Seconds of play over which difficulty ramps from start to floor/cap.
const DIFFICULTY_RAMP_SECS: f32 = 60.0;

/// Fragments requested per slice.
const FRAGMENT_COUNT: usize = 10;

/// Speed fragments fly away from the slice point, units per second.
const FRAGMENT_SPEED: f32 = 5.0;

/// How long a fragment lives before it despawns, in seconds.
const FRAGMENT_LIFETIME: f32 = 3.0;

/// Maximum number of cursor points kept for the blade trail.
const BLADE_TRAIL_LEN: usize = 16;

/// Minimum cursor speed on the play plane, in world units per second, for the
/// swipe to count as active. Below this the swipe is "stalled": it does not
/// slice and the combo resets, so holding the button still cannot farm points.
const MIN_SWIPE_SPEED: f32 = 6.0;

/// Resting camera position; shake offsets are applied relative to this.
const CAMERA_BASE: Vec3 = Vec3::new(0.0, 0.0, 22.0);

/// How fast camera shake trauma decays, per second.
const SHAKE_DECAY: f32 = 1.8;

/// Maximum camera offset at full trauma, in world units.
const SHAKE_MAX_OFFSET: f32 = 0.6;

/// Trauma added by slicing a fruit and by slicing a bomb.
const SLICE_TRAUMA: f32 = 0.28;
const BOMB_TRAUMA: f32 = 0.75;

/// Seconds the red flash holds before the game-over screen after a bomb.
const DYING_BEAT: f32 = 0.35;

/// How long a fruit "pops" (scales up) before it bursts, and how far it grows.
const SLICE_POP_TIME: f32 = 0.08;
const SLICE_POP_SCALE: f32 = 1.45;

/// Seconds after a slice you have to land the next hit and keep the combo. The
/// combo survives slow swipes / separate strokes as long as hits stay inside it.
const COMBO_WINDOW: f32 = 1.2;

/// Chance a (non-bomb) launch is a golden bonus fruit.
const GOLDEN_CHANCE: f64 = 0.08;

/// Flat points a golden fruit is worth, and the longer combo window it grants.
const GOLDEN_POINTS: usize = 5;
const COMBO_WINDOW_GOLDEN: f32 = 2.5;

/// How long a floating "+N" popup lives before it despawns, in seconds.
const POPUP_LIFETIME: f32 = 0.8;

/// How fast a floating popup rises up the screen, in pixels per second.
const POPUP_RISE_SPEED: f32 = 70.0;

/// Chance a launched object is a bomb: starts at `BOMB_CHANCE_START` and ramps
/// up to `BOMB_CHANCE_CAP` over `DIFFICULTY_RAMP_SECS`.
const BOMB_CHANCE_START: f64 = 0.2;
const BOMB_CHANCE_CAP: f64 = 0.35;

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
    app.init_resource::<HighScore>();
    app.init_resource::<NewBest>();
    app.insert_resource(SpawnTimer(Timer::from_seconds(
        SPAWN_INTERVAL,
        TimerMode::Repeating,
    )));
    app.init_resource::<CursorTrail>();
    app.init_resource::<BladeTrail>();
    app.init_resource::<Combo>();
    app.init_resource::<CameraShake>();
    app.init_resource::<DyingTimer>();
    app.init_resource::<Elapsed>();

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
            tick_elapsed,
            tick_combo,
            spawn_projectile,
            move_projectiles,
            slice_objects,
            resolve_slice_pop,
            move_fragments,
            update_score_text,
            update_combo_text,
            draw_blade_trail,
            animate_floating_text,
            fade_red_flash,
            advance_dying,
            giveup_on_escape,
        )
            .run_if(in_state(GameState::Playing)),
    );

    // Camera shake settles the camera in any state, so it always eases back.
    app.add_systems(Update, apply_camera_shake);

    // Game over screen.
    app.add_systems(
        OnEnter(GameState::GameOver),
        (record_high_score, spawn_game_over).chain(),
    );
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

/// Best score across runs this session (not reset per run).
#[derive(Resource, Default)]
struct HighScore(usize);

/// Whether the most recent run set a new high score (for the game-over screen).
#[derive(Resource, Default)]
struct NewBest(bool);

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

/// The current combo: how many fruit are in the chain and how long is left on
/// the window to keep it alive. Each slice scores its combo index (1, 2, 3, ...)
/// and refreshes the window; when the window runs out the combo resets. Slicing
/// still needs an active swipe, so the combo cannot be farmed by holding.
#[derive(Resource, Default)]
struct Combo {
    count: usize,
    timer: f32,
    /// Total points scored during the current combo, for the end-of-combo tally.
    points: usize,
}

/// Advance the combo for one more sliced fruit: bump the count, refresh the
/// window, and return the points earned (the new count).
fn advance_combo(combo: &mut Combo) -> usize {
    combo.count += 1;
    combo.timer = COMBO_WINDOW;
    combo.points += combo.count;
    combo.count
}

/// Count the combo window down; when it runs out the combo ends: show a tally
/// for a real (>= 2 hit) combo, then reset it.
fn tick_combo(
    time: Res<Time>,
    mut commands: Commands,
    window: Single<&Window>,
    mut combo: ResMut<Combo>,
) {
    if combo.count == 0 {
        return;
    }
    combo.timer -= time.delta_secs();
    if combo.timer > 0.0 {
        return;
    }

    if combo.count >= 2 {
        // Centered-ish tally near the top of the screen.
        let pos = Vec2::new(window.width() * 0.5 - 110.0, window.height() * 0.3);
        spawn_floating_text(
            &mut commands,
            pos,
            format!("COMBO x{} +{}", combo.count, combo.points),
            52.0,
            Color::srgb(1.0, 0.75, 0.2),
        );
    }

    combo.count = 0;
    combo.points = 0;
}

/// Seconds elapsed in the current run, driving the difficulty ramp.
#[derive(Resource, Default)]
struct Elapsed(f32);

/// Normalized difficulty progress in 0..1 for a given elapsed run time.
fn ramp_t(elapsed: f32) -> f32 {
    (elapsed / DIFFICULTY_RAMP_SECS).clamp(0.0, 1.0)
}

/// Spawn interval (seconds) for a given elapsed run time: eases from
/// `SPAWN_INTERVAL` down to `SPAWN_INTERVAL_FLOOR`.
fn spawn_interval_for(elapsed: f32) -> f32 {
    SPAWN_INTERVAL + (SPAWN_INTERVAL_FLOOR - SPAWN_INTERVAL) * ramp_t(elapsed)
}

/// Bomb chance for a given elapsed run time: eases from `BOMB_CHANCE_START` up
/// to `BOMB_CHANCE_CAP`.
fn bomb_chance_for(elapsed: f32) -> f64 {
    BOMB_CHANCE_START + (BOMB_CHANCE_CAP - BOMB_CHANCE_START) * ramp_t(elapsed) as f64
}

/// Advance the run clock each frame while playing.
fn tick_elapsed(time: Res<Time>, mut elapsed: ResMut<Elapsed>) {
    elapsed.0 += time.delta_secs();
}

/// Camera shake energy; decays to 0, offsetting the camera while positive.
#[derive(Resource, Default)]
struct CameraShake {
    trauma: f32,
}

/// Countdown before the game-over screen after a bomb, for the red-flash beat.
#[derive(Resource, Default)]
struct DyingTimer {
    remaining: Option<f32>,
}

/// Marker for the main camera so the shake system can find it.
#[derive(Component)]
struct MainCamera;

/// Full-screen red flash shown briefly when a bomb ends the run.
#[derive(Component)]
struct RedFlash {
    age: f32,
    lifetime: f32,
}

/// A sliced fruit mid-"pop": it scales up for a beat, then bursts.
#[derive(Component)]
struct SlicePop {
    timer: f32,
    base_scale: Vec3,
}

/// Grow a popping fruit, then trigger its explosion when the pop finishes.
fn resolve_slice_pop(
    time: Res<Time>,
    mut commands: Commands,
    mut q_pop: Query<(Entity, &mut Transform, &mut SlicePop)>,
) {
    let dt = time.delta_secs();
    for (entity, mut transform, mut pop) in q_pop.iter_mut() {
        pop.timer -= dt;
        if pop.timer <= 0.0 {
            // Restore the base scale so fragments burst at the fruit's size.
            transform.scale = pop.base_scale;
            commands
                .entity(entity)
                .remove::<SlicePop>()
                .insert(ExplodeMesh {
                    fragment_count: FRAGMENT_COUNT,
                });
            continue;
        }
        let progress = 1.0 - (pop.timer / SLICE_POP_TIME).clamp(0.0, 1.0);
        transform.scale = pop.base_scale * (1.0 + (SLICE_POP_SCALE - 1.0) * progress);
    }
}

/// Marker for the on-screen score HUD text.
#[derive(Component)]
struct ScoreText;

/// Marker for the live combo HUD text.
#[derive(Component)]
struct ComboText;

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

/// Marker for a golden bonus fruit: worth a flat `GOLDEN_POINTS` and grants
/// extra combo time.
#[derive(Component)]
struct Golden;

/// Marker for the player entity that owns the run's `Health`.
#[derive(Component)]
struct Player;

/// Velocity carried by a flying projectile (fruit or bomb).
#[derive(Component)]
struct Projectile {
    velocity: Vec3,
    /// Per-object tumble rates (radians/sec) about the local X and Y axes.
    spin: Vec2,
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
    gold_material: Handle<StandardMaterial>,
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

    // Golden bonus fruit: bright, emissive gold so it stands out.
    let gold_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.82, 0.15),
        emissive: LinearRgba::rgb(0.6, 0.45, 0.05),
        metallic: 0.9,
        perceptual_roughness: 0.25,
        ..default()
    });

    commands.insert_resource(FruitAssets {
        mesh,
        materials: fruit_materials,
        bomb_material,
        gold_material,
    });

    // Static camera looking straight down the -Z axis at the play plane.
    commands.spawn((
        Name::new("Main Camera"),
        MainCamera,
        Camera3d::default(),
        Transform::from_translation(CAMERA_BASE).looking_at(Vec3::new(0.0, 0.0, PLAY_Z), Vec3::Y),
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

/// When the player's health hits zero (a sliced bomb), kick a big shake and a
/// red flash, then transition to game over after a short beat (see
/// `advance_dying`). The `Escape` give-up stays instant.
fn on_player_died(
    add: On<Add, HealthZeroMarker>,
    q_player: Query<(), With<Player>>,
    mut commands: Commands,
    mut shake: ResMut<CameraShake>,
    mut dying: ResMut<DyingTimer>,
) {
    if !q_player.contains(add.entity) || dying.remaining.is_some() {
        return;
    }

    shake.trauma = (shake.trauma + BOMB_TRAUMA).min(1.0);
    dying.remaining = Some(DYING_BEAT);

    commands.spawn((
        Name::new("Red Flash"),
        RedFlash {
            age: 0.0,
            lifetime: DYING_BEAT,
        },
        DespawnOnExit(GameState::Playing),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.9, 0.1, 0.1, 0.5)),
    ));
}

/// Ease the camera toward its base while applying a decaying random shake.
fn apply_camera_shake(
    time: Res<Time>,
    mut shake: ResMut<CameraShake>,
    mut q_camera: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(mut transform) = q_camera.single_mut() else {
        return;
    };

    shake.trauma = (shake.trauma - SHAKE_DECAY * time.delta_secs()).max(0.0);

    // Square the trauma so small residual energy fades to nothing quickly.
    let amount = shake.trauma * shake.trauma;
    if amount <= 0.0 {
        transform.translation = CAMERA_BASE;
        return;
    }

    let mut rng = rand::rng();
    let offset = Vec3::new(
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
        0.0,
    ) * SHAKE_MAX_OFFSET
        * amount;
    transform.translation = CAMERA_BASE + offset;
}

/// Count down the post-bomb beat and switch to the game-over screen when done.
fn advance_dying(
    time: Res<Time>,
    mut dying: ResMut<DyingTimer>,
    mut next: ResMut<NextState<GameState>>,
) {
    let Some(remaining) = dying.remaining.as_mut() else {
        return;
    };
    *remaining -= time.delta_secs();
    if *remaining <= 0.0 {
        dying.remaining = None;
        next.set(GameState::GameOver);
    }
}

/// Fade the red flash out over its lifetime, then despawn it.
fn fade_red_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut q_flash: Query<(Entity, &mut RedFlash, &mut BackgroundColor)>,
) {
    let dt = time.delta_secs();
    for (entity, mut flash, mut background) in q_flash.iter_mut() {
        flash.age += dt;
        let alpha = (1.0 - flash.age / flash.lifetime).clamp(0.0, 1.0) * 0.5;
        background.0 = Color::srgba(0.9, 0.1, 0.1, alpha);
        if flash.age >= flash.lifetime {
            commands.entity(entity).despawn();
        }
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
fn spawn_menu(mut commands: Commands, high: Res<HighScore>) {
    commands.spawn((
        Name::new("Main Menu"),
        DespawnOnExit(GameState::Menu),
        centered_screen(),
        children![
            screen_text("FRUIT NINJA", 72.0, Color::srgb(0.95, 0.85, 0.25)),
            screen_text("Click to play", 32.0, Color::WHITE),
            screen_text(
                format!("Best: {}", high.0),
                24.0,
                Color::srgb(0.95, 0.85, 0.25),
            ),
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
    mut combo: ResMut<Combo>,
    mut shake: ResMut<CameraShake>,
    mut dying: ResMut<DyingTimer>,
    mut elapsed: ResMut<Elapsed>,
) {
    score.0 = 0;
    timer.reset();
    trail.previous = None;
    // Clear any trail left over from a swipe that ended the previous run so the
    // new run does not flash a stale blade.
    blade.points.clear();
    combo.count = 0;
    combo.timer = 0.0;
    combo.points = 0;
    shake.trauma = 0.0;
    dying.remaining = None;
    elapsed.0 = 0.0;
    timer.set_duration(std::time::Duration::from_secs_f32(SPAWN_INTERVAL));
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

    commands.spawn((
        Name::new("Combo HUD"),
        ComboText,
        DespawnOnExit(GameState::Playing),
        Text::new(""),
        TextFont {
            font_size: 34.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 0.55, 0.1, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(60.0),
            left: Val::Px(16.0),
            ..default()
        },
    ));
}

/// Show the live combo count while a combo is running, fading with its window.
fn update_combo_text(
    combo: Res<Combo>,
    mut q_text: Query<(&mut Text, &mut TextColor), With<ComboText>>,
) {
    for (mut text, mut color) in q_text.iter_mut() {
        if combo.count >= 2 {
            **text = format!("Combo x{}", combo.count);
            // Fade with the remaining window so it visibly cools down.
            let alpha = (combo.timer / COMBO_WINDOW).clamp(0.2, 1.0);
            color.0 = Color::srgba(1.0, 0.55, 0.1, alpha);
        } else {
            text.clear();
        }
    }
}

/// Give up the current run with Escape (a stand-in lose trigger until bombs
/// provide the real one).
fn giveup_on_escape(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next.set(GameState::GameOver);
    }
}

/// Spawn the game-over screen with the final score, scoped to `GameOver`.
fn spawn_game_over(
    mut commands: Commands,
    score: Res<Score>,
    high: Res<HighScore>,
    new_best: Res<NewBest>,
) {
    commands
        .spawn((
            Name::new("Game Over"),
            DespawnOnExit(GameState::GameOver),
            centered_screen(),
        ))
        .with_children(|parent| {
            parent.spawn(screen_text("GAME OVER", 72.0, Color::srgb(0.9, 0.25, 0.25)));
            parent.spawn(screen_text(
                score_label(score.0),
                40.0,
                Color::srgb(0.95, 0.85, 0.25),
            ));
            if new_best.0 {
                parent.spawn(screen_text("New best!", 32.0, Color::srgb(0.4, 0.95, 0.5)));
            } else {
                parent.spawn(screen_text(
                    format!("Best: {}", high.0),
                    28.0,
                    Color::srgb(0.7, 0.7, 0.7),
                ));
            }
            parent.spawn(screen_text("Click to return to menu", 28.0, Color::WHITE));
        });
}

/// Record the run's score into the session high score, flagging a new best.
fn record_high_score(
    score: Res<Score>,
    mut high: ResMut<HighScore>,
    mut new_best: ResMut<NewBest>,
) {
    new_best.0 = score.0 > high.0;
    high.0 = high.0.max(score.0);
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
    elapsed: Res<Elapsed>,
    assets: Res<FruitAssets>,
) {
    if !timer.tick(time.delta()).just_finished() {
        return;
    }

    // Ramp the next interval down as the run goes on, so launches speed up.
    timer.set_duration(std::time::Duration::from_secs_f32(spawn_interval_for(
        elapsed.0,
    )));

    let mut rng = rand::rng();

    // Spawn somewhere along the bottom and aim up-and-inward so the arc peaks
    // in view. Objects near the edges get nudged back toward the center.
    let x = rng.random_range(-6.0..6.0);
    let vx = rng.random_range(-2.5..2.5) - x * 0.25;
    let vy = rng.random_range(17.0..21.0);

    let is_bomb = rng.random_bool(bomb_chance_for(elapsed.0));
    // A non-bomb launch is occasionally a golden bonus fruit.
    let is_golden = !is_bomb && rng.random_bool(GOLDEN_CHANCE);
    let material = if is_bomb {
        assets.bomb_material.clone()
    } else if is_golden {
        assets.gold_material.clone()
    } else {
        assets.materials[rng.random_range(0..assets.materials.len())].clone()
    };

    // Vary size (bombs stay in a tighter range so they read as bombs) and give
    // each object its own tumble. Scale the hit radius to match the visible size.
    let scale = if is_bomb {
        rng.random_range(0.9..1.15)
    } else {
        rng.random_range(0.75..1.35)
    };
    let spin = Vec2::new(rng.random_range(-2.5..2.5), rng.random_range(-2.5..2.5));

    let mut object = commands.spawn((
        Name::new(if is_bomb { "Bomb" } else { "Fruit" }),
        Sliceable {
            radius: FRUIT_RADIUS * scale,
        },
        DespawnOnExit(GameState::Playing),
        Mesh3d(assets.mesh.clone()),
        MeshMaterial3d(material),
        Transform::from_xyz(x, SPAWN_Y, PLAY_Z).with_scale(Vec3::splat(scale)),
        Projectile {
            velocity: Vec3::new(vx, vy, 0.0),
            spin,
        },
    ));

    if is_bomb {
        object.insert(Bomb);
    } else if is_golden {
        object.insert(Golden);
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
        transform.rotate_local_x(dt * motion.spin.x);
        transform.rotate_local_y(dt * motion.spin.y);

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
    time: Res<Time>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    player: Single<Entity, With<Player>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut trail: ResMut<CursorTrail>,
    mut blade: ResMut<BladeTrail>,
    mut combo: ResMut<Combo>,
    mut shake: ResMut<CameraShake>,
    mut score: ResMut<Score>,
    q_sliceable: Query<(Entity, &Transform, &Sliceable, Has<Bomb>, Has<Golden>)>,
) {
    // Releasing the button ends the swipe, so the next press starts a fresh
    // segment instead of jumping across the screen from a stale point, and the
    // blade trail is cleared so it does not linger. The combo is NOT reset here:
    // it lives on its own window (see `tick_combo`) so it can span strokes.
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
    // trail is at its cap. The trail is drawn regardless of swipe speed.
    blade.points.push_back(current);
    while blade.points.len() > BLADE_TRAIL_LEN {
        blade.points.pop_front();
    }

    // The swipe segment runs from last frame's cursor to this frame's. On the
    // first frame of a press there is no previous point yet, so nothing is
    // sliced until the cursor has actually moved.
    let previous = trail.previous;
    trail.previous = Some(current);
    let Some(previous) = previous else {
        return;
    };

    // A slice only counts while the cursor is genuinely swiping. Holding still
    // or crawling stalls the swipe: it slices nothing, so the button cannot be
    // held down to farm points. The combo is left alone here -- its window keeps
    // it alive across a brief stall so you can re-swipe and continue the chain.
    if !swipe_is_active(previous, current, time.delta_secs()) {
        return;
    }

    for (entity, transform, sliceable, is_bomb, is_golden) in q_sliceable.iter() {
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
            .remove::<Projectile>();

        if is_bomb {
            // A bomb explodes instantly and is an instant loss: deal lethal
            // damage to the player, which trips HealthZeroMarker -> GameOver.
            commands.entity(entity).insert(ExplodeMesh {
                fragment_count: FRAGMENT_COUNT,
            });
            commands.trigger(HealthApplyDamage {
                entity: *player,
                source: Some(entity),
                amount: PLAYER_HEALTH,
            });
        } else {
            // Fruit pops (scales up briefly) before it bursts, so the cut reads
            // as impactful; `resolve_slice_pop` inserts ExplodeMesh when done.
            commands.entity(entity).insert(SlicePop {
                timer: SLICE_POP_TIME,
                base_scale: transform.scale,
            });
            shake.trauma = (shake.trauma + SLICE_TRAUMA).min(1.0);

            let viewport_pos = camera
                .world_to_viewport(camera_transform, transform.translation)
                .ok();

            if is_golden {
                // Golden fruit: flat bonus, and it buys extra combo time by
                // stretching the window, without advancing the combo count.
                **score += GOLDEN_POINTS;
                // Only fold into the combo tally when a combo is actually
                // running, otherwise `points` would leak (tick_combo only
                // clears it when a counted combo ends).
                if combo.count > 0 {
                    combo.points += GOLDEN_POINTS;
                }
                combo.timer = combo.timer.max(COMBO_WINDOW_GOLDEN);
                if let Some(viewport_pos) = viewport_pos {
                    spawn_floating_text(
                        &mut commands,
                        viewport_pos,
                        format!("+{GOLDEN_POINTS}"),
                        48.0,
                        Color::srgb(1.0, 0.85, 0.2),
                    );
                }
            } else {
                // Each fruit in the combo is worth one more point than the last
                // (1, 2, 3, ...); the combo window keeps the chain alive.
                let points = advance_combo(&mut combo);
                **score += points;

                if let Some(viewport_pos) = viewport_pos {
                    // The "+N" grows a little with the combo for extra punch.
                    let size = (30.0 + (points as f32 - 1.0) * 5.0).min(60.0);
                    spawn_floating_text(
                        &mut commands,
                        viewport_pos,
                        format!("+{points}"),
                        size,
                        Color::srgb(0.95, 0.85, 0.25),
                    );

                    // A multi-fruit combo reads as special: a flashy banner.
                    if combo.count >= 2 {
                        spawn_floating_text(
                            &mut commands,
                            viewport_pos - Vec2::Y * 44.0,
                            format!("COMBO x{}", combo.count),
                            48.0,
                            Color::srgb(1.0, 0.55, 0.1),
                        );
                    }
                }
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

    // Half-width of the blade at the head, in world units; tapers to 0 at tail.
    const BLADE_WIDTH: f32 = 0.22;

    for (i, (a, b)) in blade
        .points
        .iter()
        .zip(blade.points.iter().skip(1))
        .enumerate()
    {
        // t ramps 0 -> 1 from tail to head; alpha and width follow so the blade
        // looks like a bright edge trailing to a thin tail.
        let t = (i + 1) as f32 / (count - 1) as f32;
        let a = *a + lift;
        let b = *b + lift;

        // Perpendicular to the segment on the play plane, scaled by the taper.
        let dir = b - a;
        let perp = if dir.length_squared() > f32::EPSILON {
            Vec3::new(-dir.y, dir.x, 0.0).normalize() * BLADE_WIDTH * t
        } else {
            Vec3::ZERO
        };

        // Cyan edges flanking a hot white core, all fading toward the tail.
        let edge = Color::srgba(0.4, 0.85, 1.0, t * 0.6);
        let core = Color::srgba(0.9, 0.98, 1.0, t);
        gizmos.line(a + perp, b + perp, edge);
        gizmos.line(a - perp, b - perp, edge);
        gizmos.line(a, b, core);
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
    // Match the fruit's size so a big fruit bursts into big fragments (the
    // fragment meshes are sliced from the unit mesh, ignoring the shell scale).
    let scale = transform.scale;
    // The sliced shell still carries its material, so fragments burst in the
    // same color as the fruit they came from.
    let material = material.0.clone();

    for fragment in fragments.iter() {
        commands.spawn((
            Name::new("Fragment"),
            DespawnOnExit(GameState::Playing),
            Mesh3d(fragment.mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(origin).with_scale(scale),
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

/// True while the cursor is moving fast enough (>= `MIN_SWIPE_SPEED`) to count
/// as an active swipe. Holding still or crawling is not a swipe, so it neither
/// slices nor builds a combo -- this is what stops a held button from farming.
fn swipe_is_active(previous: Vec3, current: Vec3, dt: f32) -> bool {
    dt > 0.0 && (current - previous).length() / dt >= MIN_SWIPE_SPEED
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

    #[test]
    fn combo_escalates_within_a_swipe() {
        // The k-th fruit in one swipe earns k points: 1, 2, 3, ...
        let mut combo = Combo::default();
        assert_eq!(advance_combo(&mut combo), 1);
        assert_eq!(advance_combo(&mut combo), 2);
        assert_eq!(advance_combo(&mut combo), 3);
        assert_eq!(combo.count, 3);
    }

    #[test]
    fn combo_resets_after_window_expires() {
        // When the window runs out (count reset to 0), the chain starts fresh.
        let mut combo = Combo::default();
        advance_combo(&mut combo);
        advance_combo(&mut combo);
        combo.count = 0; // window expired (see tick_combo)
        assert_eq!(advance_combo(&mut combo), 1);
    }

    #[test]
    fn advancing_combo_refreshes_the_window() {
        // Each slice refreshes the full window so the chain can continue.
        let mut combo = Combo::default();
        combo.timer = 0.1;
        advance_combo(&mut combo);
        assert!((combo.timer - COMBO_WINDOW).abs() < 1e-6);
    }

    #[test]
    fn combo_accumulates_tally_points() {
        // A 3-hit combo tallies 1 + 2 + 3 = 6 points for the end summary.
        let mut combo = Combo::default();
        advance_combo(&mut combo);
        advance_combo(&mut combo);
        advance_combo(&mut combo);
        assert_eq!(combo.points, 6);
    }

    #[test]
    fn fast_motion_is_an_active_swipe() {
        // Moving one world unit in a 60 fps frame is ~60 units/s, well over the
        // threshold.
        let dt = 1.0 / 60.0;
        assert!(swipe_is_active(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), dt));
    }

    #[test]
    fn holding_still_is_not_a_swipe() {
        let dt = 1.0 / 60.0;
        assert!(!swipe_is_active(Vec3::ZERO, Vec3::ZERO, dt));
    }

    #[test]
    fn slow_crawl_is_not_a_swipe() {
        // ~0.05 units in a 60 fps frame is ~3 units/s, below MIN_SWIPE_SPEED.
        let dt = 1.0 / 60.0;
        assert!(!swipe_is_active(Vec3::ZERO, Vec3::new(0.05, 0.0, 0.0), dt));
    }

    #[test]
    fn zero_dt_is_not_a_swipe() {
        // Guards the division; a zero-length frame is never an active swipe.
        assert!(!swipe_is_active(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), 0.0));
    }

    #[test]
    fn difficulty_ramp_endpoints() {
        // At t=0 the game is at the easy start values.
        assert!((spawn_interval_for(0.0) - SPAWN_INTERVAL).abs() < 1e-6);
        assert!((bomb_chance_for(0.0) - BOMB_CHANCE_START).abs() < 1e-6);

        // At/after the ramp duration it is clamped to the hard floor/cap.
        assert!(
            (spawn_interval_for(DIFFICULTY_RAMP_SECS * 2.0) - SPAWN_INTERVAL_FLOOR).abs() < 1e-6
        );
        assert!((bomb_chance_for(DIFFICULTY_RAMP_SECS * 2.0) - BOMB_CHANCE_CAP).abs() < 1e-6);
    }

    #[test]
    fn difficulty_ramp_is_monotonic_midway() {
        // Halfway through, spawn interval is shorter and bombs more likely.
        let mid = DIFFICULTY_RAMP_SECS / 2.0;
        assert!(spawn_interval_for(mid) < SPAWN_INTERVAL);
        assert!(spawn_interval_for(mid) > SPAWN_INTERVAL_FLOOR);
        assert!(bomb_chance_for(mid) > BOMB_CHANCE_START);
        assert!(bomb_chance_for(mid) < BOMB_CHANCE_CAP);
    }
}
