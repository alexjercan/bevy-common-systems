//! Orbit Runner: a surface-dodge game played on the outside of a sphere.
//!
//! You pilot a glowing marker that rides the surface of a planet, always moving
//! forward. Steer left and right (A/D, the arrow keys, or by holding the mouse /
//! a finger to one side) to weave across the surface. Bright orbs wander the
//! planet; run one over to collect it and score. Red hazards wander too, and
//! touching one costs you health -- run out and the run ends. Survive longer and
//! the planet grows more crowded and more dangerous.
//!
//! This example exists to exercise the crate's whole spherical-motion family
//! under real gameplay, none of which the other examples demo interactively:
//!
//! - `DirectionalSphereOrbit` drives the player: each frame the game feeds it a
//!   direction vector (the marker's surface normal, advanced along a great
//!   circle) and reads back the world position on the sphere. Steering is just
//!   rotating that direction.
//! - `RandomSphereOrbit` drives every hazard and orb: they pick random target
//!   angles and wander toward them across the surface on their own.
//! - `ChaseCamera` follows the marker with `LerpSnap` smoothing, orbiting the
//!   planet so "up" is always away from the surface and the horizon curves.
//! - `HealthPlugin` owns the lose condition: a hazard deals damage, and the
//!   `HealthZeroMarker` it inserts at zero health ends the run. A hit also jolts
//!   the camera (a decaying shake layered after `ChaseCamera` writes its
//!   transform) and flashes a red overlay, so impacts read viscerally.
//! - `SfxPlugin` plays a one-shot for every event (menu, pickup, hurt, level up,
//!   game over); the files in `assets/sounds/` are generated placeholders (see
//!   `assets/sounds/README.md`). Collecting orbs in quick succession builds a
//!   streak: each orb is worth more, the pickup blip climbs in pitch, a rising
//!   `combo` chime layers in, and a "+N" popup plus a "STREAK xN" banner flash.
//!   Taking a hazard hit breaks the streak, so a clean run is worth chasing.
//!
//! It follows the shape of `06_fruitninja`: Bevy states for menu / playing /
//! game-over, a persistent camera and light, and an in-game HUD, plus the same
//! wasm/trunk web build so it can ship in the showcase gallery. All the geometry
//! is procedural (`TriangleMeshBuilder`), so it grows straight out of
//! `examples/01_sphere`.

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
use clap::Parser;
use rand::Rng;

#[derive(Parser)]
#[command(name = "07_orbit")]
#[command(version = "1.0.0")]
#[command(
    about = "Ride a marker around a planet: collect the orbs, dodge the hazards. Steer with A/D, the arrow keys, or by holding the mouse / a finger to one side.",
    long_about = None
)]
struct Cli;

/// Radius of the planet the game is played on, in world units.
const PLANET_RADIUS: f32 = 8.0;

/// How far above the surface the marker and the wandering objects float, so
/// they read as sitting *on* the planet rather than clipping into it. The orbit
/// radius each thing rides at is `PLANET_RADIUS + <lift>`.
const RUNNER_LIFT: f32 = 0.4;
const OBJECT_LIFT: f32 = 0.5;

/// Collision radii (world units). A collision is a plain sphere overlap between
/// the marker and an object, both riding just above the surface.
const RUNNER_RADIUS: f32 = 0.5;
const HAZARD_RADIUS: f32 = 0.7;
const ORB_RADIUS: f32 = 0.55;

/// Marker travel speed along the surface, in world units per second. Divided by
/// the orbit radius it becomes the angular rate the marker sweeps the planet.
const RUNNER_SPEED: f32 = 7.0;

/// How fast steering swings the marker's heading, in radians per second.
const TURN_RATE: f32 = 2.6;

/// Points awarded for collecting one orb. The run score also counts one point
/// per second survived (see `score_value`).
const ORB_POINTS: usize = 10;

/// How many orbs are kept on the planet at once; collecting one tops the field
/// back up to this count.
const ORB_COUNT: usize = 5;

/// Minimum angular gap (radians) between a fresh spawn and the marker, so a
/// hazard or orb never materializes on top of the player. About 34 degrees.
const MIN_SPAWN_SEPARATION: f32 = 0.6;

/// Hazard count ramps from `HAZARD_START` up to `HAZARD_MAX`, one extra hazard
/// per level, so the planet gets more crowded the longer you last.
const HAZARD_START: usize = 3;
const HAZARD_MAX: usize = 12;

/// Seconds of survival per difficulty level. The level drives the hazard count
/// and the wander speed, and each new level pings a sound.
const LEVEL_SECS: f32 = 20.0;

/// Wander speed (radians/sec) hazards and orbs sweep the surface at: eases up
/// from the level-1 value toward the cap as the level climbs.
const WANDER_SPEED_START: f32 = 0.35;
const WANDER_SPEED_MAX: f32 = 0.9;
const WANDER_SPEED_LEVELS: f32 = 8.0;

/// Player health at the start of a run and the damage one hazard deals, so the
/// run survives three hazard touches. It is a real `Health` value, so the
/// example drives the crate's health system end to end.
const PLAYER_HEALTH: f32 = 3.0;
const HAZARD_DAMAGE: f32 = 1.0;

/// Seconds of invulnerability after a hazard touch, so a single overlap does not
/// drain all health at once. The marker blinks while it lasts.
const HIT_COOLDOWN: f32 = 1.3;

/// Chase-camera smoothing (the `LerpSnap` factor): higher lags more. This is the
/// one knob that makes the camera glide instead of snapping to the marker.
const CAMERA_SMOOTHING: f32 = 0.12;

/// Seconds after collecting an orb you have to grab the next one to keep the
/// streak alive. Orbs are more spread out than fruit-ninja slices, so the
/// window is a touch longer. When it lapses the streak resets to zero.
const STREAK_WINDOW: f32 = 1.6;

/// Pickup pitch climbs with the streak: each orb past the first nudges
/// `pickup.wav`'s playback speed up by `PICKUP_PITCH_STEP`, capped at
/// `PICKUP_PITCH_MAX`, so a run of pickups rises in pitch.
const PICKUP_PITCH_STEP: f32 = 0.07;
const PICKUP_PITCH_MAX: f32 = 1.7;

/// Upward screen speed (px/s) of the "STREAK xN" banner. Slower than the crate
/// `Popup` default so a hot streak lingers a beat longer than a "+N".
const STREAK_RISE_SPEED: f32 = 28.0;

