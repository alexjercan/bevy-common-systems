//! 14_breach -- a grounded first-person arena shooter, the gallery's first
//! first-person game.
//!
//! It is the headline demo of three things the crate did not show before: the
//! first-person viewpoint as a real game, the first use of avian3d's
//! [`SpatialQuery`] raycasting (the hitscan gun), and a game-local first-person
//! character controller. The crate's [`camera/wasd`](bevy_common_systems::camera)
//! is a free-fly spectator camera (no gravity, ground, collision or cursor grab),
//! so a grounded shooter needs its own controller: here the player is an avian
//! `RigidBody::Dynamic` capsule with locked rotation, driven by writing
//! `LinearVelocity`, so avian's solver does collide-and-slide against the static
//! level for free (a kinematic body would not be pushed back by walls). Look is
//! always-on yaw/pitch from a grabbed mouse with a pitch clamp; the `Camera3d`
//! rides at eye height as a child of the body.
//!
//! You spawn in a walled arena; waves of glowing enemies close in from every
//! side and melee you. Left-click fires a hitscan ray -- the first enemy collider
//! in the crosshair takes damage (`HealthPlugin`), flashes (`feedback/flash`) and
//! on death bursts into physics gibs (`mesh/explode`). Clearing a wave spawns a
//! bigger, faster one. A hit spikes a red damage vignette (`feedback/screen_flash`)
//! and kicks the camera (`camera/shake`); zero health ends the run. Kills chained
//! inside a short window build a combo (`scoring/streak`) that multiplies the points
//! they are worth, floats a "+N" and flashes a "COMBO xN" tally (`ui/popup`); the
//! points score is saved across launches (`persist` + `HighScore`).
//!
//! Controls: move with WASD, look with the mouse, fire with left-click, Escape
//! gives up. The mouse is captured on start and released on the menu / game-over
//! screens. Touch (the wasm build): drag the left half to move, the right half to
//! look, and tap the FIRE button -- clunky, an FPS is the hardest genre for touch,
//! so desktop mouse+keyboard is the primary path.
//!
//! Run it: `cargo run --example 14_breach` (add `--features debug` for the
//! inspector and the headless harness).

use std::sync::Arc;

use avian3d::prelude::*;
use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorOptions, PrimaryWindow},
};
use bevy_common_systems::prelude::*;
use clap::Parser;
use rand::Rng;

#[derive(Parser)]
#[command(name = "14_breach")]
#[command(version = "1.0.0")]
#[command(
    about = "A grounded first-person arena shooter: survive waves of enemies.",
    long_about = None
)]
struct Cli;

// --- Tuning -----------------------------------------------------------------

/// Half the arena's side length, in metres (walls sit at +/- this).
const ARENA_HALF: f32 = 18.0;
/// Wall height / thickness.
const WALL_H: f32 = 5.0;
const WALL_T: f32 = 1.0;

/// Player capsule radius and cylinder length (total height = 2r + len).
const PLAYER_R: f32 = 0.4;
const PLAYER_LEN: f32 = 1.0;
/// Resting height of the capsule centre above the floor.
const PLAYER_CENTER_Y: f32 = PLAYER_R + PLAYER_LEN * 0.5;
/// Camera height above the capsule centre (eye height ~1.4m).
const EYE_H: f32 = 0.55;
/// Walk speed, m/s.
const PLAYER_SPEED: f32 = 6.5;
/// Player hit points.
const PLAYER_HEALTH: f32 = 100.0;

/// Radians of view rotation per pixel of mouse motion.
const LOOK_SENS: f32 = 0.0022;
/// Pitch is clamped to +/- this (just under 90deg) so the view cannot flip.
const PITCH_LIMIT: f32 = 1.54;

/// Downward gravity.
const GRAVITY: f32 = 22.0;

/// Gun: seconds between shots, ray length, and damage per hit.
const GUN_COOLDOWN: f32 = 0.14;
const GUN_RANGE: f32 = 120.0;
const GUN_DAMAGE: f32 = 12.0;

/// Scoring: each kill is worth `BASE_KILL_POINTS * streak`, so chaining kills
/// inside `COMBO_WINDOW` seconds ramps the payout (streak 1 -> 10, 2 -> 20, ...).
const BASE_KILL_POINTS: u32 = 10;
/// Seconds after a kill you have to land the next one to keep the combo alive.
/// Longer than fruitninja's swipe combo because FPS kills are further apart.
const COMBO_WINDOW: f32 = 2.5;

/// Enemy capsule, health, base speed (ramped per wave), melee reach, attack
/// cadence and damage.
const ENEMY_R: f32 = 0.5;
const ENEMY_LEN: f32 = 0.5;
const ENEMY_HEALTH: f32 = 30.0;
const ENEMY_SPEED_BASE: f32 = 2.4;
const ENEMY_SPEED_PER_WAVE: f32 = 0.35;
const MELEE_RANGE: f32 = 1.7;
/// Damage per second each enemy within `MELEE_RANGE` drains from the player.
const ENEMY_DPS: f32 = 16.0;

/// Enemies spawn on a ring this far from the centre.
const SPAWN_RADIUS: f32 = ARENA_HALF - 2.5;

// --- App --------------------------------------------------------------------

