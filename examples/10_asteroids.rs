//! A 3D "asteroids" shooter, and the headline demo of using `ExplodeMeshPlugin`
//! to turn a sliced mesh into real physics bodies.
//!
//! Boot into a main menu, tap or click to play. You fly a little ship around a
//! bounded arena in zero gravity and shoot drifting octahedron asteroids. Unlike
//! `06_fruitninja`, where a slice bursts into fragments that just fade and
//! despawn, here a hit inserts `ExplodeMesh` and every sliced shard is respawned
//! as a real avian3d rigid body: the shards keep drifting, bounce off the arena
//! walls, and become new, smaller hazards you still have to clear. Shoot the big
//! rocks into medium shards, the medium shards into small ones, and the small
//! ones into nothing -- clear the whole field to advance to the next, busier
//! wave. Bump into any rock and you lose a hull point (with a brief invulnerable
//! flash); run out of hull and the run ends.
//!
//! Controls are unified across keyboard, mouse and touch so the wasm/mobile
//! showcase build is playable with a single finger:
//!
//! - Keyboard (classic): A/D or Left/Right rotate, W/Up thrust, Space fires.
//! - Pointer (mouse or touch): the ship turns to face the pointer; hold to fly
//!   toward it and auto-fire, release to coast on inertia.
//!
//! What each crate piece does here:
//! - `mesh/explode` (`ExplodeMeshPlugin`) slices a hit asteroid; the
//!   `on_fragments_spawned` observer respawns each shard as an avian body.
//! - avian3d supplies the whole simulation: `RigidBody::Dynamic` asteroids with
//!   `LinearVelocity`/`AngularVelocity`, static `RigidBody::Static` walls with
//!   `Restitution` for the bounces, sensor bullets, `CollisionLayers` to keep
//!   asteroids from colliding with each other, `LockedAxes` to pin play to the
//!   XY plane, and `CollisionStart` messages for the hit / damage logic.
//! - `camera/post` (`PostProcessingDefaultPlugin`) adds bloom so the glowing
//!   bullets and thruster flame streak.
//! - `HealthPlugin` owns the hull / lose condition, `SfxPlugin` the one-shots,
//!   and `StatusBarPlugin` the FPS overlay; Bevy states run the menu flow.
//!
//! Sounds live in `assets/sounds/` and are tiny generated placeholders; see
//! `assets/sounds/README.md` to drop in real audio.

use std::{
    collections::HashSet,
    f32::consts::{FRAC_PI_2, PI, TAU},
};

use avian3d::prelude::*;
use bevy::{mesh::VertexAttributeValues, prelude::*};
use bevy_common_systems::prelude::*;
use clap::Parser;
use noise::{Fbm, MultiFractal, Perlin};
use rand::{Rng, RngCore};

#[derive(Parser)]
#[command(name = "10_asteroids")]
#[command(version = "1.0.0")]
#[command(
    about = "Fly a ship and shoot drifting asteroids into physics-body fragments. Rotate with A/D, thrust with W, fire with Space -- or hold the mouse / a finger to fly toward it and shoot.",
    long_about = None
)]
struct Cli;

// ---------------------------------------------------------------------------
// Tunable constants
// ---------------------------------------------------------------------------

/// Half-extents of the square play arena on the XY plane, in world units. The
/// static walls sit just outside these bounds and every body plays inside them.
const ARENA_HALF: f32 = 15.0;

/// Thickness of the (invisible) static wall colliders that contain the arena.
const WALL_THICKNESS: f32 = 2.0;

/// Vertical field of view of the camera, in radians (Bevy's perspective default).
/// Used by `fit_camera` to pull the camera back far enough to frame the arena at
/// any window aspect, so the portrait mobile canvas and a landscape desktop
/// window both show the whole field.
const CAMERA_FOV: f32 = PI / 4.0;

/// Extra margin around the arena when framing it, as a fraction of `ARENA_HALF`.
const CAMERA_MARGIN: f32 = 1.15;

/// Ship collider radius (a sphere approximating the cone hull), in world units.
const SHIP_RADIUS: f32 = 0.7;

/// Ship flight model: thrust acceleration, top speed, and how fast idle velocity
/// bleeds off (exponential drag per second). Drag is light so the ship keeps a
/// satisfying inertial drift, in the spirit of the original game.
const SHIP_THRUST: f32 = 26.0;
const SHIP_MAX_SPEED: f32 = 16.0;
const SHIP_DRAG: f32 = 0.7;

/// How fast the ship turns, in radians per second (keyboard and pointer aim).
const SHIP_TURN_SPEED: f32 = 3.6;

/// Bullet speed and lifetime, and the minimum time between shots.
const BULLET_SPEED: f32 = 28.0;
const BULLET_RADIUS: f32 = 0.16;
const BULLET_LIFETIME: f32 = 1.05;
const FIRE_COOLDOWN: f32 = 0.16;

/// Seconds of invulnerability after taking a hit; the ship blinks during it.
const INVULN_TIME: f32 = 1.6;

/// Player hull at the start of a run. Each rock you bump into costs one point;
/// it is a real `Health` value so the example drives the crate's health system.
const PLAYER_HEALTH: f32 = 3.0;

/// Collider radius of an asteroid at each split generation (index = generation).
/// Generation 0 is a full rock; higher generations are the smaller shards. The
/// last entry is also the terminal size: shooting one just destroys it.
const ASTEROID_RADII: [f32; 3] = [2.2, 1.15, 0.6];

/// The highest split generation. A hit on a lower generation slices the rock
/// into the next; a hit on this one destroys it outright (no more bodies).
const MAX_SPLIT_GEN: usize = 2;

/// Fragments requested from `ExplodeMesh` per split. Kept small so the field
/// stays clearable and the body count bounded (up to ~1 + 3 + 9 per rock).
const FRAGMENTS_PER_SPLIT: usize = 3;

/// Extra outward speed each shard is given along its slice direction when a rock
/// splits, on top of inheriting the parent rock's drift velocity.
const SPLIT_SPEED: f32 = 4.5;

/// Restitution of the arena walls; near-elastic so rocks keep drifting after a
/// bounce instead of piling up against the edge.
const WALL_RESTITUTION: f32 = 1.0;
const ASTEROID_RESTITUTION: f32 = 0.9;

/// Drift-speed band for freshly spawned rocks. The upper end climbs with the
/// wave number (see `asteroid_drift_speed`) so later waves feel faster.
const ASTEROID_SPEED_MIN: f32 = 1.8;
const ASTEROID_SPEED_BASE: f32 = 3.2;
const ASTEROID_SPEED_PER_WAVE: f32 = 0.6;
const ASTEROID_SPEED_MAX: f32 = 9.0;

/// Max magnitude of a rock's random tumble, radians per second about Z.
const ASTEROID_SPIN_MAX: f32 = 1.6;

/// Resting camera framing is computed by `fit_camera`; shake offsets are applied
/// on top of it. These govern the impact shake.
const SHAKE_DECAY: f32 = 1.8;
const SHAKE_MAX_OFFSET: f32 = 0.7;
const SHAKE_SPLIT: f32 = 0.22;
const SHAKE_HIT: f32 = 0.7;

/// Seconds the death sequence holds (ship gone, red flash) before the game-over
/// screen appears.
const DYING_BEAT: f32 = 0.6;