/// Camera-shake feel on a hazard hit. `trauma` is 0..1; it decays at
/// `SHAKE_DECAY` per second and, squared, scales a random camera offset up to
/// `SHAKE_MAX_OFFSET` world units. `HAZARD_TRAUMA` is the jolt one hit adds.
/// Kept small on purpose: this is a smooth chase-camera game, not a fixed one,
/// so a big shake would fight the `LerpSnap` glide.
const SHAKE_DECAY: f32 = 2.2;
const SHAKE_MAX_OFFSET: f32 = 0.5;
const HAZARD_TRAUMA: f32 = 0.7;

/// Damage-flash feel: a hazard hit spikes a red full-screen overlay to
/// `DAMAGE_FLASH_PEAK_ALPHA` and it fades out at `DAMAGE_FLASH_DECAY` per
/// second, so a hit is unmissable even when it lands off-camera.
const DAMAGE_FLASH_PEAK_ALPHA: f32 = 0.38;
const DAMAGE_FLASH_DECAY: f32 = 3.0;

fn main() {
    let _ = Cli::parse();
    let mut app = App::new();

    // On the web the game runs inside a canvas: fit it to its parent element so
    // it fills the frame the showcase site embeds it in. These fields are
    // ignored on native, so the desktop example is unchanged.
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

    // The crate systems this example is built to showcase.
    app.add_plugins(DirectionalSphereOrbitPlugin);
    app.add_plugins(SphereRandomOrbitPlugin);
    app.add_plugins(ChaseCameraPlugin);
    // Trauma-driven camera shake; the camera carries a `CameraShake` and game
    // code adds trauma through its `CameraShakeInput`.
    app.add_plugins(CameraShakePlugin);
    app.add_plugins(HealthPlugin);
    app.add_plugins(SfxPlugin);
    app.add_plugins(StatusBarPlugin);
    // Floating "+N" / streak popups rise and fade via the crate's PopupPlugin.
    app.add_plugins(PopupPlugin);
    app.add_plugins(MenuPlugin);
    // Full-screen red damage overlay spiked on a hazard hit, via ScreenFlashPlugin.
    app.add_plugins(ScreenFlashPlugin);

    app.init_state::<GameState>();

    app.init_resource::<Score>();
    app.init_resource::<HighScore>();
    app.init_resource::<NewBest>();
    app.init_resource::<Elapsed>();
    app.init_resource::<Level>();
    app.init_resource::<HitCooldown>();
    app.insert_resource(Streak::new(STREAK_WINDOW));

    // Persistent scene: camera, light, planet and the FPS status bar live for
    // the whole run, independent of game state.
    app.add_systems(Startup, setup);

    // Main menu.
    app.add_systems(OnEnter(GameState::Menu), spawn_menu);
    app.add_systems(Update, advance_from_menu.run_if(in_state(GameState::Menu)));

    // Playing: reset the run, spawn the marker + HUD, then run the gameplay loop.
    app.add_systems(
        OnEnter(GameState::Playing),
        (start_game, spawn_runner, spawn_hud),
    );
    app.add_systems(
        Update,
        (
            tick_elapsed,
            advance_level,
            steer_runner,
            maintain_objects,
            tick_hit_cooldown,
            tick_streak,
            blink_runner,
            update_hud,
            giveup_on_escape,
        )
            .run_if(in_state(GameState::Playing)),
    );

    // Position work happens in PostUpdate, after the orbit plugins have turned
    // this frame's inputs into surface positions and before the chase camera
    // syncs its transform, so the camera frames the marker's new position with
    // no one-frame lag.
    app.add_systems(
        PostUpdate,
        (
            apply_runner_transform,
            apply_object_transforms,
            resolve_collisions,
            drive_chase_camera,
        )
            .chain()
            .after(DirectionalSphereOrbitSystems::Sync)
            .after(SphereRandomOrbitSystems::Sync)
            .before(ChaseCameraSystems::Sync)
            .run_if(in_state(GameState::Playing)),
    );

    // Game over screen.
    app.add_systems(
        OnEnter(GameState::GameOver),
        (record_high_score, spawn_game_over, play_game_over_sfx).chain(),
    );
    app.add_systems(
        Update,
        advance_from_game_over.run_if(in_state(GameState::GameOver)),
    );

    app.add_observer(on_runner_died);

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

/// Points scored from collecting orbs this run. Each orb is worth
/// `ORB_POINTS` times the current streak length (see `orb_points_for`), so a
/// chain of quick pickups is worth much more than the same orbs collected
/// slowly. The displayed score folds in survival time too (see `score_value`).
#[derive(Resource, Default, Deref, DerefMut)]
struct Score(usize);

/// Best score across runs this session (not reset per run).
#[derive(Resource, Default)]
struct HighScore(usize);

/// Whether the most recent run set a new high score (for the game-over screen).
#[derive(Resource, Default)]
struct NewBest(bool);

/// Seconds elapsed in the current run, driving the score and the difficulty
/// level.
#[derive(Resource, Default)]
struct Elapsed(f32);

/// Current difficulty level (1-based). Bumped every `LEVEL_SECS` seconds; drives
/// the hazard count and wander speed.
#[derive(Resource)]
struct Level(usize);

impl Default for Level {
    fn default() -> Self {
        Self(1)
    }
}

/// Time left on the post-hit invulnerability window; the marker takes no damage
/// and blinks while this is above zero.
#[derive(Resource, Default)]
struct HitCooldown(f32);

/// The player marker. Holds the moving frame on the sphere surface: `up` is the
/// outward surface normal (also the direction fed to the orbit), and `forward`
/// is the unit tangent it is travelling along. Steering rotates `forward` about
/// `up`; advancing rotates both along a great circle.
#[derive(Component)]
struct Runner {
    up: Vec3,
    forward: Vec3,
}

/// Marker for a wandering hazard (touching it hurts).
#[derive(Component)]
struct Hazard;

/// Marker for a wandering orb (collecting it scores).
#[derive(Component)]
struct Orb;

/// Marker for the main camera.
#[derive(Component)]
struct MainCamera;

/// Marker for the score HUD text.
#[derive(Component)]
struct ScoreText;

/// Marker for the level HUD text.
#[derive(Component)]
struct LevelText;

/// Marker for the health-bar fill node (its width tracks current health).
#[derive(Component)]
struct HealthBarFill;

/// Marker for the single "STREAK xN" banner, so a fresh one can replace the
/// previous banner instead of overprinting it during a fast chain.
#[derive(Component)]
struct StreakBanner;

/// Marker for the full-screen red damage-flash overlay.
#[derive(Component)]
struct DamageFlashOverlay;

/// Shared render assets so the marker, hazards and orbs are cheap to spawn.
#[derive(Resource)]
struct OrbitAssets {
    runner_mesh: Handle<Mesh>,
    hazard_mesh: Handle<Mesh>,
    orb_mesh: Handle<Mesh>,
    runner_material: Handle<StandardMaterial>,
    hazard_material: Handle<StandardMaterial>,
    orb_material: Handle<StandardMaterial>,
}

/// One `AudioSource` handle per gameplay event. Loaded once at startup; the
/// systems trigger `PlaySfx` with the matching handle. The files under
/// `assets/sounds/` are placeholders (see `assets/sounds/README.md`); drop real
/// audio in at the same paths and nothing here changes.
#[derive(Resource)]
struct SfxAssets {
    /// Starting a run from the menu (or leaving the game-over screen).
    menu_select: Handle<AudioSource>,
    /// An orb is collected.
    pickup: Handle<AudioSource>,
    /// A streak reaches x2 or more (a rising chime layered over the pickup).
    /// Shared with `06_fruitninja`, which uses it for its slice combos.
    combo: Handle<AudioSource>,
    /// A hazard is touched (damage taken).
    hurt: Handle<AudioSource>,
    /// A new difficulty level is reached.
    level_up: Handle<AudioSource>,
    /// The run ends (game-over screen).
    game_over: Handle<AudioSource>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Load one sound per gameplay event. Paths are relative to `assets/`.
    commands.insert_resource(SfxAssets {
        menu_select: asset_server.load("sounds/menu_select.wav"),
        pickup: asset_server.load("sounds/pickup.wav"),
        combo: asset_server.load("sounds/combo.wav"),
        hurt: asset_server.load("sounds/hurt.wav"),
        level_up: asset_server.load("sounds/level_up.wav"),
        game_over: asset_server.load("sounds/game_over.wav"),
    });

    // The planet: a smooth octahedron sphere, straight from `01_sphere`. It is
    // the fixed stage the whole game orbits, so it is spawned once here.
    let planet_mesh = meshes.add(TriangleMeshBuilder::new_octahedron(4).build());
    commands.spawn((
        Name::new("Planet"),
        Mesh3d(planet_mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.18, 0.32, 0.45),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Transform::from_scale(Vec3::splat(PLANET_RADIUS)),
    ));

    // Shared meshes/materials for the movers. The marker and orbs are emissive
    // so they pop against the planet; hazards are an angry matte red.
    commands.insert_resource(OrbitAssets {
        runner_mesh: meshes.add(TriangleMeshBuilder::new_octahedron(2).build()),
        hazard_mesh: meshes.add(TriangleMeshBuilder::new_octahedron(1).build()),
        orb_mesh: meshes.add(TriangleMeshBuilder::new_octahedron(2).build()),
        runner_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.95, 1.0),
            emissive: LinearRgba::rgb(0.5, 0.7, 1.0),
            ..default()
        }),
        hazard_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.12, 0.12),
            emissive: LinearRgba::rgb(0.35, 0.02, 0.02),
            perceptual_roughness: 0.5,
            ..default()
        }),
        orb_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.95, 0.7),
            emissive: LinearRgba::rgb(0.1, 0.7, 0.45),
            ..default()
        }),
    });

    // Chase camera. It is spawned once and follows whatever the game writes into
    // its `ChaseCameraInput` each frame; between runs it just holds its last
    // frame. Offset sits it up and behind the marker's surface frame; the focus
    // offset looks ahead along the direction of travel.
    commands.spawn((
        Name::new("Main Camera"),
        MainCamera,
        Camera3d::default(),
        ChaseCamera {
            offset: Vec3::new(0.0, 4.5, -11.0),
            focus_offset: Vec3::new(0.0, 1.5, 16.0),
            smoothing: CAMERA_SMOOTHING,
        },
        // Trauma-driven shake layered on top of the chase framing; game code
        // adds trauma via `CameraShakeInput`. Offsets all three axes like the
        // original hand-rolled shake did.
        CameraShake {
            decay: SHAKE_DECAY,
            max_offset: Vec3::splat(SHAKE_MAX_OFFSET),
            ..default()
        },
        // A little ambient so the dark side of the planet is not pure black.
        // In Bevy 0.19 `AmbientLight` is a per-camera component, not a resource.
        AmbientLight {
            brightness: 250.0,
            ..default()
        },
        Transform::from_xyz(0.0, 6.0, PLANET_RADIUS + 12.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // A key light so the planet and the movers catch highlights.
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 9000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.6, 0.0)),
    ));

    // Status bar: FPS only. The score / health live in the in-game HUD.
    commands.spawn((status_bar(StatusBarRootConfig::default()),));
    commands.spawn((status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: status_fps_value_fn(),
        color_fn: status_fps_color_fn(),
        prefix: "".to_string(),
        suffix: "fps".to_string(),
    }),));
}