fn main() {
    let _ = Cli::parse();
    let mut app = App::new();

    let primary_window = Window {
        title: "14_breach".into(),
        #[cfg(target_arch = "wasm32")]
        canvas: Some("#game-canvas".into()),
        #[cfg(target_arch = "wasm32")]
        fit_canvas_to_parent: true,
        ..default()
    };
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(primary_window),
        ..default()
    }));

    app.add_plugins(PhysicsPlugins::default());
    app.insert_resource(Gravity(Vec3::new(0.0, -GRAVITY, 0.0)));

    #[cfg(feature = "debug")]
    app.add_plugins(InspectorDebugPlugin);

    // Headless verification harness (dev tooling, `debug` feature). Inert unless
    // BCS_AUTOPILOT / BCS_SHOT is set; see `docs/dev-harness.md`.
    #[cfg(feature = "debug")]
    {
        app.add_plugins(
            AutopilotPlugin::new()
                .hold(GameState::Menu, 0.6)
                .hold(GameState::Playing, 4.0)
                .hold(GameState::GameOver, 0.8)
                .input(|world, _elapsed| {
                    if *world.resource::<State<GameState>>().get() != GameState::Playing {
                        return;
                    }
                    // Aim at the nearest enemy and fire. The look system can't be
                    // driven by injected mouse motion, so set the controller yaw
                    // directly here -- this exercises the real raycast -> damage ->
                    // kill path, then the enemies close in and end the run.
                    let mut player_q = world.query_filtered::<&Transform, With<Player>>();
                    let Ok(ppos) = player_q.single(world).map(|t| t.translation) else {
                        return;
                    };
                    let mut enemy_q = world.query_filtered::<&Transform, With<Enemy>>();
                    let nearest = enemy_q.iter(world).map(|t| t.translation).min_by(|a, b| {
                        a.distance_squared(ppos)
                            .total_cmp(&b.distance_squared(ppos))
                    });
                    if let Some(epos) = nearest {
                        let dir = epos - ppos;
                        let yaw = (-dir.x).atan2(-dir.z);
                        let mut ctrl_q =
                            world.query_filtered::<&mut DoomControllerState, With<Player>>();
                        if let Ok(mut c) = ctrl_q.single_mut(world) {
                            c.yaw = yaw;
                            c.pitch = 0.0;
                        }
                    }
                    world
                        .resource_mut::<ButtonInput<MouseButton>>()
                        .press(MouseButton::Left);
                }),
        );
        app.add_plugins(ScreenshotPlugin::new(GameState::Playing).settle_frames(30));
    }

    if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default());
    }

    app.add_plugins(HealthPlugin);
    app.add_plugins(SfxPlugin);
    app.add_plugins(FlashPlugin);
    app.add_plugins(ScreenFlashPlugin);
    app.add_plugins(CameraShakePlugin);
    app.add_plugins(PostProcessingDefaultPlugin);
    app.add_plugins(ExplodeMeshPlugin);
    app.add_plugins(TempEntityPlugin);
    app.add_plugins(StatusBarPlugin);
    app.add_plugins(TouchpadPlugin);
    app.add_plugins(PopupPlugin);
    app.add_plugins(DoomControllerPlugin);
    app.add_plugins(PersistPlugin::<HighScore<u32>>::new("14_breach.high_score"));

    app.insert_resource(ClearColor(Color::srgb(0.02, 0.03, 0.05)));
    app.init_resource::<Score>();
    app.init_resource::<Combo>();
    app.init_resource::<KillFeed>();
    app.init_resource::<Wave>();
    app.init_resource::<PlayerHp>();
    app.init_resource::<RunOver>();
    app.init_resource::<TouchInput>();

    app.init_state::<GameState>();

    app.add_systems(Startup, setup);

    // Menu.
    app.add_systems(OnEnter(GameState::Menu), spawn_menu);
    app.add_systems(
        Update,
        (menu_start, pulse_menu_title).run_if(in_state(GameState::Menu)),
    );

    // Playing.
    app.add_systems(
        OnEnter(GameState::Playing),
        (start_run, spawn_scene, spawn_hud, capture_cursor).chain(),
    );
    app.add_systems(
        Update,
        (
            // Feed the controller's input BEFORE it runs (its Drive set), then apply
            // its velocity output AFTER -- otherwise the controller reads last frame's
            // look/move (a one-frame lag).
            (read_touch, feed_look, feed_move)
                .chain()
                .before(DoomControllerSystems::Drive),
            apply_move_velocity.after(DoomControllerSystems::Drive),
            player_shoot,
            drive_enemies,
            enemy_melee,
            advance_waves,
            mirror_player_hp,
            (spawn_kill_popups, tick_combo, update_combo_text),
            check_run_over,
            set_state_on_key(KeyCode::Escape, GameState::GameOver),
        )
            .run_if(in_state(GameState::Playing)),
    );
    app.add_systems(OnExit(GameState::Playing), free_cursor);

    // Game over.
    app.add_systems(
        OnEnter(GameState::GameOver),
        (record_high_score, spawn_game_over, play_game_over_sfx).chain(),
    );
    app.add_systems(
        Update,
        gameover_dismiss.run_if(in_state(GameState::GameOver)),
    );

    app.add_observer(on_health_zero);
    app.add_observer(on_fragments_spawned);

    app.run();
}

// --- State ------------------------------------------------------------------

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

/// Physics layers: the ray and colliders filter on these.
#[derive(PhysicsLayer, Default, Clone, Copy)]
enum GameLayer {
    #[default]
    Default,
    World,
    Player,
    Enemy,
}

// --- Sounds -----------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Sfx {
    Shoot,
    Hit,
    EnemyDown,
    Hurt,
    Wave,
    Select,
    GameOver,
}

// --- Resources --------------------------------------------------------------

/// Run score. `points` is the combo-scaled tally (the persisted high score);
/// `kills` is the raw body count, kept for the game-over readout.
#[derive(Resource, Default)]
struct Score {
    points: u32,
    kills: u32,
}

/// The decaying kill combo plus the points earned during the current window
/// (shown in the "COMBO xN +P" tally when it lapses).
#[derive(Resource)]
struct Combo {
    streak: Streak,
    window_points: u32,
}

impl Default for Combo {
    fn default() -> Self {
        Self {
            streak: Streak::new(COMBO_WINDOW),
            window_points: 0,
        }
    }
}

/// Points earned by kills this frame, drained by `spawn_kill_popups` into "+N"
/// popups. Decouples the (headlessly testable) death observer from the UI.
#[derive(Resource, Default)]
struct KillFeed(Vec<u32>);

#[derive(Resource, Default)]
struct Wave {
    number: u32,
    alive: u32,
}

/// Player health mirrored for the HUD status item (read from `&World`).
#[derive(Resource)]
struct PlayerHp {
    current: f32,
    max: f32,
}