/// Number of background stars scattered behind the arena.
const STAR_COUNT: usize = 90;

fn main() {
    let _ = Cli::parse();
    let mut app = App::new();

    // On the web the game runs inside a canvas: fit it to its parent element so
    // it fills the (portrait, mobile-ish) frame the showcase site embeds it in.
    // These fields are ignored on native, so the desktop example is unchanged.
    let primary_window = Window {
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

    // avian runs the whole simulation. Space has no gravity, so zero the global
    // resource; bodies drift on their own momentum.
    app.add_plugins(PhysicsPlugins::default());
    app.insert_resource(Gravity(Vec3::ZERO));

    // Deep-space backdrop.
    app.insert_resource(ClearColor(Color::srgb(0.01, 0.01, 0.03)));

    #[cfg(feature = "debug")]
    app.add_plugins(InspectorDebugPlugin);

    // FrameTimeDiagnosticsPlugin feeds the status bar's FPS item.
    if !app.is_plugin_added::<bevy::diagnostic::FrameTimeDiagnosticsPlugin>() {
        app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
    }

    app.add_plugins(ExplodeMeshPlugin);
    app.add_plugins(PostProcessingDefaultPlugin);
    app.add_plugins(CameraShakePlugin);
    app.add_plugins(StatusBarPlugin);
    app.add_plugins(HealthPlugin);
    app.add_plugins(SfxPlugin);

    app.init_state::<GameState>();

    app.init_resource::<Pointer>();
    app.init_resource::<Score>();
    app.init_resource::<HighScore>();
    app.init_resource::<NewBest>();
    app.init_resource::<Wave>();
    app.init_resource::<DyingTimer>();

    // Persistent scene: camera, lights, arena walls, starfield, FPS overlay.
    app.add_systems(Startup, setup);

    // Pointer state is resolved every frame, in every state, so menus and
    // gameplay share one mouse+touch abstraction.
    app.add_systems(PreUpdate, update_pointer);

    // `fit_camera` writes the camera base every frame; it must run between the
    // shake plugin's Restore and Apply phases (both in PostUpdate) so the shake
    // offset rides on top of the fresh framing rather than accumulating.
    app.add_systems(
        PostUpdate,
        fit_camera
            .after(CameraShakeSystems::Restore)
            .before(CameraShakeSystems::Apply),
    );
    app.add_systems(Update, draw_starfield);

    // Main menu.
    app.add_systems(OnEnter(GameState::Menu), spawn_menu);
    app.add_systems(
        Update,
        (menu_click, pulse_menu_title).run_if(in_state(GameState::Menu)),
    );

    // Playing: reset, spawn the ship + HUD + first wave, then run the loop.
    app.add_systems(
        OnEnter(GameState::Playing),
        (start_game, spawn_ship, spawn_hud, spawn_wave).chain(),
    );
    app.add_systems(
        Update,
        (
            control_ship,
            fire_bullets,
            tick_bullets,
            clamp_asteroid_speed,
            handle_collisions,
            blink_invulnerable_ship,
            advance_wave,
            draw_arena_border,
            update_hud,
            advance_dying,
            giveup_on_escape,
        )
            .run_if(in_state(GameState::Playing)),
    );

    // Game over screen.
    app.add_systems(
        OnEnter(GameState::GameOver),
        (record_high_score, spawn_game_over, play_game_over_sfx).chain(),
    );
    app.add_systems(Update, gameover_click.run_if(in_state(GameState::GameOver)));

    app.add_observer(on_fragments_spawned);
    app.add_observer(on_player_died);

    app.run();
}

// ---------------------------------------------------------------------------
// Physics layers and top-level state
// ---------------------------------------------------------------------------

/// Collision layers. Bullets only ever interact with asteroids; asteroids bounce
/// off walls and the ship but pass through each other (as in the original game),
/// which also avoids a pile of shards resolving overlaps the instant a rock
/// splits. The ship only reports contact with asteroids.
#[derive(PhysicsLayer, Default, Clone, Copy)]
enum GameLayer {
    #[default]
    Default,
    Ship,
    Bullet,
    Asteroid,
    Wall,
}

/// Top-level game flow: the menu, the playable run, and the game-over screen.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

// ---------------------------------------------------------------------------
// Unified pointer input (mouse + touch)
// ---------------------------------------------------------------------------

/// The current pointer, unified across mouse and touch. An active touch wins over
/// the mouse cursor so a finger drives aiming on the wasm/mobile build.
#[derive(Resource, Default)]
struct Pointer {
    /// On-screen position (logical window pixels) of the active pointer, if any.
    screen_pos: Option<Vec2>,
    /// Whether the pointer is currently down (mouse button held or a finger on
    /// the screen).
    pressed: bool,
    /// True only on the frame the press began, for the menu "advance on tap".
    just_pressed: bool,
}

/// Resolve the pointer from raw mouse and touch input each frame. Reading the raw
/// state directly (rather than through `bevy_enhanced_input`) keeps this example
/// self-contained; the desktop path is simply the mouse, since `Touches` is
/// always empty there.
fn update_pointer(
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    window: Single<&Window>,
    mut pointer: ResMut<Pointer>,
) {
    let touch_pos = touches.iter().next().map(|touch| touch.position());
    let touch_just = touches.iter_just_pressed().next().is_some();

    pointer.pressed = mouse.pressed(MouseButton::Left) || touch_pos.is_some();
    pointer.just_pressed = mouse.just_pressed(MouseButton::Left) || touch_just;
    // An active touch takes priority over the mouse cursor.
    pointer.screen_pos = touch_pos.or_else(|| window.cursor_position());
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Running number of asteroids destroyed this run.
#[derive(Resource, Default, Deref, DerefMut)]
struct Score(usize);

/// Best score across runs this session (not reset per run).
#[derive(Resource, Default)]
struct HighScore(usize);

/// Whether the most recent run set a new high score (for the game-over screen).
#[derive(Resource, Default)]
struct NewBest(bool);

/// The current wave number (1-based). Higher waves spawn more, faster rocks.
#[derive(Resource, Default, Deref, DerefMut)]
struct Wave(usize);

/// Countdown before the game-over screen after the ship is destroyed.
#[derive(Resource, Default)]
struct DyingTimer {
    remaining: Option<f32>,
}

/// Shared render / build assets, so spawning rocks and bullets is cheap.
#[derive(Resource)]
struct GameAssets {
    /// A handful of pre-built noise-displaced octahedron rock meshes (generation
    /// 0), sized to `ASTEROID_RADII[0]`. Each new rock picks one at random.
    rock_meshes: Vec<Handle<Mesh>>,
    rock_material: Handle<StandardMaterial>,
    bullet_mesh: Handle<Mesh>,
    bullet_material: Handle<StandardMaterial>,
}

/// One `AudioSource` handle per gameplay event; the files under `assets/sounds/`
/// are placeholders (see `assets/sounds/README.md`).
#[derive(Resource)]
struct SfxAssets {
    /// Clicking "play" on the menu / tapping back from game over.
    menu_select: Handle<AudioSource>,
    /// The ship fires a bullet.
    shot: Handle<AudioSource>,
    /// An asteroid is destroyed / split.
    explode: Handle<AudioSource>,
    /// The ship takes a hit.
    hurt: Handle<AudioSource>,
    /// A wave is cleared.
    wave_clear: Handle<AudioSource>,
    /// The run ends.
    game_over: Handle<AudioSource>,
}

/// Fixed background star positions on a plane behind the arena, drawn as faint
/// gizmo points for a sense of depth.
#[derive(Resource)]
struct Starfield {
    points: Vec<Vec3>,
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// The player ship's flight state. The ship entity is an avian kinematic body
/// (so it reports contact with rocks and deflects them) whose velocity and
/// facing we author every frame; its visual cone is a child (`ShipModel`) we
/// rotate freely, decoupling the look from the sphere collider.
#[derive(Component)]
struct Ship {
    /// Facing angle on the play plane, radians measured from +X.
    heading: f32,
    /// Inertial velocity we integrate ourselves and push into `LinearVelocity`.
    velocity: Vec3,
    /// Time until the next shot is allowed, seconds.
    fire_cooldown: f32,
    /// Remaining invulnerability after a hit, seconds (0 = vulnerable).
    invuln: f32,
}

/// Marker for the ship's visual cone child, rotated to show the heading.
#[derive(Component)]
struct ShipModel;

/// Marker for the ship's thruster flame child, shown only while thrusting.
#[derive(Component)]
struct ShipFlame;

/// A drifting asteroid / shard. `generation` is how many times it has been split
/// (0 = a full rock); `MAX_SPLIT_GEN` is terminal.
#[derive(Component)]
struct Asteroid {
    generation: usize,
}

/// Marker added the moment a rock is scheduled to explode, so a second bullet in
/// the same frame cannot slice it twice.
#[derive(Component)]
struct Exploding;

/// Marker for a fired bullet (a kinematic sensor body).
#[derive(Component)]
struct Bullet {
    /// Remaining lifetime, seconds; despawns at zero if it has not hit anything.
    life: f32,
}

/// Marker for the main camera so the framing / shake systems can find it.
#[derive(Component)]
struct MainCamera;

/// Full-screen red flash shown briefly when the ship is destroyed.
#[derive(Component)]
struct RedFlash {
    age: f32,
    lifetime: f32,
}

/// HUD text markers.
#[derive(Component)]
struct ScoreText;
#[derive(Component)]
struct WaveText;
#[derive(Component)]
struct HullText;
#[derive(Component)]
struct MenuTitle;

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::rng();

    // Load one sound per gameplay event. Paths are relative to `assets/`.
    commands.insert_resource(SfxAssets {
        menu_select: asset_server.load("sounds/menu_select.wav"),
        shot: asset_server.load("sounds/launch.wav"),
        explode: asset_server.load("sounds/bomb.wav"),
        hurt: asset_server.load("sounds/hurt.wav"),
        wave_clear: asset_server.load("sounds/level_up.wav"),
        game_over: asset_server.load("sounds/game_over.wav"),
    });

    // A few distinct rocky octahedra, displaced with Fbm/Perlin noise (the
    // `02_planet` recipe) and pre-scaled to the generation-0 radius so slicing
    // and colliders all work in real world units, no Transform scale in play.
    let rock_meshes = (0..4)
        .map(|_| {
            let noise = Fbm::<Perlin>::new(rng.next_u32())
                .set_frequency(1.1)
                .set_persistence(0.5)
                .set_octaves(4);
            let mesh = TriangleMeshBuilder::new_octahedron(3)
                .apply_noise(&noise)
                .build()
                .scaled_by(Vec3::splat(ASTEROID_RADII[0]));
            meshes.add(mesh)
        })
        .collect();

    let rock_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.52, 0.5),
        perceptual_roughness: 0.95,
        ..default()
    });

    // Bullets glow hot so bloom streaks them.
    let bullet_mesh = meshes.add(Sphere::new(BULLET_RADIUS).mesh().ico(2).unwrap());
    // Emissive HDR (not `unlit`): `unlit` would skip the lighting pass where
    // emissive is applied, so the bullet would not bloom. Left lit, the HDR
    // emissive dominates and streaks under `camera/post` bloom.
    let bullet_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.95, 1.0),
        emissive: LinearRgba::rgb(1.0, 5.0, 8.0),
        ..default()
    });

    commands.insert_resource(GameAssets {
        rock_meshes,
        rock_material,
        bullet_mesh,
        bullet_material,
    });

    // Top-down camera looking straight at the XY play plane; `fit_camera` sets
    // its distance every frame so the arena stays framed at any aspect.
    commands.spawn((
        Name::new("Main Camera"),
        MainCamera,
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
        PostProcessingCamera,
        // Impact shake on top of `fit_camera`'s base. The arena is centered on
        // the origin so only x/y are offset; z stays 0 so `fit_camera` keeps
        // ownership of the framing distance.
        CameraShake {
            decay: SHAKE_DECAY,
            max_offset: Vec3::new(SHAKE_MAX_OFFSET, SHAKE_MAX_OFFSET, 0.0),
            ..default()
        },
        AmbientLight {
            color: Color::WHITE,
            brightness: 220.0,
            ..default()
        },
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 6000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.7, -0.5, 0.0)),
    ));

    // Four static walls just outside the arena bounds, near-elastic and
    // frictionless so rocks bounce back in and keep their momentum. They are
    // invisible; `draw_arena_border` shows the boundary while playing.
    spawn_walls(&mut commands);

    // A fixed starfield on a plane well behind the action.
    let points = (0..STAR_COUNT)
        .map(|_| {
            Vec3::new(
                rng.random_range(-ARENA_HALF * 1.6..ARENA_HALF * 1.6),
                rng.random_range(-ARENA_HALF * 1.6..ARENA_HALF * 1.6),
                -12.0,
            )
        })
        .collect();
    commands.insert_resource(Starfield { points });

    // Status bar: FPS only (score / wave / hull live in the in-game HUD).
    commands.spawn((status_bar(StatusBarRootConfig::default()),));
    commands.spawn((status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: status_fps_value_fn(),
        color_fn: status_fps_color_fn(),
        prefix: "".to_string(),
        suffix: "fps".to_string(),
    }),));
}