// --- Pure helpers (unit-tested below) -------------------------------------

/// Advance the marker's surface frame by one step: apply steering, then travel
/// forward along the great circle.
///
/// `up` is the outward surface normal, `forward` the unit tangent it moves
/// along. `steer` is in -1..1 (negative = left, positive = right). Returns the
/// new `(up, forward)`, kept orthonormal so the marker never drifts off the
/// surface or lets its heading collapse.
fn step_runner_frame(
    up: Vec3,
    forward: Vec3,
    steer: f32,
    turn_rate: f32,
    speed: f32,
    radius: f32,
    dt: f32,
) -> (Vec3, Vec3) {
    let up = up.normalize();
    // Re-orthonormalize the heading against up so accumulated float error can
    // never tilt it into or out of the surface; fall back to any tangent if the
    // two ever line up exactly.
    let forward = (forward - up * forward.dot(up))
        .try_normalize()
        .unwrap_or_else(|| up.any_orthonormal_vector());

    // Steer: swing the heading about the surface normal. Positive steer turns
    // right, which is a negative (clockwise, seen from outside) rotation.
    let forward = (Quat::from_axis_angle(up, -steer * turn_rate * dt) * forward).normalize();

    // Travel: rotate both up and forward along the great circle in the heading
    // direction. The rotation axis is up x forward, and the angle is the arc
    // length (speed * dt) over the radius.
    let axis = up.cross(forward);
    let Some(axis) = axis.try_normalize() else {
        return (up, forward);
    };
    let advance = Quat::from_axis_angle(axis, speed / radius * dt);
    let up = (advance * up).normalize();
    let forward = (advance * forward).normalize();
    (up, forward)
}