impl Default for PlayerHp {
    fn default() -> Self {
        Self {
            current: PLAYER_HEALTH,
            max: PLAYER_HEALTH,
        }
    }
}

/// Set by the player-death observer; a Playing system reads it to end the run.
#[derive(Resource, Default)]
struct RunOver(bool);

/// Touch controls distilled each frame (additive to keyboard/mouse).
#[derive(Resource, Default)]
struct TouchInput {
    move_vec: Vec2,
    look_delta: Vec2,
    fire: bool,
    /// Per-finger start positions, to classify move (left) vs look (right).
    move_finger: Option<(u64, Vec2)>,
    look_finger: Option<(u64, Vec2)>,
}

// --- Components --------------------------------------------------------------

#[derive(Component)]
struct Player;

/// The gun's fire-rate gate.
#[derive(Component)]
struct Gun {
    cooldown: Cooldown,
}

#[derive(Component)]
struct Enemy {
    speed: f32,
}

#[derive(Component)]
struct DamageVignette;

#[derive(Component)]
struct MenuTitle;

// --- Pure logic (unit-tested) ----------------------------------------------

/// Enemies in wave `n` (0-based): a steady ramp.
fn wave_size(n: u32) -> u32 {
    3 + 2 * n
}

/// `count` evenly spaced points on a horizontal ring of `radius` at floor level.
fn ring_positions(count: u32, radius: f32) -> Vec<Vec3> {
    (0..count)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / count.max(1) as f32;
            Vec3::new(radius * a.cos(), PLAYER_CENTER_Y, radius * a.sin())
        })
        .collect()
}

fn enemy_speed(wave: u32) -> f32 {
    ENEMY_SPEED_BASE + ENEMY_SPEED_PER_WAVE * wave as f32
}

// --- Setup ------------------------------------------------------------------

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SoundBank::load(
        &asset_server,
        [
            (Sfx::Shoot, "launch"),
            (Sfx::Hit, "pickup"),
            (Sfx::EnemyDown, "combo"),
            (Sfx::Hurt, "hurt"),
            (Sfx::Wave, "level_up"),
            (Sfx::Select, "menu_select"),
            (Sfx::GameOver, "game_over"),
        ],
    ));

    // A sun so the scene is lit; ambient fill lives on the camera (0.19).
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 9000.0,
            ..default()
        },
        Transform::from_xyz(20.0, 40.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn best_line(best: u32) -> String {
    if best > 0 {
        format!("Best: {best}")
    } else {
        "No run yet".to_string()
    }
}

// --- Menu -------------------------------------------------------------------

fn spawn_menu(mut commands: Commands, high: Res<HighScore<u32>>) {
    commands
        .spawn((
            Name::new("Menu"),
            DespawnOnExit(GameState::Menu),
            centered_screen(),
        ))
        .with_children(|parent| {
            parent.spawn((
                MenuTitle,
                screen_text("BREACH", 84.0, Color::srgb(1.0, 0.5, 0.3)),
            ));
            parent.spawn(screen_text(
                "FIRST-PERSON ARENA SHOOTER",
                26.0,
                Color::srgb(0.8, 0.82, 0.9),
            ));
            parent.spawn(screen_text(
                "Enemies close in from every side. Hold the line.",
                20.0,
                Color::srgb(0.7, 0.75, 0.85),
            ));
            parent.spawn(screen_text(
                "WASD to move, mouse to look, left-click to fire. Escape gives up.",
                18.0,
                Color::srgb(0.62, 0.67, 0.77),
            ));
            parent.spawn(screen_text(
                best_line(high.best()),
                24.0,
                Color::srgb(0.95, 0.85, 0.35),
            ));
            parent.spawn(screen_text(
                "Click or press any key to begin",
                24.0,
                Color::srgb(0.9, 0.9, 0.9),
            ));
        });
}

fn pulse_menu_title(time: Res<Time>, mut q: Query<&mut TextColor, With<MenuTitle>>) {
    let t = (time.elapsed_secs() * 2.4).sin() * 0.5 + 0.5;
    let b = 0.55 + 0.45 * t;
    for mut color in &mut q {
        color.0 = Color::srgb(b, 0.5 * b, 0.3 * b);
    }
}

fn menu_start(
    mut commands: Commands,
    sfx: Res<SoundBank<Sfx>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut next: ResMut<NextState<GameState>>,
) {
    let pressed = mouse.just_pressed(MouseButton::Left)
        || keys.get_just_pressed().next().is_some()
        || touches.any_just_pressed();
    if pressed {
        commands.play_sfx_volume(sfx.get(Sfx::Select), 0.7);
        next.set(GameState::Playing);
    }
}

// --- Run start / scene ------------------------------------------------------

fn start_run(
    mut score: ResMut<Score>,
    mut combo: ResMut<Combo>,
    mut feed: ResMut<KillFeed>,
    mut wave: ResMut<Wave>,
    mut hp: ResMut<PlayerHp>,
    mut over: ResMut<RunOver>,
) {
    *score = Score::default();
    *combo = Combo::default();
    feed.0.clear();
    *wave = Wave::default();
    *hp = PlayerHp::default();
    over.0 = false;
}

fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let floor_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.13, 0.17),
        perceptual_roughness: 0.95,
        ..default()
    });
    let wall_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.2, 0.26),
        perceptual_roughness: 0.9,
        ..default()
    });

    let world_layers =
        // World collides with players, enemies AND gibs (Default), so shards do not
        // fall through the floor.
        CollisionLayers::new(
            [GameLayer::World],
            [GameLayer::Player, GameLayer::Enemy, GameLayer::Default],
        );

    // Floor (top surface at y = 0).
    commands.spawn((
        Name::new("Floor"),
        DespawnOnExit(GameState::Playing),
        RigidBody::Static,
        Collider::cuboid(ARENA_HALF * 2.0, 1.0, ARENA_HALF * 2.0),
        Mesh3d(meshes.add(Cuboid::new(ARENA_HALF * 2.0, 1.0, ARENA_HALF * 2.0))),
        MeshMaterial3d(floor_mat),
        world_layers,
        Transform::from_xyz(0.0, -0.5, 0.0),
    ));

    // Four perimeter walls.
    let walls = [
        (
            Vec3::new(0.0, WALL_H * 0.5, ARENA_HALF),
            Vec3::new(ARENA_HALF * 2.0, WALL_H, WALL_T),
        ),
        (
            Vec3::new(0.0, WALL_H * 0.5, -ARENA_HALF),
            Vec3::new(ARENA_HALF * 2.0, WALL_H, WALL_T),
        ),
        (
            Vec3::new(ARENA_HALF, WALL_H * 0.5, 0.0),
            Vec3::new(WALL_T, WALL_H, ARENA_HALF * 2.0),
        ),
        (
            Vec3::new(-ARENA_HALF, WALL_H * 0.5, 0.0),
            Vec3::new(WALL_T, WALL_H, ARENA_HALF * 2.0),
        ),
    ];
    for (pos, size) in walls {
        commands.spawn((
            Name::new("Wall"),
            DespawnOnExit(GameState::Playing),
            RigidBody::Static,
            Collider::cuboid(size.x, size.y, size.z),
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(wall_mat.clone()),
            world_layers,
            Transform::from_translation(pos),
        ));
    }

    // The arena is deliberately OPEN (floor + perimeter walls, no interior cover):
    // enemies path in a straight line toward the player with no obstacle avoidance,
    // so any interior block would sit on some enemy's radial path and leave it stuck.
    // An open arena keeps the swarm a reliable threat; the player kites in the open.

    // The player: a dynamic capsule with locked rotation (the solver does
    // collide-and-slide), and the eye camera as a child.
    let player = commands
        .spawn((
            Name::new("Player"),
            Player,
            // The crate's Doom-style FP controller (look + planar-move math); the
            // game feeds it input and applies its velocity output below.
            DoomController {
                move_speed: PLAYER_SPEED,
                look_sensitivity: LOOK_SENS,
                pitch_min: -PITCH_LIMIT,
                pitch_max: PITCH_LIMIT,
            },
            Gun {
                cooldown: Cooldown::new(GUN_COOLDOWN),
            },
            Health::new(PLAYER_HEALTH),
            RigidBody::Dynamic,
            Collider::capsule(PLAYER_R, PLAYER_LEN),
            LockedAxes::ROTATION_LOCKED,
            // The player collides with the world only, NOT enemies: dynamic-vs-dynamic
            // knockback would fling approaching enemies out of melee range, so instead
            // enemies overlap the player and the distance-based melee lands reliably.
            CollisionLayers::new([GameLayer::Player], [GameLayer::World]),
            Visibility::default(),
            // Spawn at the arena centre; enemies converge from the ring around you.
            Transform::from_xyz(0.0, PLAYER_CENTER_Y + 0.3, 0.0),
            DespawnOnExit(GameState::Playing),
        ))
        .id();

    commands.entity(player).with_children(|p| {
        p.spawn((
            Name::new("Eye Camera"),
            DoomEye,
            Camera3d::default(),
            Transform::from_xyz(0.0, EYE_H, 0.0),
            PostProcessingCamera,
            CameraShake {
                decay: 3.5,
                max_offset: Vec3::splat(0.18),
                ..default()
            },
            AmbientLight {
                color: Color::srgb(0.6, 0.65, 0.8),
                brightness: 130.0,
                ..default()
            },
        ));
    });
}