/// Spawn the four static wall colliders that contain the arena.
fn spawn_walls(commands: &mut Commands) {
    let span = ARENA_HALF * 2.0 + WALL_THICKNESS * 2.0;
    let offset = ARENA_HALF + WALL_THICKNESS * 0.5;
    // (center, half-extents) for each of the four walls.
    let walls = [
        (
            Vec3::new(0.0, offset, 0.0),
            Vec3::new(span, WALL_THICKNESS, 10.0),
        ),
        (
            Vec3::new(0.0, -offset, 0.0),
            Vec3::new(span, WALL_THICKNESS, 10.0),
        ),
        (
            Vec3::new(offset, 0.0, 0.0),
            Vec3::new(WALL_THICKNESS, span, 10.0),
        ),
        (
            Vec3::new(-offset, 0.0, 0.0),
            Vec3::new(WALL_THICKNESS, span, 10.0),
        ),
    ];
    for (center, size) in walls {
        commands.spawn((
            Name::new("Wall"),
            Transform::from_translation(center),
            RigidBody::Static,
            Collider::cuboid(size.x, size.y, size.z),
            Restitution::new(WALL_RESTITUTION).with_combine_rule(CoefficientCombine::Max),
            Friction::ZERO,
            CollisionLayers::new([GameLayer::Wall], [GameLayer::Asteroid]),
        ));
    }
}