/// Build the world rotation of a surface frame: local `-Z` points along
/// `forward` (direction of travel) and local `+Y` along `up` (surface normal).
///
/// This is what both the marker's transform and the chase camera's anchor
/// rotation use, so the camera sits behind-and-above the marker and looks the
/// way it is going.
fn frame_rotation(up: Vec3, forward: Vec3) -> Quat {
    // Callers keep the frame orthonormal, but fall back defensively so a
    // degenerate (zero or parallel) input can never produce a NaN quaternion.
    let up = up.normalize_or(Vec3::Y);
    let forward = forward.normalize_or(Vec3::NEG_Z);
    // right = forward x up gives a right-handed basis with local +Z = -forward
    // (so local -Z faces forward). Rebuilding "back" from right x up keeps the
    // basis exactly orthonormal even if up / forward drifted slightly.
    let right = forward.cross(up).try_normalize().unwrap_or(Vec3::X);
    let back = right.cross(up).normalize_or(Vec3::Z);
    Quat::from_mat3(&Mat3::from_cols(right, up, back))
}

/// True when two surface things (marker vs object) overlap: a plain distance
/// test between their centers against the sum of their radii.
fn spheres_overlap(a: Vec3, b: Vec3, radius_sum: f32) -> bool {
    a.distance_squared(b) <= radius_sum * radius_sum
}

/// A fresh spawn's surface direction is "clear" of the marker when it sits at
/// least `MIN_SPAWN_SEPARATION` radians away from the marker's direction, so
/// nothing ever materializes on top of the player and deals unavoidable damage.
/// Both arguments are unit directions; a larger dot product means a smaller
/// angle, so clear means the dot is below `cos(MIN_SPAWN_SEPARATION)`.
fn spawn_is_clear(candidate_dir: Vec3, runner_dir: Vec3) -> bool {
    candidate_dir.dot(runner_dir) <= MIN_SPAWN_SEPARATION.cos()
}

/// Difficulty level for a given elapsed run time (1-based), one level per
/// `LEVEL_SECS` seconds.
fn level_for(elapsed: f32) -> usize {
    1 + (elapsed / LEVEL_SECS).floor().max(0.0) as usize
}

/// How many hazards the planet should hold at a given level: starts at
/// `HAZARD_START` and adds one per level, capped at `HAZARD_MAX`.
fn hazard_target_for(level: usize) -> usize {
    (HAZARD_START + level.saturating_sub(1)).min(HAZARD_MAX)
}

/// Wander speed (radians/sec) for movers at a given level: eases up from
/// `WANDER_SPEED_START` toward `WANDER_SPEED_MAX`.
fn wander_speed_for(level: usize) -> f32 {
    let t = ((level.saturating_sub(1)) as f32 / WANDER_SPEED_LEVELS).clamp(0.0, 1.0);
    WANDER_SPEED_START + (WANDER_SPEED_MAX - WANDER_SPEED_START) * t
}

/// The displayed run score: the orb points banked this run plus one point per
/// whole second survived.
fn score_value(orb_points: usize, elapsed: f32) -> usize {
    orb_points + elapsed.max(0.0) as usize
}

/// Points a single orb is worth at a given streak length: the base `ORB_POINTS`
/// scaled by the streak, so the 1st orb is worth `ORB_POINTS`, the 2nd twice
/// that, and so on. A lone pickup (streak 1) is unchanged from before.
fn orb_points_for(streak_count: usize) -> usize {
    ORB_POINTS * streak_count.max(1)
}

/// Playback speed for `pickup.wav` at a given streak length: rises with the
/// streak so a chain of pickups climbs in pitch, capped at `PICKUP_PITCH_MAX`.
fn pickup_pitch_for(streak_count: usize) -> f32 {
    (1.0 + streak_count.saturating_sub(1) as f32 * PICKUP_PITCH_STEP).min(PICKUP_PITCH_MAX)
}

// --- Input ----------------------------------------------------------------

/// Steering input in -1..1: negative steers left, positive right. Sums the
/// keyboard (A/D or arrow keys) with a held pointer's horizontal offset from the
/// screen center, so a mouse or a finger works the same as the keys.
fn read_steer(
    keys: &ButtonInput<KeyCode>,
    mouse: &ButtonInput<MouseButton>,
    touches: &Touches,
    window: &Window,
) -> f32 {
    let mut steer = 0.0;
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        steer -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        steer += 1.0;
    }

    // A live touch drives steering; otherwise a held mouse button does. The
    // horizontal distance from the screen center sets how hard we turn.
    let pointer = touches
        .iter()
        .next()
        .map(|touch| touch.position())
        .or_else(|| {
            mouse
                .pressed(MouseButton::Left)
                .then(|| window.cursor_position())
                .flatten()
        });
    if let Some(pos) = pointer {
        let half = (window.width() * 0.35).max(1.0);
        steer += ((pos.x - window.width() * 0.5) / half).clamp(-1.0, 1.0);
    }

    steer.clamp(-1.0, 1.0)
}

/// True on the frame a "continue" press begins: a click, a tap, Space or Enter.
/// Used by the menu and game-over screens.
fn advance_pressed(
    mouse: &ButtonInput<MouseButton>,
    keys: &ButtonInput<KeyCode>,
    touches: &Touches,
) -> bool {
    mouse.just_pressed(MouseButton::Left)
        || keys.just_pressed(KeyCode::Space)
        || keys.just_pressed(KeyCode::Enter)
        || touches.iter_just_pressed().next().is_some()
}

// --- Menu -----------------------------------------------------------------

/// Spawn the main menu (title + prompt), scoped to the `Menu` state.
fn spawn_menu(mut commands: Commands, high: Res<HighScore>) {
    commands.spawn((
        Name::new("Main Menu"),
        DespawnOnExit(GameState::Menu),
        centered_screen(),
        children![
            (
                screen_text("ORBIT RUNNER", 68.0, Color::srgb(0.5, 0.8, 1.0)),
                TitlePulse::new(Color::srgb(0.5, 0.8, 1.0)),
            ),
            screen_text("Tap or click to play", 32.0, Color::WHITE),
            screen_text(
                format!("Best: {}", high.0),
                24.0,
                Color::srgb(0.5, 0.9, 0.7),
            ),
            screen_text(
                "steer A/D or drag - grab the orbs - dodge the red - Esc to give up",
                20.0,
                Color::srgb(0.7, 0.7, 0.7),
            ),
        ],
    ));
}

/// Start the game on a click / tap from the menu.
fn advance_from_menu(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    sfx: Res<SfxAssets>,
    mut next: ResMut<NextState<GameState>>,
) {
    if advance_pressed(&mouse, &keys, &touches) {
        commands.play_sfx_volume(sfx.menu_select.clone(), 0.7);
        next.set(GameState::Playing);
    }
}

// --- Run lifecycle --------------------------------------------------------