// --- HUD --------------------------------------------------------------------

fn spawn_hud(mut commands: Commands) {
    // Status bar: health, wave, score, fps.
    commands.spawn((
        status_bar(StatusBarRootConfig::default()),
        DespawnOnExit(GameState::Playing),
    ));
    commands.spawn((
        DespawnOnExit(GameState::Playing),
        status_bar_item(StatusBarItemConfig {
            icon: None,
            value_fn: |world: &World| {
                world
                    .get_resource::<PlayerHp>()
                    .map(|hp| Arc::new(hp.current.max(0.0).round() as u32) as Arc<dyn StatusValue>)
            },
            color_fn: |_| Some(Color::srgb(0.4, 1.0, 0.5)),
            prefix: "HP".to_string(),
            suffix: "".to_string(),
        }),
    ));
    commands.spawn((
        DespawnOnExit(GameState::Playing),
        status_bar_item(StatusBarItemConfig {
            icon: None,
            value_fn: |world: &World| {
                world
                    .get_resource::<Wave>()
                    .map(|w| Arc::new(w.number.max(1)) as Arc<dyn StatusValue>)
            },
            color_fn: |_| Some(Color::srgb(0.7, 0.8, 1.0)),
            prefix: "WAVE".to_string(),
            suffix: "".to_string(),
        }),
    ));
    commands.spawn((
        DespawnOnExit(GameState::Playing),
        status_bar_item(StatusBarItemConfig {
            icon: None,
            value_fn: |world: &World| {
                world
                    .get_resource::<Score>()
                    .map(|s| Arc::new(s.points) as Arc<dyn StatusValue>)
            },
            color_fn: |_| Some(Color::srgb(0.95, 0.85, 0.3)),
            prefix: "SCORE".to_string(),
            suffix: "".to_string(),
        }),
    ));
    commands.spawn((status_bar_with_fps(), DespawnOnExit(GameState::Playing)));

    // Centre crosshair.
    commands
        .spawn((
            Name::new("Crosshair root"),
            DespawnOnExit(GameState::Playing),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                Node {
                    width: Val::Px(6.0),
                    height: Val::Px(6.0),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
            ));
        });

    // Live combo readout, centred just above the crosshair; hidden until a
    // streak of 2+ is running (driven by `update_combo_text`).
    commands.spawn((
        Name::new("Combo readout"),
        ComboText,
        DespawnOnExit(GameState::Playing),
        Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            top: Val::Percent(38.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(34.0),
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.6, 0.2)),
        TextLayout {
            justify: Justify::Center,
            ..default()
        },
    ));

    // Persistent damage vignette overlay (re-spiked on hit).
    commands.spawn((
        Name::new("Damage vignette"),
        DamageVignette,
        DespawnOnExit(GameState::Playing),
        screen_flash_node(),
        BackgroundColor(Color::srgba(0.8, 0.05, 0.05, 0.0)),
        GlobalZIndex(10),
    ));

    // On-screen fire button (revealed on first touch) for the wasm build.
    commands
        .spawn((
            Name::new("Fire button"),
            DespawnOnExit(GameState::Playing),
            RevealOnTouch,
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Percent(6.0),
                bottom: Val::Percent(12.0),
                width: Val::Px(96.0),
                height: Val::Px(96.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 0.4, 0.3, 0.35)),
        ))
        .with_children(|p| {
            p.spawn(screen_text("FIRE", 22.0, Color::srgb(1.0, 0.95, 0.9)));
        });
}