/// Pull the camera back far enough to frame the whole arena at the current window
/// aspect, so both the portrait mobile canvas and a landscape desktop window show
/// the full field. The base Z is stored back into the transform; the
/// `CameraShakePlugin` adds the shake offset on top (Restore -> `fit_camera` ->
/// Apply, all in `PostUpdate`).
fn fit_camera(window: Single<&Window>, mut q_cam: Query<&mut Transform, With<MainCamera>>) {
    let Ok(mut transform) = q_cam.single_mut() else {
        return;
    };
    let aspect = (window.width() / window.height()).max(0.01);
    let margin = ARENA_HALF * CAMERA_MARGIN;
    let half_fov = CAMERA_FOV * 0.5;
    // Distance needed to fit the margin vertically, and horizontally given the
    // aspect; take the larger so nothing is cropped.
    let dist_v = margin / half_fov.tan();
    let dist_h = margin / (half_fov.tan() * aspect);
    transform.translation.z = dist_v.max(dist_h);
}

/// Draw the fixed starfield behind the arena.
fn draw_starfield(stars: Res<Starfield>, mut gizmos: Gizmos) {
    for point in &stars.points {
        gizmos.circle(
            Isometry3d::from_translation(*point),
            0.05,
            Color::srgba(0.7, 0.75, 0.9, 0.5),
        );
    }
}

// ---------------------------------------------------------------------------
// Menu
// ---------------------------------------------------------------------------

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
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        TextLayout {
            justify: Justify::Center,
            ..default()
        },
    )
}

fn spawn_menu(mut commands: Commands, high: Res<HighScore>) {
    commands.spawn((
        Name::new("Main Menu"),
        DespawnOnExit(GameState::Menu),
        centered_screen(),
        children![
            (
                screen_text("ASTEROIDS", 72.0, Color::srgb(0.6, 0.9, 1.0)),
                MenuTitle,
            ),
            screen_text("Tap or click to play", 32.0, Color::WHITE),
            screen_text(
                format!("Best: {}", high.0),
                24.0,
                Color::srgb(0.6, 0.9, 1.0),
            ),
            screen_text(
                "A/D rotate - W thrust - Space fire, or hold the pointer to fly and shoot",
                20.0,
                Color::srgb(0.7, 0.7, 0.7),
            ),
        ],
    ));
}

/// Gently pulse the menu title's alpha so the menu breathes.
fn pulse_menu_title(time: Res<Time>, mut q_title: Query<&mut TextColor, With<MenuTitle>>) {
    let alpha = 0.65 + 0.35 * (time.elapsed_secs() * 2.5).sin();
    for mut color in q_title.iter_mut() {
        color.0 = Color::srgba(0.6, 0.9, 1.0, alpha);
    }
}

fn menu_click(
    mut commands: Commands,
    pointer: Res<Pointer>,
    keys: Res<ButtonInput<KeyCode>>,
    sfx: Res<SfxAssets>,
    mut next: ResMut<NextState<GameState>>,
) {
    if pointer.just_pressed || keys.just_pressed(KeyCode::Space) {
        commands.play_sfx_volume(sfx.menu_select.clone(), 0.7);
        next.set(GameState::Playing);
    }
}

// ---------------------------------------------------------------------------
// Run lifecycle
// ---------------------------------------------------------------------------

/// Reset per-run state when a new game starts.
fn start_game(
    mut score: ResMut<Score>,
    mut wave: ResMut<Wave>,
    mut q_shake: Query<&mut CameraShakeInput>,
    mut dying: ResMut<DyingTimer>,
) {
    score.0 = 0;
    wave.0 = 1;
    if let Ok(mut input) = q_shake.single_mut() {
        input.reset = true;
    }
    dying.remaining = None;
}

/// Spawn the player ship: a kinematic avian body (sphere collider, reports rock
/// contact, deflects rocks) with a cone model and thruster flame as children.
fn spawn_ship(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let hull_mesh = meshes.add(Cone {
        radius: 0.6,
        height: 1.7,
    });
    let hull_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.9, 1.0),
        emissive: LinearRgba::rgb(0.1, 0.25, 0.5),
        ..default()
    });
    let flame_mesh = meshes.add(Cone {
        radius: 0.32,
        height: 1.0,
    });
    // Emissive HDR, left lit so the emissive is actually applied and blooms
    // (see the bullet material note on why `unlit` would suppress it).
    let flame_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.6, 0.1),
        emissive: LinearRgba::rgb(8.0, 3.0, 0.4),
        ..default()
    });

    commands
        .spawn((
            Name::new("Ship"),
            Ship {
                heading: FRAC_PI_2,
                velocity: Vec3::ZERO,
                fire_cooldown: 0.0,
                invuln: INVULN_TIME,
            },
            Health::new(PLAYER_HEALTH),
            Transform::from_xyz(0.0, 0.0, 0.0),
            // The ship carries no mesh of its own (its model / flame are
            // children), so give it an explicit Visibility for the children to
            // inherit -- otherwise their visibility does not propagate (B0004).
            Visibility::default(),
            RigidBody::Kinematic,
            Collider::sphere(SHIP_RADIUS),
            LinearVelocity::default(),
            LockedAxes::new().lock_translation_z(),
            CollisionLayers::new([GameLayer::Ship], [GameLayer::Asteroid]),
            CollisionEventsEnabled,
            DespawnOnExit(GameState::Playing),
        ))
        .with_children(|parent| {
            // Cone points along +Y in local space; `control_ship` rotates this
            // model child so +Y aligns with the heading. The flame is nested
            // under the model (not the ship) so it inherits that rotation and
            // trails behind the nose as the ship turns.
            parent.spawn((
                Name::new("Ship Model"),
                ShipModel,
                Mesh3d(hull_mesh),
                MeshMaterial3d(hull_material),
                Transform::default(),
                children![(
                    Name::new("Ship Flame"),
                    ShipFlame,
                    Mesh3d(flame_mesh),
                    MeshMaterial3d(flame_material),
                    // Behind the cone base (model-local -Y), pointing back.
                    Transform::from_xyz(0.0, -1.1, 0.0).with_rotation(Quat::from_rotation_z(PI)),
                    Visibility::Hidden,
                )],
            ));
        });
}

/// Spawn the in-game HUD (score / wave / hull), scoped to the `Playing` state.
fn spawn_hud(mut commands: Commands) {
    let hud_text = |text: &str, top: f32, color: Color| {
        (
            DespawnOnExit(GameState::Playing),
            Text::new(text.to_string()),
            TextFont {
                font_size: FontSize::Px(34.0),
                ..default()
            },
            TextColor(color),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(top),
                left: Val::Px(16.0),
                ..default()
            },
        )
    };
    commands.spawn((
        Name::new("Score HUD"),
        ScoreText,
        hud_text("Score: 0", 16.0, Color::srgb(0.6, 0.9, 1.0)),
    ));
    commands.spawn((
        Name::new("Wave HUD"),
        WaveText,
        hud_text("Wave 1", 54.0, Color::srgb(0.8, 0.85, 0.95)),
    ));
    commands.spawn((
        Name::new("Hull HUD"),
        HullText,
        hud_text("Hull: 3", 92.0, Color::srgb(0.5, 0.95, 0.6)),
    ));
}