/// Reset per-run state when a new game starts.
fn start_game(
    mut score: ResMut<Score>,
    mut elapsed: ResMut<Elapsed>,
    mut level: ResMut<Level>,
    mut cooldown: ResMut<HitCooldown>,
    mut streak: ResMut<Streak>,
    mut q_shake: Query<&mut CameraShakeInput>,
) {
    score.0 = 0;
    elapsed.0 = 0.0;
    level.0 = 1;
    cooldown.0 = 0.0;
    streak.reset();
    // Snap any lingering shake back to zero so it does not bleed into the run.
    if let Ok(mut input) = q_shake.single_mut() {
        input.reset = true;
    }
    // The damage overlay is respawned transparent by spawn_hud each run, so
    // there is no persistent flash intensity to reset here.
}

/// Spawn the player marker: a `DirectionalSphereOrbit` rider plus the `Runner`
/// frame the game steers, and the `Health` the run rides on. Scoped to
/// `GameOver` so it stays visible in the frozen scene behind the overlay.
fn spawn_runner(mut commands: Commands, assets: Res<OrbitAssets>) {
    // Start on the "front" of the planet (+Z) heading up toward the north pole.
    let up = Vec3::Z;
    let forward = Vec3::Y;

    commands.spawn((
        Name::new("Runner"),
        Runner { up, forward },
        Health::new(PLAYER_HEALTH),
        DirectionalSphereOrbit {
            radius: PLANET_RADIUS + RUNNER_LIFT,
            center: Vec3::ZERO,
            direction: up,
            smoothing: 0.0,
        },
        DespawnOnExit(GameState::GameOver),
        Mesh3d(assets.runner_mesh.clone()),
        MeshMaterial3d(assets.runner_material.clone()),
        Transform::from_translation(up * (PLANET_RADIUS + RUNNER_LIFT))
            .with_scale(Vec3::splat(RUNNER_RADIUS)),
    ));
}

/// Read steering and advance the marker's surface frame, then write the new
/// heading into the orbit's input so the plugin resolves the world position.
fn steer_runner(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    window: Single<&Window>,
    mut q_runner: Query<(&mut Runner, &mut DirectionalSphereOrbitInput)>,
) {
    let steer = read_steer(&keys, &mouse, &touches, &window);
    let dt = time.delta_secs();

    for (mut runner, mut input) in q_runner.iter_mut() {
        let (up, forward) = step_runner_frame(
            runner.up,
            runner.forward,
            steer,
            TURN_RATE,
            RUNNER_SPEED,
            PLANET_RADIUS + RUNNER_LIFT,
            dt,
        );
        runner.up = up;
        runner.forward = forward;
        // The orbit maps this direction to the surface point in that direction.
        input.0 = up;
    }
}

/// Place the marker at the orbit's resolved surface position and orient it along
/// its heading. Runs after the orbit plugin has updated the output.
fn apply_runner_transform(
    mut q_runner: Query<(&Runner, &DirectionalSphereOrbitOutput, &mut Transform)>,
) {
    for (runner, output, mut transform) in q_runner.iter_mut() {
        transform.translation = **output;
        transform.rotation = frame_rotation(runner.up, runner.forward);
    }
}

/// Copy each wandering object's resolved orbit position onto its transform.
fn apply_object_transforms(
    mut q_objects: Query<(&RandomSphereOrbitOutput, &mut Transform), Without<Runner>>,
) {
    for (output, mut transform) in q_objects.iter_mut() {
        transform.translation = **output;
    }
}

/// Point the chase camera at the marker's surface frame so it orbits the planet
/// behind and above the marker, looking the way it travels.
fn drive_chase_camera(
    q_runner: Query<(&Runner, &DirectionalSphereOrbitOutput)>,
    mut q_camera: Query<&mut ChaseCameraInput, With<MainCamera>>,
) {
    let Ok((runner, output)) = q_runner.single() else {
        return;
    };
    let Ok(mut input) = q_camera.single_mut() else {
        return;
    };
    input.anchor_pos = **output;
    input.anchor_rot = frame_rotation(runner.up, runner.forward);
}

// --- Wandering objects ----------------------------------------------------

/// Spawn one wandering thing (hazard or orb) at a random surface angle clear of
/// the marker. Both use `RandomSphereOrbit`, which drives their motion entirely
/// on its own. `runner_dir` is the marker's current surface direction; the
/// random angle is resampled a few times so the spawn does not land on the
/// player (see `spawn_is_clear`).
fn spawn_wanderer(
    commands: &mut Commands,
    runner_dir: Vec3,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    collision_radius: f32,
    lift: f32,
    angular_speed: f32,
    is_hazard: bool,
) {
    let mut rng = rand::rng();
    let random_angles = |rng: &mut rand::rngs::ThreadRng| {
        (
            rng.random_range(0.0..std::f32::consts::TAU),
            rng.random_range(-std::f32::consts::FRAC_PI_2..std::f32::consts::FRAC_PI_2),
        )
    };

    // Resample until the candidate direction is clear of the marker, capped so a
    // pathological run can never loop forever; the last sample is used if none
    // of the tries were clear (extremely unlikely with this small exclusion).
    let (mut theta, mut phi) = random_angles(&mut rng);
    for _ in 0..8 {
        // A unit direction for the candidate angle -- exercises `meth` directly.
        if spawn_is_clear(spherical_to_cartesian(1.0, theta, phi), runner_dir) {
            break;
        }
        (theta, phi) = random_angles(&mut rng);
    }
    let radius = PLANET_RADIUS + lift;

    let mut entity = commands.spawn((
        Name::new(if is_hazard { "Hazard" } else { "Orb" }),
        RandomSphereOrbit {
            radius,
            angular_speed,
            center: Vec3::ZERO,
            initial_theta: theta,
            initial_phi: phi,
        },
        DespawnOnExit(GameState::GameOver),
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_scale(Vec3::splat(collision_radius)),
    ));
    if is_hazard {
        entity.insert(Hazard);
    } else {
        entity.insert(Orb);
    }
}