// --- Cursor -----------------------------------------------------------------

/// True while a headless verification run drives the game; skip cursor grab then
/// so the locked pointer does not interfere.
fn headless() -> bool {
    std::env::var("BCS_AUTOPILOT").is_ok() || std::env::var("BCS_SHOT").is_ok()
}

/// Capture the mouse for looking, unless a headless verification run is driving the
/// game (a locked pointer would interfere). Uses the crate's `input/cursor` helper.
fn capture_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    if headless() {
        return;
    }
    grab_cursor(&mut cursor);
}

fn free_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    release_cursor(&mut cursor);
}

// --- Input -> controller ----------------------------------------------------
//
// The crate's `DoomController` owns the look accumulation (+ pitch clamp, writing
// the DoomEye child) and the move math; the game just feeds it input each frame and
// applies its velocity output to the avian body.

fn feed_look(
    motion: Res<AccumulatedMouseMotion>,
    touch: Res<TouchInput>,
    mut input: Single<&mut DoomControllerInput, With<Player>>,
) {
    input.look = motion.delta + touch.look_delta;
}

fn feed_move(
    keys: Res<ButtonInput<KeyCode>>,
    touch: Res<TouchInput>,
    mut input: Single<&mut DoomControllerInput, With<Player>>,
) {
    let mut intent = touch.move_vec;
    if keys.pressed(KeyCode::KeyW) {
        intent.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        intent.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        intent.x += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        intent.x -= 1.0;
    }
    input.movement = intent;
}

/// Apply the controller's velocity output to the body, leaving `.y` to gravity so
/// avian's solver does collide-and-slide. Runs after `DoomControllerSystems::Drive`.
fn apply_move_velocity(player: Single<(&DoomControllerOutput, &mut LinearVelocity), With<Player>>) {
    let (output, mut vel) = player.into_inner();
    vel.0.x = output.velocity.x;
    vel.0.z = output.velocity.z;
}

// --- Gun --------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn player_shoot(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    touch: Res<TouchInput>,
    spatial: SpatialQuery,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sfx: Res<SoundBank<Sfx>>,
    player: Single<(Entity, &mut Gun), With<Player>>,
    cam: Single<&GlobalTransform, With<DoomEye>>,
    enemies: Query<(), With<Enemy>>,
    mut shake: Query<&mut CameraShakeInput>,
) {
    let (player_entity, mut gun) = player.into_inner();
    gun.cooldown.tick(time.delta_secs());

    let firing = mouse.pressed(MouseButton::Left) || touch.fire;
    if !firing || !gun.cooldown.ready() {
        return;
    }
    gun.cooldown.trigger();

    let origin = cam.translation();
    let Ok(dir) = Dir3::new(cam.forward().as_vec3()) else {
        return;
    };
    let filter = SpatialQueryFilter::from_mask([GameLayer::Enemy, GameLayer::World])
        .with_excluded_entities([player_entity]);

    let hit = spatial.cast_ray(origin, dir, GUN_RANGE, true, &filter);
    let end = match hit {
        Some(h) => origin + dir.as_vec3() * h.distance,
        None => origin + dir.as_vec3() * GUN_RANGE,
    };

    // Damage + flash if we hit an enemy. Flash BEFORE the damage trigger: a lethal
    // hit runs the death chain (HealthZeroMarker -> ExplodeMesh -> fragments ->
    // despawn) during the command flush, so a Flash queued after would land on a
    // despawned entity.
    if let Some(h) = hit {
        if enemies.contains(h.entity) {
            commands.entity(h.entity).insert(Flash {
                color: Color::srgb(1.0, 0.9, 0.6),
                duration: 0.18,
                ..default()
            });
            commands.trigger(HealthApplyDamage {
                entity: h.entity,
                source: Some(player_entity),
                amount: GUN_DAMAGE,
            });
            commands.play_sfx_volume(sfx.get(Sfx::Hit), 0.35);
        }
    }

    // Tracer: a thin glowing beam from just below the muzzle to the hit point.
    let muzzle = origin + dir.as_vec3() * 0.3 - Vec3::Y * 0.1;
    let len = (end - muzzle).length().max(0.1);
    let mid = (muzzle + end) * 0.5;
    let tracer_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.85, 0.4),
        emissive: LinearRgba::rgb(6.0, 4.0, 1.0),
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.03, 0.03, len))),
        MeshMaterial3d(tracer_mat),
        Transform::from_translation(mid).looking_at(end, Vec3::Y),
        TempEntity(0.05),
        DespawnOnExit(GameState::Playing),
    ));

    commands.play_sfx_volume(sfx.get(Sfx::Shoot), 0.3);
    if let Ok(mut input) = shake.single_mut() {
        input.add_trauma += 0.12;
    }
}

// --- Enemies ----------------------------------------------------------------