/// Refresh the HUD text each frame.
fn update_hud(
    score: Res<Score>,
    wave: Res<Wave>,
    q_ship: Query<&Health, With<Ship>>,
    mut q_score: Query<&mut Text, (With<ScoreText>, Without<WaveText>, Without<HullText>)>,
    mut q_wave: Query<&mut Text, (With<WaveText>, Without<ScoreText>, Without<HullText>)>,
    mut q_hull: Query<&mut Text, (With<HullText>, Without<ScoreText>, Without<WaveText>)>,
) {
    if let Ok(mut text) = q_score.single_mut() {
        **text = format!("Score: {}", score.0);
    }
    if let Ok(mut text) = q_wave.single_mut() {
        **text = format!("Wave {}", wave.0);
    }
    if let Ok(mut text) = q_hull.single_mut() {
        let hull = q_ship
            .single()
            .map(|h| h.current.max(0.0) as usize)
            .unwrap_or(0);
        **text = format!("Hull: {hull}");
    }
}

// ---------------------------------------------------------------------------
// Ship control
// ---------------------------------------------------------------------------

/// Drive the ship each frame from keyboard or pointer input: turn, thrust with
/// inertia, clamp to the arena (reflecting off the walls), and push the result
/// into the kinematic body's `LinearVelocity`. The visual cone child is rotated
/// to the heading; the flame is shown while thrusting.
#[allow(clippy::too_many_arguments)]
fn control_ship(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    pointer: Res<Pointer>,
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut q_ship: Query<(&mut Ship, &Transform, &mut LinearVelocity)>,
    mut q_model: Query<&mut Transform, (With<ShipModel>, Without<Ship>)>,
    mut q_flame: Query<&mut Visibility, With<ShipFlame>>,
) {
    let dt = time.delta_secs();
    let Ok((mut ship, transform, mut lin_vel)) = q_ship.single_mut() else {
        return;
    };

    if ship.invuln > 0.0 {
        ship.invuln = (ship.invuln - dt).max(0.0);
    }
    if ship.fire_cooldown > 0.0 {
        ship.fire_cooldown = (ship.fire_cooldown - dt).max(0.0);
    }

    // Where is the pointer on the play plane, if it is down?
    let (cam, cam_transform) = *camera;
    let pointer_target = if pointer.pressed {
        pointer
            .screen_pos
            .and_then(|pos| pointer_on_play_plane(pos, cam, cam_transform))
    } else {
        None
    };

    // Turn: pointer aim wins when the pointer is held, else keyboard rotation.
    if let Some(target) = pointer_target {
        let to_target = (target - transform.translation).truncate();
        if to_target.length_squared() > 1e-4 {
            let desired = to_target.y.atan2(to_target.x);
            ship.heading = rotate_toward(ship.heading, desired, SHIP_TURN_SPEED * dt);
        }
    } else {
        let mut turn = 0.0;
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
            turn += 1.0;
        }
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
            turn -= 1.0;
        }
        ship.heading = wrap_angle(ship.heading + turn * SHIP_TURN_SPEED * dt);
    }

    let forward = Vec3::new(ship.heading.cos(), ship.heading.sin(), 0.0);

    // Thrust: keyboard, or holding the pointer down.
    let thrusting =
        keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) || pointer_target.is_some();
    if thrusting {
        ship.velocity += forward * SHIP_THRUST * dt;
    }
    // Exponential drag, then clamp to top speed.
    ship.velocity *= (1.0 - SHIP_DRAG * dt).clamp(0.0, 1.0);
    if ship.velocity.length() > SHIP_MAX_SPEED {
        ship.velocity = ship.velocity.normalize() * SHIP_MAX_SPEED;
    }

    // Keep the ship inside the arena: a kinematic body is not stopped by the
    // static walls, so reflect its velocity at the bounds ourselves.
    let bound = ARENA_HALF - SHIP_RADIUS;
    let next = transform.translation + ship.velocity * dt;
    if next.x.abs() > bound && next.x * ship.velocity.x > 0.0 {
        ship.velocity.x = -ship.velocity.x * 0.5;
    }
    if next.y.abs() > bound && next.y * ship.velocity.y > 0.0 {
        ship.velocity.y = -ship.velocity.y * 0.5;
    }
    lin_vel.0 = ship.velocity;

    // Point the visual cone (+Y local) along the heading, and show the flame.
    if let Ok(mut model) = q_model.single_mut() {
        model.rotation = Quat::from_rotation_z(ship.heading - FRAC_PI_2);
    }
    if let Ok(mut flame) = q_flame.single_mut() {
        *flame = if thrusting {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Fire bullets on Space (keyboard) or while the pointer is held, on a cooldown.
fn fire_bullets(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    pointer: Res<Pointer>,
    assets: Res<GameAssets>,
    sfx: Res<SfxAssets>,
    mut q_ship: Query<(&mut Ship, &Transform)>,
) {
    let Ok((mut ship, transform)) = q_ship.single_mut() else {
        return;
    };
    let wants_fire = keys.pressed(KeyCode::Space) || pointer.pressed;
    if !wants_fire || ship.fire_cooldown > 0.0 {
        return;
    }
    ship.fire_cooldown = FIRE_COOLDOWN;

    let forward = Vec3::new(ship.heading.cos(), ship.heading.sin(), 0.0);
    let muzzle = transform.translation + forward * (SHIP_RADIUS + BULLET_RADIUS + 0.4);
    // Inherit the ship's velocity so shots track the ship's drift.
    let velocity = forward * BULLET_SPEED + ship.velocity;

    commands.spawn((
        Name::new("Bullet"),
        Bullet {
            life: BULLET_LIFETIME,
        },
        Mesh3d(assets.bullet_mesh.clone()),
        MeshMaterial3d(assets.bullet_material.clone()),
        Transform::from_translation(muzzle),
        RigidBody::Kinematic,
        Collider::sphere(BULLET_RADIUS),
        Sensor,
        LinearVelocity(velocity),
        LockedAxes::new().lock_translation_z(),
        CollisionLayers::new([GameLayer::Bullet], [GameLayer::Asteroid]),
        CollisionEventsEnabled,
        DespawnOnExit(GameState::Playing),
    ));

    // A soft, slightly pitch-varied blip so rapid fire does not drone.
    let mut rng = rand::rng();
    commands.trigger(
        PlaySfx::new(sfx.shot.clone())
            .with_volume(0.25)
            .with_speed(rng.random_range(1.3..1.6)),
    );
}

/// Age out bullets that never hit anything, and despawn any that leave the arena.
fn tick_bullets(
    mut commands: Commands,
    time: Res<Time>,
    mut q_bullet: Query<(Entity, &mut Bullet, &Transform)>,
) {
    let dt = time.delta_secs();
    let bound = ARENA_HALF + 1.0;
    for (entity, mut bullet, transform) in q_bullet.iter_mut() {
        bullet.life -= dt;
        let out = transform.translation.x.abs() > bound || transform.translation.y.abs() > bound;
        if bullet.life <= 0.0 || out {
            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Asteroids and collisions
// ---------------------------------------------------------------------------

/// Spawn a single asteroid body at `pos` of the given generation, drifting at
/// `velocity` with a random tumble. Rocks are avian dynamic bodies that bounce
/// off the walls but pass through each other (`CollisionLayers`), pinned to the
/// XY plane by `LockedAxes`.
#[allow(clippy::too_many_arguments)]
fn spawn_asteroid(
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    pos: Vec3,
    generation: usize,
    velocity: Vec3,
    spin: f32,
) {
    let radius = ASTEROID_RADII[generation.min(MAX_SPLIT_GEN)];
    commands.spawn((
        Name::new("Asteroid"),
        Asteroid { generation },
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(pos.with_z(0.0)),
        RigidBody::Dynamic,
        Collider::sphere(radius),
        LinearVelocity(velocity.with_z(0.0)),
        AngularVelocity(Vec3::new(0.0, 0.0, spin)),
        Restitution::new(ASTEROID_RESTITUTION).with_combine_rule(CoefficientCombine::Max),
        Friction::ZERO,
        LockedAxes::new()
            .lock_translation_z()
            .lock_rotation_x()
            .lock_rotation_y(),
        CollisionLayers::new(
            [GameLayer::Asteroid],
            [GameLayer::Wall, GameLayer::Ship, GameLayer::Bullet],
        ),
        DespawnOnExit(GameState::Playing),
    ));
}

/// Spawn the current wave (the `OnEnter(Playing)` system).
fn spawn_wave(mut commands: Commands, wave: Res<Wave>, assets: Res<GameAssets>) {
    populate_wave(&mut commands, wave.0, &assets);
}

/// Spawn `asteroids_in_wave` generation-0 rocks along the arena edges, drifting
/// inward at a wave-scaled speed. Shared by the initial spawn and `advance_wave`.
fn populate_wave(commands: &mut Commands, wave: usize, assets: &GameAssets) {
    let mut rng = rand::rng();
    let count = asteroids_in_wave(wave);
    let speed_cap = asteroid_drift_speed(wave);
    for _ in 0..count {
        // Pick a spot near an edge and aim generally toward the middle.
        let edge = rng.random_range(-ARENA_HALF..ARENA_HALF);
        let pos = if rng.random_bool(0.5) {
            Vec3::new(edge, ARENA_HALF - 0.5, 0.0) * if rng.random_bool(0.5) { 1.0 } else { -1.0 }
        } else {
            Vec3::new(ARENA_HALF - 0.5, edge, 0.0) * if rng.random_bool(0.5) { 1.0 } else { -1.0 }
        };
        let inward = (-pos).truncate().normalize_or_zero();
        let jitter = rng.random_range(-0.6..0.6);
        let dir = Vec2::from_angle(jitter).rotate(inward);
        let speed = rng.random_range(ASTEROID_SPEED_MIN..speed_cap.max(ASTEROID_SPEED_MIN + 0.1));
        let velocity = Vec3::new(dir.x, dir.y, 0.0) * speed;
        let spin = rng.random_range(-ASTEROID_SPIN_MAX..ASTEROID_SPIN_MAX);
        let mesh = assets.rock_meshes[rng.random_range(0..assets.rock_meshes.len())].clone();
        spawn_asteroid(
            commands,
            mesh,
            assets.rock_material.clone(),
            pos,
            0,
            velocity,
            spin,
        );
    }
}

/// Keep rock speeds inside a sane band: near-elastic bounces can nudge speeds up
/// over time, and a rock that stalls dead is no fun, so clamp both ends.
fn clamp_asteroid_speed(mut q: Query<&mut LinearVelocity, With<Asteroid>>) {
    for mut lin in q.iter_mut() {
        let speed = lin.0.length();
        if speed > ASTEROID_SPEED_MAX {
            lin.0 = lin.0 / speed * ASTEROID_SPEED_MAX;
        } else if speed < ASTEROID_SPEED_MIN && speed > 1e-3 {
            lin.0 = lin.0 / speed * ASTEROID_SPEED_MIN;
        }
    }
}

/// Resolve avian collision events: bullets destroy or split rocks; a rock that
/// touches the ship costs a hull point (unless the ship is invulnerable).
#[allow(clippy::too_many_arguments)]
fn handle_collisions(
    mut events: MessageReader<CollisionStart>,
    mut commands: Commands,
    q_bullet: Query<(), With<Bullet>>,
    q_asteroid: Query<(&Asteroid, Has<Exploding>)>,
    mut q_ship: Query<(Entity, &mut Ship)>,
    mut q_shake: Query<&mut CameraShakeInput>,
    mut score: ResMut<Score>,
    sfx: Res<SfxAssets>,
) {
    let mut handled_asteroids: HashSet<Entity> = HashSet::new();
    let mut despawned_bullets: HashSet<Entity> = HashSet::new();
    let Ok((ship_entity, mut ship)) = q_ship.single_mut() else {
        return;
    };
    let mut shake = q_shake.single_mut().ok();

    for event in events.read() {
        let (a, b) = (event.collider1, event.collider2);

        // Bullet vs asteroid.
        let bullet_hit = if q_bullet.contains(a) && q_asteroid.contains(b) {
            Some((a, b))
        } else if q_bullet.contains(b) && q_asteroid.contains(a) {
            Some((b, a))
        } else {
            None
        };
        if let Some((bullet, asteroid)) = bullet_hit {
            if despawned_bullets.insert(bullet) {
                commands.entity(bullet).despawn();
            }
            if !handled_asteroids.insert(asteroid) {
                continue;
            }
            let Ok((rock, exploding)) = q_asteroid.get(asteroid) else {
                continue;
            };
            if exploding {
                continue;
            }
            **score += 1;
            if rock.generation >= MAX_SPLIT_GEN {
                // Smallest shard: destroy it outright, no new bodies.
                commands.entity(asteroid).despawn();
                if let Some(input) = shake.as_mut() {
                    input.add_trauma += SHAKE_SPLIT * 0.5;
                }
                commands.trigger(
                    PlaySfx::new(sfx.explode.clone())
                        .with_volume(0.5)
                        .with_speed(1.5),
                );
            } else {
                // Slice it: `on_fragments_spawned` respawns the shards as bodies.
                commands.entity(asteroid).insert((
                    Exploding,
                    ExplodeMesh {
                        fragment_count: FRAGMENTS_PER_SPLIT,
                    },
                ));
                if let Some(input) = shake.as_mut() {
                    input.add_trauma += SHAKE_SPLIT;
                }
                let speed = 1.0 + rock.generation as f32 * 0.35;
                commands.trigger(
                    PlaySfx::new(sfx.explode.clone())
                        .with_volume(0.8)
                        .with_speed(speed),
                );
            }
            continue;
        }

        // Ship vs asteroid.
        let ship_hit = (a == ship_entity && q_asteroid.contains(b))
            || (b == ship_entity && q_asteroid.contains(a));
        if ship_hit && ship.invuln <= 0.0 {
            ship.invuln = INVULN_TIME;
            if let Some(input) = shake.as_mut() {
                input.add_trauma += SHAKE_HIT;
            }
            commands.play_sfx(sfx.hurt.clone());
            commands.trigger(HealthApplyDamage {
                entity: ship_entity,
                source: None,
                amount: 1.0,
            });
        }
    }
}

/// When `ExplodeMeshPlugin` slices a rock, respawn each shard as a smaller avian
/// body: it inherits the parent's drift, gets an outward burst along its slice
/// direction, and a random tumble. The shard mesh is recentered on its own
/// centroid so it tumbles about its middle rather than the old rock center.
fn on_fragments_spawned(
    insert: On<Insert, ExplodeFragments>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    q: Query<(
        &ExplodeFragments,
        &Transform,
        &LinearVelocity,
        &MeshMaterial3d<StandardMaterial>,
        &Asteroid,
    )>,
) {
    let entity = insert.entity;
    let Ok((fragments, transform, lin_vel, material, rock)) = q.get(entity) else {
        return;
    };

    let child_gen = rock.generation + 1;
    let parent_vel = lin_vel.0;
    let mut rng = rand::rng();

    for fragment in fragments.iter() {
        let Some(base) = meshes.get(&fragment.mesh) else {
            continue;
        };
        let centroid = mesh_centroid(base);
        // Recenter the shard on its centroid so it spins naturally in place.
        let recentered = base
            .clone()
            .transformed_by(Transform::from_translation(-centroid));
        let mesh = meshes.add(recentered);

        // Place the shard where it actually was on the rock, and push it out
        // along its slice normal (both rotated into world space).
        let world_offset = transform.rotation * centroid;
        let world_dir = (transform.rotation * fragment.direction.as_vec3()).normalize_or_zero();
        let velocity = parent_vel + world_dir * SPLIT_SPEED;
        let spin = rng.random_range(-ASTEROID_SPIN_MAX * 2.0..ASTEROID_SPIN_MAX * 2.0);

        spawn_asteroid(
            &mut commands,
            mesh,
            material.0.clone(),
            transform.translation + world_offset,
            child_gen,
            velocity,
            spin,
        );
    }

    // Remove the sliced shell; the shards above have replaced it.
    commands.entity(entity).despawn();
}

/// When no asteroids remain, clear the wave: bump the wave number, flash a
/// banner via the wave sound, and spawn the next, busier wave.
fn advance_wave(
    mut commands: Commands,
    q_asteroids: Query<(), With<Asteroid>>,
    dying: Res<DyingTimer>,
    mut wave: ResMut<Wave>,
    assets: Res<GameAssets>,
    sfx: Res<SfxAssets>,
) {
    // Do not spawn a new wave mid-death.
    if dying.remaining.is_some() || !q_asteroids.is_empty() {
        return;
    }
    **wave += 1;
    commands.play_sfx_volume(sfx.wave_clear.clone(), 0.8);
    populate_wave(&mut commands, wave.0, &assets);
}

/// Blink the ship's model while it is invulnerable so the i-frames read clearly.
fn blink_invulnerable_ship(
    time: Res<Time>,
    q_ship: Query<&Ship>,
    mut q_model: Query<&mut Visibility, With<ShipModel>>,
) {
    let Ok(ship) = q_ship.single() else {
        return;
    };
    let visible = ship.invuln <= 0.0 || (time.elapsed_secs() * 18.0).sin() > 0.0;
    for mut visibility in q_model.iter_mut() {
        *visibility = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Draw the arena boundary as a glowing rectangle while playing.
fn draw_arena_border(mut gizmos: Gizmos) {
    gizmos.rect(
        Isometry3d::from_translation(Vec3::ZERO),
        Vec2::splat(ARENA_HALF * 2.0),
        Color::srgba(0.3, 0.6, 0.9, 0.8),
    );
}

// ---------------------------------------------------------------------------
// Death and game over
// ---------------------------------------------------------------------------

/// When the ship's hull hits zero, kick a big shake and a red flash, then head to
/// the game-over screen after a short beat.
fn on_player_died(
    add: On<Add, HealthZeroMarker>,
    q_ship: Query<(), With<Ship>>,
    mut commands: Commands,
    mut q_shake: Query<&mut CameraShakeInput>,
    mut dying: ResMut<DyingTimer>,
) {
    if !q_ship.contains(add.entity) || dying.remaining.is_some() {
        return;
    }
    // Adding 1.0 clamps trauma to full, kicking the biggest shake.
    if let Ok(mut input) = q_shake.single_mut() {
        input.add_trauma += 1.0;
    }
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

/// Count down the death beat and switch to the game-over screen, fading the flash.
fn advance_dying(
    time: Res<Time>,
    mut dying: ResMut<DyingTimer>,
    mut next: ResMut<NextState<GameState>>,
    mut q_flash: Query<(&mut RedFlash, &mut BackgroundColor)>,
) {
    let Some(remaining) = dying.remaining.as_mut() else {
        return;
    };
    let dt = time.delta_secs();
    for (mut flash, mut background) in q_flash.iter_mut() {
        flash.age += dt;
        let alpha = (1.0 - flash.age / flash.lifetime).clamp(0.0, 1.0) * 0.5;
        background.0 = Color::srgba(0.9, 0.1, 0.1, alpha);
    }
    *remaining -= dt;
    if *remaining <= 0.0 {
        dying.remaining = None;
        next.set(GameState::GameOver);
    }
}

/// Give up the current run with Escape.
fn giveup_on_escape(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next.set(GameState::GameOver);
    }
}

fn record_high_score(
    score: Res<Score>,
    mut high: ResMut<HighScore>,
    mut new_best: ResMut<NewBest>,
) {
    new_best.0 = score.0 > high.0;
    high.0 = high.0.max(score.0);
}

fn spawn_game_over(
    mut commands: Commands,
    score: Res<Score>,
    wave: Res<Wave>,
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
            parent.spawn(screen_text("GAME OVER", 72.0, Color::srgb(0.9, 0.3, 0.3)));
            parent.spawn(screen_text(
                format!("Score: {} - reached wave {}", score.0, wave.0),
                36.0,
                Color::srgb(0.6, 0.9, 1.0),
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
            parent.spawn(screen_text(
                "Tap or click to return to menu",
                28.0,
                Color::WHITE,
            ));
        });
}

fn play_game_over_sfx(mut commands: Commands, sfx: Res<SfxAssets>) {
    commands.play_sfx_volume(sfx.game_over.clone(), 0.9);
}

fn gameover_click(
    pointer: Res<Pointer>,
    keys: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if pointer.just_pressed || keys.just_pressed(KeyCode::Space) {
        next.set(GameState::Menu);
    }
}

// ---------------------------------------------------------------------------
// Pure helpers (unit tested)
// ---------------------------------------------------------------------------

/// Wrap an angle to the range `(-PI, PI]`.
fn wrap_angle(angle: f32) -> f32 {
    let mut a = (angle + PI).rem_euclid(TAU) - PI;
    if a <= -PI {
        a += TAU;
    }
    a
}

/// Signed smallest angular difference `target - current`, in `(-PI, PI]`.
fn angle_diff(current: f32, target: f32) -> f32 {
    wrap_angle(target - current)
}

/// Rotate `current` toward `target` by at most `max_step` radians (shortest way).
fn rotate_toward(current: f32, target: f32, max_step: f32) -> f32 {
    let diff = angle_diff(current, target);
    if diff.abs() <= max_step {
        wrap_angle(target)
    } else {
        wrap_angle(current + max_step * diff.signum())
    }
}

/// Number of generation-0 rocks a wave spawns.
fn asteroids_in_wave(wave: usize) -> usize {
    3 + wave
}

/// Upper drift-speed bound for a wave's rocks, climbing with the wave then capped.
fn asteroid_drift_speed(wave: usize) -> f32 {
    (ASTEROID_SPEED_BASE + wave.saturating_sub(1) as f32 * ASTEROID_SPEED_PER_WAVE)
        .min(ASTEROID_SPEED_MAX)
}

/// Centroid (mean vertex position) of a mesh's position attribute.
fn mesh_centroid(mesh: &Mesh) -> Vec3 {
    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        if positions.is_empty() {
            return Vec3::ZERO;
        }
        let sum: Vec3 = positions.iter().map(|p| Vec3::from_array(*p)).sum();
        sum / positions.len() as f32
    } else {
        Vec3::ZERO
    }
}

/// World position where the pointer ray meets the play plane (Z = 0).
fn pointer_on_play_plane(
    screen_pos: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec3> {
    let ray = camera
        .viewport_to_world(camera_transform, screen_pos)
        .ok()?;
    let plane = InfinitePlane3d::new(Vec3::Z);
    let distance = ray.intersect_plane(Vec3::ZERO, plane)?;
    Some(ray.get_point(distance))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_angle_folds_into_range() {
        assert!((wrap_angle(0.0)).abs() < 1e-6);
        assert!((wrap_angle(TAU) - 0.0).abs() < 1e-5);
        assert!((wrap_angle(3.0 * PI) - PI).abs() < 1e-5);
        let w = wrap_angle(-3.0 * PI);
        assert!(w > -PI - 1e-5 && w <= PI + 1e-5);
    }

    #[test]
    fn angle_diff_takes_the_short_way() {
        // From just below PI to just above -PI is a small positive step across
        // the seam, not nearly a full turn.
        let d = angle_diff(PI - 0.1, -PI + 0.1);
        assert!(d.abs() < 0.3, "expected small diff, got {d}");
    }

    #[test]
    fn rotate_toward_steps_and_arrives() {
        // A step smaller than the gap moves partway.
        let stepped = rotate_toward(0.0, 1.0, 0.25);
        assert!((stepped - 0.25).abs() < 1e-6);
        // A step larger than the gap snaps to the target.
        let arrived = rotate_toward(0.0, 0.1, 0.5);
        assert!((arrived - 0.1).abs() < 1e-6);
    }

    #[test]
    fn rotate_toward_crosses_the_seam_the_short_way() {
        // Turning from just below PI toward just above -PI should increase the
        // angle across the +PI/-PI seam, not spin all the way back.
        let result = rotate_toward(PI - 0.05, -PI + 0.05, 0.02);
        assert!(result > PI - 0.05 || result <= -PI + 0.05 + 1e-6);
    }

    #[test]
    fn waves_grow() {
        assert_eq!(asteroids_in_wave(1), 4);
        assert_eq!(asteroids_in_wave(5), 8);
        assert!(asteroids_in_wave(3) > asteroids_in_wave(1));
    }

    #[test]
    fn drift_speed_climbs_then_caps() {
        assert!((asteroid_drift_speed(1) - ASTEROID_SPEED_BASE).abs() < 1e-6);
        assert!(asteroid_drift_speed(5) > asteroid_drift_speed(1));
        assert!(asteroid_drift_speed(1000) <= ASTEROID_SPEED_MAX + 1e-6);
    }

    #[test]
    fn centroid_of_symmetric_mesh_is_origin() {
        let mesh = TriangleMeshBuilder::new_octahedron(2).build();
        let c = mesh_centroid(&mesh);
        assert!(
            c.length() < 1e-3,
            "octahedron centroid should be ~origin, got {c:?}"
        );
    }

    #[test]
    fn centroid_tracks_a_translated_mesh() {
        let mesh = TriangleMeshBuilder::new_octahedron(2)
            .build()
            .translated_by(Vec3::new(5.0, 0.0, 0.0));
        let c = mesh_centroid(&mesh);
        assert!(
            (c.x - 5.0).abs() < 1e-3,
            "expected centroid near x=5, got {c:?}"
        );
    }

    /// The headline behavior, headless: a hit rock (generation 0) slices via
    /// `ExplodeMeshPlugin` and the `on_fragments_spawned` observer respawns each
    /// shard as a real avian body of the next generation, inheriting the parent's
    /// drift and carrying its own collider -- unlike `06_fruitninja`, where the
    /// shards are throwaway visuals. This drives the exact runtime path a bullet
    /// hit triggers, minus the collision event, so CI can verify it without a
    /// window (the graphical example cannot be a CI test).
    #[test]
    fn splitting_a_rock_spawns_smaller_physics_bodies() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Mesh>();
        app.init_asset::<StandardMaterial>();
        app.add_plugins(ExplodeMeshPlugin);
        app.add_observer(on_fragments_spawned);

        let mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(
            TriangleMeshBuilder::new_octahedron(3)
                .build()
                .scaled_by(Vec3::splat(ASTEROID_RADII[0])),
        );
        let material = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial::default());

        // A generation-0 rock drifting along +X, then hit (ExplodeMesh inserted).
        let drift = Vec3::new(2.0, 0.0, 0.0);
        let rock = app
            .world_mut()
            .spawn((
                Asteroid { generation: 0 },
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_xyz(1.0, 2.0, 0.0),
                RigidBody::Dynamic,
                Collider::sphere(ASTEROID_RADII[0]),
                LinearVelocity(drift),
                ExplodeMesh {
                    fragment_count: FRAGMENTS_PER_SPLIT,
                },
            ))
            .id();

        // One update slices (plugin inserts ExplodeFragments -> observer runs),
        // a second flushes the observer's spawn/despawn commands.
        app.update();
        app.update();

        // The original shell is gone.
        assert!(
            app.world().get_entity(rock).is_err(),
            "the sliced rock should have despawned"
        );

        // Its shards exist as generation-1 avian bodies with colliders, each
        // carrying the parent's inherited drift plus an outward burst.
        let mut query = app
            .world_mut()
            .query::<(&Asteroid, &RigidBody, &Collider, &LinearVelocity)>();
        let shards: Vec<_> = query.iter(app.world()).collect();
        assert!(
            shards.len() >= 2,
            "expected at least two shard bodies, got {}",
            shards.len()
        );
        for (asteroid, body, _collider, velocity) in &shards {
            assert_eq!(asteroid.generation, 1, "shards are the next generation");
            assert!(
                matches!(body, RigidBody::Dynamic),
                "shards are dynamic bodies"
            );
            // Each shard inherits the parent's drift and adds an outward burst
            // along its slice normal (`parent_vel + world_dir * SPLIT_SPEED`).
            // The burst is a 3D direction but the arena is planar, so
            // `spawn_asteroid` drops its z component -- a shard whose burst
            // points mostly out of plane can therefore end up slower than the
            // parent, so we cannot assert `speed >= parent`. What always holds
            // is that the planar velocity is the parent drift plus a burst no
            // larger than `SPLIT_SPEED`: the drift was inherited, not discarded.
            assert!(
                (velocity.0 - drift).length() <= SPLIT_SPEED + 1e-3,
                "a shard should be the parent drift plus a bounded burst, got {} (delta {})",
                velocity.0.length(),
                (velocity.0 - drift).length()
            );
        }
        // ...and the burst is actually applied: at least one shard has to differ
        // from the bare parent drift, or the outward-burst term is dead. (A
        // single shard could coincidentally land near the parent drift when its
        // burst points nearly straight out of plane, so we only require one.)
        assert!(
            shards
                .iter()
                .any(|(_, _, _, velocity)| (velocity.0 - drift).length() > 1e-2),
            "no shard carried an outward burst -- the split velocity is just the parent drift"
        );
    }
}