/// Keep the planet stocked: top hazards up to the level's target and orbs up to
/// `ORB_COUNT`. Called every frame while playing, so a collected orb is replaced
/// and a new level's extra hazards appear.
fn maintain_objects(
    mut commands: Commands,
    assets: Res<OrbitAssets>,
    level: Res<Level>,
    q_runner: Query<&Runner>,
    q_hazards: Query<(), With<Hazard>>,
    q_orbs: Query<(), With<Orb>>,
) {
    let speed = wander_speed_for(level.0);
    // The marker's current surface direction, so fresh spawns keep clear of it.
    let runner_dir = q_runner.single().map(|runner| runner.up).unwrap_or(Vec3::Z);

    let hazard_target = hazard_target_for(level.0);
    for _ in q_hazards.iter().count()..hazard_target {
        spawn_wanderer(
            &mut commands,
            runner_dir,
            assets.hazard_mesh.clone(),
            assets.hazard_material.clone(),
            HAZARD_RADIUS,
            OBJECT_LIFT,
            speed,
            true,
        );
    }

    for _ in q_orbs.iter().count()..ORB_COUNT {
        spawn_wanderer(
            &mut commands,
            runner_dir,
            assets.orb_mesh.clone(),
            assets.orb_material.clone(),
            ORB_RADIUS,
            OBJECT_LIFT,
            speed * 0.8,
            false,
        );
    }
}

/// Collide the marker with the wandering objects: collect overlapping orbs
/// (score + respawn) and take damage from overlapping hazards (unless still in
/// the post-hit invulnerability window).
fn resolve_collisions(
    mut commands: Commands,
    sfx: Res<SfxAssets>,
    mut score: ResMut<Score>,
    mut cooldown: ResMut<HitCooldown>,
    mut streak: ResMut<Streak>,
    mut q_shake: Query<&mut CameraShakeInput>,
    q_flash_overlay: Query<Entity, With<DamageFlashOverlay>>,
    q_runner: Query<(Entity, &Transform), With<Runner>>,
    q_hazards: Query<(Entity, &Transform), With<Hazard>>,
    q_orbs: Query<(Entity, &Transform), With<Orb>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    q_banners: Query<Entity, With<StreakBanner>>,
) {
    let Ok((runner_entity, runner_transform)) = q_runner.single() else {
        return;
    };
    let runner_pos = runner_transform.translation;

    // Orbs: collect any overlapping one. `maintain_objects` replaces it next
    // frame, so the field stays full.
    for (orb, transform) in q_orbs.iter() {
        if spheres_overlap(
            runner_pos,
            transform.translation,
            RUNNER_RADIUS + ORB_RADIUS,
        ) {
            commands.entity(orb).despawn();

            // Extend the streak and score the orb scaled by how long the chain
            // is, so quick collecting pays off.
            let count = streak.hit();
            let points = orb_points_for(count);
            score.0 += points;

            // Pickup blip climbs in pitch as the streak grows; a chain of x2+
            // also layers the rising combo chime.
            commands.trigger(
                PlaySfx::new(sfx.pickup.clone())
                    .with_volume(0.8)
                    .with_speed(pickup_pitch_for(count)),
            );
            if count >= 2 {
                commands.trigger(
                    PlaySfx::new(sfx.combo.clone())
                        .with_volume(0.5)
                        .with_speed(pickup_pitch_for(count)),
                );
            }

            // Float a "+N" popup at the orb's screen position (best-effort:
            // skip it if the orb is off-screen / behind the camera), and a
            // "STREAK xN" banner near the top once the chain reaches x2.
            if let Ok((camera, camera_transform)) = q_camera.single() {
                if let Some(viewport_pos) =
                    world_to_screen(camera, camera_transform, transform.translation)
                {
                    commands
                        .spawn(popup(
                            viewport_pos,
                            format!("+{points}"),
                            34.0,
                            Color::srgb(0.6, 0.95, 0.75),
                        ))
                        .insert(DespawnOnExit(GameState::Playing));
                }
            }
            if count >= 2 {
                // Replace any live banner so a fast chain updates the count in
                // place instead of stacking overlapping popups.
                for banner in q_banners.iter() {
                    commands.entity(banner).despawn();
                }
                spawn_streak_banner(&mut commands, count);
            }
        }
    }

    // Hazards: one hit per invulnerability window, so a single overlap does not
    // drain all health at once.
    if cooldown.0 > 0.0 {
        return;
    }
    for (_hazard, transform) in q_hazards.iter() {
        if spheres_overlap(
            runner_pos,
            transform.translation,
            RUNNER_RADIUS + HAZARD_RADIUS,
        ) {
            cooldown.0 = HIT_COOLDOWN;
            // A hit also breaks the streak, jolts the camera and flashes red.
            streak.reset();
            if let Ok(mut input) = q_shake.single_mut() {
                input.add_trauma += HAZARD_TRAUMA;
            }
            // Re-spike the persistent full-screen overlay to its peak; it decays
            // back to transparent via ScreenFlashPlugin.
            if let Ok(overlay) = q_flash_overlay.single() {
                commands.entity(overlay).insert(ScreenFlash {
                    peak_alpha: DAMAGE_FLASH_PEAK_ALPHA,
                    decay: DAMAGE_FLASH_DECAY,
                    despawn_on_end: false,
                });
            }
            commands.play_sfx(sfx.hurt.clone());
            commands.trigger(HealthApplyDamage {
                entity: runner_entity,
                source: None,
                amount: HAZARD_DAMAGE,
            });
            break;
        }
    }
}

// --- Timers, level, HUD ---------------------------------------------------

/// Advance the run clock each frame while playing.
fn tick_elapsed(time: Res<Time>, mut elapsed: ResMut<Elapsed>) {
    elapsed.0 += time.delta_secs();
}

/// Bump the difficulty level as the run clock crosses each `LEVEL_SECS` mark and
/// ping the level-up sound.
fn advance_level(
    mut commands: Commands,
    elapsed: Res<Elapsed>,
    sfx: Res<SfxAssets>,
    mut level: ResMut<Level>,
) {
    let target = level_for(elapsed.0);
    if target > level.0 {
        level.0 = target;
        commands.play_sfx_volume(sfx.level_up.clone(), 0.7);
    }
}

/// Count down the post-hit invulnerability window.
fn tick_hit_cooldown(time: Res<Time>, mut cooldown: ResMut<HitCooldown>) {
    if cooldown.0 > 0.0 {
        cooldown.0 = (cooldown.0 - time.delta_secs()).max(0.0);
    }
}

/// Blink the marker while it is invulnerable so the hit reads clearly.
fn blink_runner(cooldown: Res<HitCooldown>, mut q_runner: Query<&mut Visibility, With<Runner>>) {
    let Ok(mut visibility) = q_runner.single_mut() else {
        return;
    };
    // Toggle a few times a second while the window lasts; solid otherwise.
    let hidden = cooldown.0 > 0.0 && (cooldown.0 * 12.0) as i32 % 2 == 0;
    *visibility = if hidden {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };
}