fn spawn_wave(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    wave: &mut Wave,
) {
    let count = wave_size(wave.number);
    let speed = enemy_speed(wave.number);
    // Enemies collide with the world and each other (so they spread around you),
    // but NOT the player (see the player's layers) -- they overlap you and melee.
    let enemy_layers =
        CollisionLayers::new([GameLayer::Enemy], [GameLayer::World, GameLayer::Enemy]);

    let mut rng = rand::rng();
    for base in ring_positions(count, SPAWN_RADIUS) {
        // Jitter so they do not stack perfectly.
        let pos = base
            + Vec3::new(
                rng.random_range(-1.5..1.5),
                0.0,
                rng.random_range(-1.5..1.5),
            );
        let mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.25, 0.3),
            emissive: LinearRgba::rgb(2.2, 0.3, 0.4),
            ..default()
        });
        commands.spawn((
            Name::new("Enemy"),
            Enemy { speed },
            Health::new(ENEMY_HEALTH),
            RigidBody::Dynamic,
            Collider::capsule(ENEMY_R, ENEMY_LEN),
            LockedAxes::ROTATION_LOCKED,
            enemy_layers,
            Mesh3d(
                meshes.add(
                    TriangleMeshBuilder::new_octahedron(2)
                        .with_scale(Vec3::splat(ENEMY_R * 1.6))
                        .build(),
                ),
            ),
            MeshMaterial3d(mat),
            Transform::from_translation(pos),
            DespawnOnExit(GameState::Playing),
        ));
    }
    wave.alive = count;
}

/// Start the first wave and, whenever the arena is cleared, roll the next
/// (bigger, faster) one. `wave.number` counts waves spawned so far (shown 1-based
/// on the HUD); `spawn_wave` sizes off the pre-increment 0-based index.
fn advance_waves(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wave: ResMut<Wave>,
    sfx: Res<SoundBank<Sfx>>,
    enemies: Query<(), With<Enemy>>,
) {
    let alive = enemies.iter().count() as u32;
    wave.alive = alive;
    if alive > 0 {
        return;
    }
    if wave.number > 0 {
        commands.play_sfx_volume(sfx.get(Sfx::Wave), 0.6);
    }
    spawn_wave(&mut commands, &mut meshes, &mut materials, &mut wave);
    wave.number += 1;
}

fn drive_enemies(
    player: Single<&Transform, With<Player>>,
    mut enemies: Query<(&Transform, &mut LinearVelocity, &Enemy), Without<Player>>,
) {
    let target = player.translation;
    for (transform, mut vel, enemy) in &mut enemies {
        let mut to = target - transform.translation;
        to.y = 0.0;
        let dir = to.normalize_or_zero();
        vel.0.x = dir.x * enemy.speed;
        vel.0.z = dir.z * enemy.speed;
    }
}

/// Any enemy within `MELEE_RANGE` drains the player continuously (damage-over-time,
/// scaled by how many are on you). A per-hit cooldown was unreliable: enemies jostle
/// in and out of range faster than the cooldown cycles, so a standing player barely
/// took damage. Continuous proximity damage makes a swarm a real, verifiable threat.
fn enemy_melee(
    time: Res<Time>,
    mut commands: Commands,
    mut feedback_timer: Local<f32>,
    player: Single<(Entity, &Transform), With<Player>>,
    enemies: Query<&Transform, (With<Enemy>, Without<Player>)>,
    mut vignette: Query<Entity, With<DamageVignette>>,
    mut shake: Query<&mut CameraShakeInput>,
    sfx: Res<SoundBank<Sfx>>,
) {
    let (player_entity, player_transform) = *player;
    let mut attackers = 0u32;
    for transform in &enemies {
        let mut to = player_transform.translation - transform.translation;
        to.y = 0.0;
        if to.length() <= MELEE_RANGE {
            attackers += 1;
        }
    }
    if attackers == 0 {
        return;
    }

    commands.trigger(HealthApplyDamage {
        entity: player_entity,
        source: None,
        amount: attackers as f32 * ENEMY_DPS * time.delta_secs(),
    });

    // Throttle the juice so it pulses rather than firing every frame.
    *feedback_timer -= time.delta_secs();
    if *feedback_timer <= 0.0 {
        *feedback_timer = 0.45;
        if let Ok(overlay) = vignette.single_mut() {
            commands.entity(overlay).insert(ScreenFlash {
                peak_alpha: 0.4,
                decay: 3.0,
                despawn_on_end: false,
            });
        }
        if let Ok(mut input) = shake.single_mut() {
            input.add_trauma += 0.3;
        }
        commands.play_sfx_volume(sfx.get(Sfx::Hurt), 0.5);
    }
}

// --- Death / run end --------------------------------------------------------

// No `SoundBank` param here, so the lose/score accounting is unit-testable with
// just HealthPlugin (the death sound plays in `on_fragments_spawned`).
fn on_health_zero(
    add: On<Add, HealthZeroMarker>,
    mut commands: Commands,
    enemies: Query<(), With<Enemy>>,
    players: Query<(), With<Player>>,
    mut score: ResMut<Score>,
    mut combo: ResMut<Combo>,
    mut feed: ResMut<KillFeed>,
    mut over: ResMut<RunOver>,
) {
    let entity = add.entity;
    if enemies.contains(entity) {
        // Score the kill by the current streak length: chained kills pay more.
        let streak = combo.streak.hit();
        let gained = BASE_KILL_POINTS * streak as u32;
        score.points += gained;
        score.kills += 1;
        combo.window_points += gained;
        feed.0.push(gained);
        // Stop steering / meleeing a dead enemy in the frames before it despawns:
        // drop `Enemy` and burst into gibs; the fragments observer despawns the body.
        commands
            .entity(entity)
            .remove::<Enemy>()
            .insert(ExplodeMesh { fragment_count: 6 });
    } else if players.contains(entity) {
        over.0 = true;
    }
}