/// Spawn the in-game HUD (score, level, health bar), scoped to `Playing`.
fn spawn_hud(mut commands: Commands) {
    // Persistent full-screen overlay, transparent until a hit spikes it (a hazard
    // inserts a `ScreenFlash` to re-spike; ScreenFlashPlugin decays it back).
    // Spawned first and pinned below the HUD so it never hides the score / health.
    commands.spawn((
        Name::new("Damage Flash"),
        DamageFlashOverlay,
        DespawnOnExit(GameState::Playing),
        GlobalZIndex(-1),
        screen_flash_node(),
        BackgroundColor(Color::srgba(0.85, 0.05, 0.05, 0.0)),
    ));

    commands.spawn((
        Name::new("Score HUD"),
        ScoreText,
        DespawnOnExit(GameState::Playing),
        Text::new(score_label(0)),
        TextFont {
            font_size: FontSize::Px(38.0),
            ..default()
        },
        TextColor(Color::srgb(0.6, 0.9, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(16.0),
            ..default()
        },
    ));

    commands.spawn((
        Name::new("Level HUD"),
        LevelText,
        DespawnOnExit(GameState::Playing),
        Text::new(level_label(1)),
        TextFont {
            font_size: FontSize::Px(28.0),
            ..default()
        },
        TextColor(Color::srgb(0.5, 0.9, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            right: Val::Px(16.0),
            ..default()
        },
    ));

    // Health bar: a dark track with a green fill whose width tracks health.
    commands
        .spawn((
            Name::new("Health Track"),
            DespawnOnExit(GameState::Playing),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(64.0),
                left: Val::Px(16.0),
                width: Val::Px(220.0),
                height: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.12, 0.8)),
        ))
        .with_children(|track| {
            track.spawn((
                Name::new("Health Fill"),
                HealthBarFill,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.85, 0.4)),
            ));
        });
}

/// Text shown by the score HUD for a given displayed score.
fn score_label(score: usize) -> String {
    format!("Score: {score}")
}

/// Text shown by the level HUD for a given level.
fn level_label(level: usize) -> String {
    format!("Level {level}")
}

/// Refresh the HUD each frame: score (orbs + survival), level, and the health
/// bar fill width / color.
fn update_hud(
    score: Res<Score>,
    elapsed: Res<Elapsed>,
    level: Res<Level>,
    q_runner: Query<&Health, With<Runner>>,
    mut q_score: Query<&mut Text, (With<ScoreText>, Without<LevelText>)>,
    mut q_level: Query<&mut Text, (With<LevelText>, Without<ScoreText>)>,
    mut q_fill: Query<(&mut Node, &mut BackgroundColor), With<HealthBarFill>>,
) {
    for mut text in q_score.iter_mut() {
        **text = score_label(score_value(score.0, elapsed.0));
    }
    for mut text in q_level.iter_mut() {
        **text = level_label(level.0);
    }

    if let Ok(health) = q_runner.single() {
        let fraction = (health.current / health.max).clamp(0.0, 1.0);
        for (mut node, mut color) in q_fill.iter_mut() {
            node.width = Val::Percent(fraction * 100.0);
            // Green when healthy, sliding to red as it drains.
            color.0 = Color::srgb(
                0.3 + (1.0 - fraction) * 0.6,
                0.85 * fraction,
                0.4 * fraction,
            );
        }
    }
}

/// Give up the current run with Escape.
fn giveup_on_escape(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Escape) {
        next.set(GameState::GameOver);
    }
}

// --- Streak & floating popups ---------------------------------------------

/// Count the streak window down; when it lapses the streak resets to zero so the
/// next pickup starts a fresh chain. 07 has no end-of-streak tally, so the
/// `tick` return is ignored.
fn tick_streak(time: Res<Time>, mut streak: ResMut<Streak>) {
    streak.tick(time.delta_secs());
}

/// Flash a "STREAK xN" banner high on the screen. It rises and fades via the
/// crate `Popup` like the "+N" popups but starts higher, wider and slower, so a
/// hot streak is unmissable. Built as a custom layout (centered, full width) on
/// top of a `Popup` rather than the `popup()` helper.
fn spawn_streak_banner(commands: &mut Commands, count: usize) {
    let color = Color::srgb(1.0, 0.85, 0.35);
    commands.spawn((
        Name::new("Streak Banner"),
        StreakBanner,
        Popup {
            rise_speed: STREAK_RISE_SPEED,
            base_color: color,
            ..default()
        },
        DespawnOnExit(GameState::Playing),
        Text::new(format!("STREAK x{count}")),
        TextFont {
            font_size: FontSize::Px(44.0),
            ..default()
        },
        TextColor(color),
        TextLayout {
            justify: Justify::Center,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(130.0),
            width: Val::Percent(100.0),
            ..default()
        },
    ));
}

// --- Game over ------------------------------------------------------------

/// End the run when the marker's health hits zero (a hazard finished it off).
fn on_runner_died(
    add: On<Add, HealthZeroMarker>,
    q_runner: Query<(), With<Runner>>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
) {
    // Only react to the player's death, and only while actually playing.
    if q_runner.contains(add.entity) && *state.get() == GameState::Playing {
        next.set(GameState::GameOver);
    }
}

/// Record the run's score into the session high score, flagging a new best.
fn record_high_score(
    score: Res<Score>,
    elapsed: Res<Elapsed>,
    mut high: ResMut<HighScore>,
    mut new_best: ResMut<NewBest>,
) {
    let final_score = score_value(score.0, elapsed.0);
    new_best.0 = final_score > high.0;
    high.0 = high.0.max(final_score);
}

/// Spawn the game-over screen with the final score, scoped to `GameOver`.
fn spawn_game_over(
    mut commands: Commands,
    score: Res<Score>,
    elapsed: Res<Elapsed>,
    high: Res<HighScore>,
    new_best: Res<NewBest>,
) {
    let final_score = score_value(score.0, elapsed.0);
    commands
        .spawn((
            Name::new("Game Over"),
            DespawnOnExit(GameState::GameOver),
            centered_screen(),
        ))
        .with_children(|parent| {
            parent.spawn(screen_text("GAME OVER", 72.0, Color::srgb(0.9, 0.3, 0.3)));
            parent.spawn(screen_text(
                score_label(final_score),
                40.0,
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

/// Play the game-over sting when the screen appears.
fn play_game_over_sfx(mut commands: Commands, sfx: Res<SfxAssets>) {
    commands.play_sfx_volume(sfx.game_over.clone(), 0.9);
}

/// Return to the menu on a tap / click from the game-over screen.
fn advance_from_game_over(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut next: ResMut<NextState<GameState>>,
) {
    if advance_pressed(&mouse, &keys, &touches) {
        next.set(GameState::Menu);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;
    const RADIUS: f32 = PLANET_RADIUS + RUNNER_LIFT;

    /// The surface frame stays orthonormal (unit vectors, perpendicular) after a
    /// step, so the marker never drifts off the surface or lets its heading
    /// collapse.
    #[test]
    fn frame_stays_orthonormal_after_step() {
        let (up, forward) =
            step_runner_frame(Vec3::Z, Vec3::Y, 0.5, TURN_RATE, RUNNER_SPEED, RADIUS, DT);
        assert!((up.length() - 1.0).abs() < 1e-5);
        assert!((forward.length() - 1.0).abs() < 1e-5);
        assert!(up.dot(forward).abs() < 1e-5);
    }

    /// Travelling forward moves the surface position (up) toward the heading.
    #[test]
    fn advancing_moves_position_toward_heading() {
        let up = Vec3::Z;
        let forward = Vec3::Y;
        let (new_up, _) = step_runner_frame(up, forward, 0.0, TURN_RATE, RUNNER_SPEED, RADIUS, DT);
        // The new position should have tilted toward +Y (the heading).
        assert!(new_up.dot(forward) > up.dot(forward));
        assert!(new_up.dot(Vec3::Z) < 1.0);
    }

    /// With no steering the heading stays on the same great circle: no drift
    /// sideways off the plane spanned by up and forward.
    #[test]
    fn no_steer_keeps_a_straight_course() {
        let up = Vec3::Z;
        let forward = Vec3::Y;
        // The great circle through Z heading Y lives in the Y-Z plane (x == 0).
        let (new_up, new_forward) =
            step_runner_frame(up, forward, 0.0, TURN_RATE, RUNNER_SPEED, RADIUS, DT);
        assert!(new_up.x.abs() < 1e-5);
        assert!(new_forward.x.abs() < 1e-5);
    }

    /// Opposite steering inputs turn the heading opposite ways.
    #[test]
    fn steering_turns_the_heading() {
        let up = Vec3::Z;
        let forward = Vec3::Y;
        let (_, left) = step_runner_frame(up, forward, -1.0, TURN_RATE, RUNNER_SPEED, RADIUS, DT);
        let (_, right) = step_runner_frame(up, forward, 1.0, TURN_RATE, RUNNER_SPEED, RADIUS, DT);
        // The two headings differ, and each has swung off the pure-forward axis.
        assert!(left.distance(right) > 1e-3);
        assert!(left.x * right.x <= 0.0);
    }

    /// The frame rotation aims local -Z along the heading and local +Y along up.
    #[test]
    fn frame_rotation_aligns_axes() {
        let up = Vec3::Z;
        let forward = Vec3::Y;
        let rot = frame_rotation(up, forward);
        assert!((rot * Vec3::NEG_Z).abs_diff_eq(forward, 1e-5));
        assert!((rot * Vec3::Y).abs_diff_eq(up, 1e-5));
    }

    #[test]
    fn overlap_when_close_and_not_when_far() {
        assert!(spheres_overlap(Vec3::ZERO, Vec3::new(0.9, 0.0, 0.0), 1.0));
        assert!(!spheres_overlap(Vec3::ZERO, Vec3::new(1.1, 0.0, 0.0), 1.0));
    }

    #[test]
    fn spawn_is_clear_rejects_near_the_marker() {
        // Build candidates by rotating the marker direction by a known angle, so
        // the test is independent of the spherical-angle convention.
        let runner = Vec3::Z;
        // Same direction as the marker: not clear.
        assert!(!spawn_is_clear(runner, runner));
        // A hair off (well inside the exclusion angle): still not clear.
        let near = Quat::from_axis_angle(Vec3::X, MIN_SPAWN_SEPARATION * 0.5) * runner;
        assert!(!spawn_is_clear(near, runner));
        // Comfortably past the exclusion angle: clear.
        let far = Quat::from_axis_angle(Vec3::X, MIN_SPAWN_SEPARATION * 2.0) * runner;
        assert!(spawn_is_clear(far, runner));
        // The opposite pole is always clear.
        assert!(spawn_is_clear(-runner, runner));
    }

    #[test]
    fn level_climbs_with_elapsed_time() {
        assert_eq!(level_for(0.0), 1);
        assert_eq!(level_for(LEVEL_SECS - 0.1), 1);
        assert_eq!(level_for(LEVEL_SECS), 2);
        assert_eq!(level_for(LEVEL_SECS * 3.0), 4);
    }

    #[test]
    fn hazard_target_grows_and_caps() {
        assert_eq!(hazard_target_for(1), HAZARD_START);
        assert_eq!(hazard_target_for(2), HAZARD_START + 1);
        // Far past the cap it saturates at HAZARD_MAX.
        assert_eq!(hazard_target_for(100), HAZARD_MAX);
    }

    #[test]
    fn wander_speed_ramps_between_bounds() {
        assert!((wander_speed_for(1) - WANDER_SPEED_START).abs() < 1e-6);
        let mid = wander_speed_for(5);
        assert!(mid > WANDER_SPEED_START && mid < WANDER_SPEED_MAX);
        // Well past the ramp it is clamped to the cap.
        assert!((wander_speed_for(100) - WANDER_SPEED_MAX).abs() < 1e-6);
    }

    #[test]
    fn score_folds_orb_points_and_survival() {
        // 30 banked orb points plus 5 whole seconds survived.
        assert_eq!(score_value(30, 5.7), 30 + 5);
        assert_eq!(score_value(0, 0.0), 0);
    }

    #[test]
    fn streak_advances_and_scores_more_each_orb() {
        // The streak bookkeeping (count + window) is the crate's `Streak`; here we
        // check this game's value rule scales with the count it returns.
        let mut streak = Streak::new(STREAK_WINDOW);
        assert_eq!(streak.hit(), 1);
        assert_eq!(orb_points_for(1), ORB_POINTS);
        assert_eq!(streak.hit(), 2);
        assert_eq!(streak.hit(), 3);
        assert_eq!(orb_points_for(2), 2 * ORB_POINTS);
        assert!(orb_points_for(3) > orb_points_for(2));
        // A zero streak still values an orb at the base (never zero points).
        assert_eq!(orb_points_for(0), ORB_POINTS);
    }

    #[test]
    fn pickup_pitch_rises_with_streak_and_caps() {
        // A lone pickup plays at normal speed.
        assert!((pickup_pitch_for(1) - 1.0).abs() < 1e-6);
        // It climbs with the streak...
        assert!(pickup_pitch_for(3) > pickup_pitch_for(1));
        // ... but never past the cap.
        assert!((pickup_pitch_for(1000) - PICKUP_PITCH_MAX).abs() < 1e-6);
    }
}