/// Turn an exploded enemy's shards into short-lived physics gibs, then remove the
/// sliced shell (mirrors `10_asteroids::on_fragments_spawned`). `ExplodeFragments`
/// is only ever inserted on a dead enemy, so no marker filter is needed.
fn on_fragments_spawned(
    insert: On<Insert, ExplodeFragments>,
    mut commands: Commands,
    sfx: Res<SoundBank<Sfx>>,
    q: Query<(
        &ExplodeFragments,
        &Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let entity = insert.entity;
    let Ok((fragments, transform, material)) = q.get(entity) else {
        return;
    };
    commands.play_sfx_volume(sfx.get(Sfx::EnemyDown), 0.5);
    let mut rng = rand::rng();
    for fragment in fragments.iter() {
        let world_dir = (transform.rotation * fragment.direction.as_vec3()).normalize_or_zero();
        let vel = world_dir * rng.random_range(3.0..6.0) + Vec3::Y * 2.0;
        commands.spawn((
            Mesh3d(fragment.mesh.clone()),
            MeshMaterial3d(material.0.clone()),
            Transform::from_translation(transform.translation),
            RigidBody::Dynamic,
            Collider::sphere(0.15),
            CollisionLayers::new([GameLayer::Default], [GameLayer::World]),
            LinearVelocity(vel),
            TempEntity(1.4),
            DespawnOnExit(GameState::Playing),
        ));
    }
    commands.entity(entity).despawn();
}

// --- Combos / scoring feedback ----------------------------------------------

/// Live "COMBO xN" readout under the crosshair; shown only while a streak runs.
#[derive(Component)]
struct ComboText;

/// Drain the frame's kills into "+N" popups floating up from near the crosshair
/// (jittered so a multi-kill does not stack into one label). Runs in Playing, so
/// it has the window; the death observer only fills `KillFeed`.
fn spawn_kill_popups(
    mut commands: Commands,
    mut feed: ResMut<KillFeed>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if feed.0.is_empty() {
        return;
    }
    let Ok(window) = windows.single() else {
        feed.0.clear();
        return;
    };
    let centre = Vec2::new(window.width(), window.height()) * 0.5;
    let mut rng = rand::rng();
    for gained in feed.0.drain(..) {
        let jitter = Vec2::new(rng.random_range(-60.0..60.0), rng.random_range(-40.0..10.0));
        commands
            .spawn(popup(
                centre + jitter,
                format!("+{gained}"),
                30.0,
                Color::srgb(0.95, 0.85, 0.3),
            ))
            .insert(DespawnOnExit(GameState::Playing));
    }
}

/// Advance the combo decay; when the streak lapses on a chain of 2+, flash a
/// "COMBO xN +P" tally near the top of the screen and reset the window points.
fn tick_combo(
    mut commands: Commands,
    time: Res<Time>,
    mut combo: ResMut<Combo>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let Some(final_count) = combo.streak.tick(time.delta_secs()) else {
        return;
    };
    if final_count >= 2 {
        if let Ok(window) = windows.single() {
            let pos = Vec2::new(window.width() * 0.5 - 90.0, window.height() * 0.28);
            commands
                .spawn(popup(
                    pos,
                    format!("COMBO x{} +{}", final_count, combo.window_points),
                    40.0,
                    Color::srgb(1.0, 0.6, 0.2),
                ))
                .insert(DespawnOnExit(GameState::Playing));
        }
    }
    combo.window_points = 0;
}

/// Show/hide and update the live combo readout, fading it as the window drains.
fn update_combo_text(
    combo: Res<Combo>,
    mut q: Query<(&mut Text, &mut TextColor, &mut Visibility), With<ComboText>>,
) {
    let Ok((mut text, mut color, mut vis)) = q.single_mut() else {
        return;
    };
    let count = combo.streak.count();
    if count >= 2 {
        *vis = Visibility::Inherited;
        **text = format!("COMBO x{count}");
        let alpha = combo.streak.remaining_frac().clamp(0.25, 1.0);
        color.0 = Color::srgba(1.0, 0.6, 0.2, alpha);
    } else {
        *vis = Visibility::Hidden;
    }
}

fn mirror_player_hp(player: Single<&Health, With<Player>>, mut hp: ResMut<PlayerHp>) {
    hp.current = player.current;
    hp.max = player.max;
}

fn check_run_over(over: Res<RunOver>, mut next: ResMut<NextState<GameState>>) {
    if over.0 {
        next.set(GameState::GameOver);
    }
}

// --- Touch ------------------------------------------------------------------

fn read_touch(
    touches: Res<Touches>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut touch: ResMut<TouchInput>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let w = Vec2::new(window.width(), window.height());
    touch.move_vec = Vec2::ZERO;
    touch.look_delta = Vec2::ZERO;
    touch.fire = false;

    // Drop fingers that lifted.
    if let Some((id, _)) = touch.move_finger {
        if touches.get_pressed(id).is_none() {
            touch.move_finger = None;
        }
    }
    if let Some((id, _)) = touch.look_finger {
        if touches.get_pressed(id).is_none() {
            touch.look_finger = None;
        }
    }

    // The fire button occupies the bottom-right; a touch there fires.
    // Matches the on-screen FIRE button (right: 6%, bottom: 12%, ~96px square), so a
    // tap on the button reads as fire and the rest of the right half still looks.
    let fire_zone = Rect::new(0.66, 0.74, 0.98, 0.92);

    for t in touches.iter() {
        let id = t.id();
        let pos = t.position();
        let frac = pos / w;
        if button_grid_at(pos, w, 1, 1, fire_zone).is_some() {
            touch.fire = true;
            continue;
        }
        if frac.x < 0.5 {
            // Left half: movement stick. Ignore extra left-half fingers -- only the
            // one that owns the slot (or the first to claim it) drives movement.
            match touch.move_finger {
                Some((fid, origin)) if fid == id => {
                    let d = stick_deflection(pos - origin, 70.0, 8.0);
                    // Screen y is down; forward is up-screen.
                    touch.move_vec = Vec2::new(d.x, -d.y);
                }
                None => touch.move_finger = Some((id, pos)),
                _ => {}
            }
        } else {
            // Right half: look. Same single-owner rule.
            match touch.look_finger {
                Some((fid, last)) if fid == id => {
                    touch.look_delta = pos - last;
                    touch.look_finger = Some((id, pos));
                }
                None => touch.look_finger = Some((id, pos)),
                _ => {}
            }
        }
    }
}

// --- Game over --------------------------------------------------------------

fn record_high_score(score: Res<Score>, mut high: ResMut<HighScore<u32>>) {
    high.record(score.points);
}

fn spawn_game_over(
    mut commands: Commands,
    score: Res<Score>,
    wave: Res<Wave>,
    high: Res<HighScore<u32>>,
) {
    let new_best = high.is_new_best();
    commands
        .spawn((
            Name::new("Game Over"),
            DespawnOnExit(GameState::GameOver),
            centered_screen(),
        ))
        .with_children(|parent| {
            parent.spawn(screen_text("YOU DIED", 84.0, Color::srgb(1.0, 0.3, 0.3)));
            parent.spawn(screen_text(
                format!(
                    "{} pts -- {} kills over {} waves",
                    score.points, score.kills, wave.number
                ),
                30.0,
                Color::srgb(0.95, 0.95, 1.0),
            ));
            if new_best {
                parent.spawn(screen_text(
                    "New best!",
                    26.0,
                    Color::srgb(0.95, 0.85, 0.35),
                ));
            } else {
                parent.spawn(screen_text(
                    best_line(high.best()),
                    24.0,
                    Color::srgb(0.8, 0.8, 0.9),
                ));
            }
            parent.spawn(screen_text(
                "Click or press any key for the menu",
                22.0,
                Color::srgb(0.7, 0.75, 0.85),
            ));
        });
}

fn play_game_over_sfx(mut commands: Commands, sfx: Res<SoundBank<Sfx>>) {
    commands.play_sfx_volume(sfx.get(Sfx::GameOver), 0.8);
}

fn gameover_dismiss(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut next: ResMut<NextState<GameState>>,
) {
    let pressed = mouse.just_pressed(MouseButton::Left)
        || keys.get_just_pressed().next().is_some()
        || touches.any_just_pressed();
    if pressed {
        next.set(GameState::Menu);
    }
}

// --- Tests ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // The look/move math (move_dir, pitch clamp) now lives in and is tested by the
    // crate's `physics/doom_controller` module.

    #[test]
    fn wave_size_ramps_monotonically() {
        assert_eq!(wave_size(0), 3);
        assert_eq!(wave_size(1), 5);
        assert_eq!(wave_size(2), 7);
        for n in 0..10 {
            assert!(wave_size(n + 1) > wave_size(n));
        }
    }

    #[test]
    fn ring_positions_count_and_radius() {
        let r = 10.0;
        let pts = ring_positions(6, r);
        assert_eq!(pts.len(), 6);
        for p in &pts {
            let horiz = Vec2::new(p.x, p.z).length();
            assert!((horiz - r).abs() < 1e-3);
            assert!((p.y - PLAYER_CENTER_Y).abs() < 1e-6);
        }
    }

    #[test]
    fn ring_positions_handles_zero() {
        assert!(ring_positions(0, 5.0).is_empty());
    }

    #[test]
    fn enemy_speed_ramps() {
        assert!(enemy_speed(3) > enemy_speed(0));
    }

    // The lose condition and score accounting live in the `on_health_zero` observer
    // and `check_run_over`, which the headless autopilot can NOT prove (it force-
    // transitions Playing -> GameOver on a timer). Exercise them in a real App.

    fn death_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin, HealthPlugin));
        app.init_state::<GameState>();
        app.init_resource::<Score>();
        app.init_resource::<Combo>();
        app.init_resource::<KillFeed>();
        app.init_resource::<RunOver>();
        app.add_observer(on_health_zero);
        app.add_systems(Update, check_run_over);
        app
    }

    fn kill_enemy(app: &mut App) {
        let enemy = app
            .world_mut()
            .spawn((Enemy { speed: 1.0 }, Health::new(10.0)))
            .id();
        app.world_mut().trigger(HealthApplyDamage {
            entity: enemy,
            source: None,
            amount: 25.0,
        });
        // The death chain (HealthZeroMarker -> on_health_zero) needs a flush.
        app.update();
    }

    #[test]
    fn player_death_ends_the_run() {
        let mut app = death_app();
        let player = app.world_mut().spawn((Player, Health::new(10.0))).id();
        app.world_mut().trigger(HealthApplyDamage {
            entity: player,
            source: None,
            amount: 25.0,
        });
        // Damage -> HealthZeroMarker -> on_health_zero sets RunOver -> check_run_over
        // -> NextState(GameOver) -> the transition applies. A few frames cover it.
        for _ in 0..4 {
            app.update();
        }
        assert!(
            app.world().resource::<RunOver>().0,
            "player death sets RunOver"
        );
        assert_eq!(
            *app.world().resource::<State<GameState>>().get(),
            GameState::GameOver,
            "the run ends at GameOver"
        );
    }

    #[test]
    fn enemy_death_scores_one_kill_and_does_not_end_the_run() {
        let mut app = death_app();
        kill_enemy(&mut app);
        let score = app.world().resource::<Score>();
        assert_eq!(score.kills, 1, "one kill counted");
        assert_eq!(
            score.points, BASE_KILL_POINTS,
            "the first kill (streak 1) is worth the base points"
        );
        assert!(
            !app.world().resource::<RunOver>().0,
            "an enemy dying does not end the run"
        );
    }

    #[test]
    fn chained_kills_multiply_by_the_streak() {
        let mut app = death_app();
        // Two kills back-to-back (no `tick_combo` runs, so the streak never lapses):
        // streak 1 -> BASE, streak 2 -> 2*BASE, totalling 3*BASE points over 2 kills.
        kill_enemy(&mut app);
        kill_enemy(&mut app);
        let score = app.world().resource::<Score>();
        assert_eq!(score.kills, 2);
        assert_eq!(
            score.points,
            BASE_KILL_POINTS * 3,
            "streak-scaled: 1x + 2x base"
        );
        assert_eq!(
            app.world().resource::<Combo>().streak.count(),
            2,
            "the streak is at 2"
        );
    }

    #[test]
    fn the_streak_lapses_after_its_window() {
        let mut streak = Streak::new(COMBO_WINDOW);
        streak.hit();
        streak.hit();
        assert_eq!(
            streak.tick(COMBO_WINDOW * 0.5),
            None,
            "still inside the window"
        );
        assert_eq!(
            streak.tick(COMBO_WINDOW),
            Some(2),
            "past the window it lapses, returning the final count"
        );
    }
}
