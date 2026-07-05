//! 12_bastion -- a defend-the-core tower defense, and the headline demo of
//! [`camera/project`](bevy_common_systems::camera::project) together with the
//! two "aim/track" halves of the transform family that no other example shows.
//!
//! A glowing **Core** (a [`Health`] pool) sits at the center of a circular arena.
//! Enemies spawn all around the border and crawl inward from every bearing; one
//! that reaches the Core damages it, and when the Core's health hits zero the run
//! ends. You earn credits by killing enemies and spend them to place towers on
//! the ground around the Core and to upgrade them. Enemies arrive in *packs* --
//! bursts that spawn together from spread-out bearings -- and each wave ramps the
//! pack size, the number of packs, and the enemies' speed and toughness, so the
//! total onslaught grows faster than linearly.
//!
//! It exists to exercise three crate modules interactively for the first time:
//!
//! - `camera/project` (the headline): `pointer_on_plane` maps a mouse/touch pixel
//!   to the world point on the ground plane where a tower is placed (a ghost +
//!   range ring is previewed there), and `world_to_screen` anchors the floating
//!   "+N" credit popups over a killed enemy and the upgrade label over a selected
//!   tower.
//! - `transform/point_rotation` drives an **orbit camera**: an invisible pivot at
//!   the Core carries `PointRotation`, and dragging the pointer (or A/D) feeds it
//!   a yaw-only delta so the whole view spins around the battlefield. Pitch is
//!   never touched, so it is a flat yaw orbit. The camera is a child at a fixed
//!   angled offset, so the pleasant top-down framing holds no matter how far you
//!   orbit.
//! - `transform/smooth_look_rotation` drives every **tower turret**: a tower
//!   auto-targets the nearest enemy in range and its turret rotates toward it at a
//!   rate-limited `speed`, so a fast enemy can briefly out-slew a cheap turret
//!   until it is upgraded. The turn rate is a real per-tower stat.
//!
//! It also reuses the established shape and juice kit: `States` for
//! menu/playing/game-over, one-shot sounds via [`SfxPlugin`], a `ui/status` HUD,
//! `mesh/explode` shards on a kill, `ui/popup` "+N", `camera/shake`, `feedback`
//! flashes, `time/cooldown` per-tower fire cadence, `scoring/streak`,
//! `helpers/temp` self-despawning effects, and the same wasm/trunk web build. The
//! tower and enemy stats are data-driven: they load from
//! `assets/bastion/catalog.json` at startup (native reads the file, so editing
//! the stats or adding a new tower/enemy needs no recompile; wasm uses a
//! compiled-in copy), and both the build key bindings and the enemy spawn mix
//! iterate the catalog so new JSON entries participate automatically. See
//! `docs/2026-07-05-bastion-data-catalog.md`.
//!
//! Controls: drag the pointer (or A/D / left-right arrow keys) to spin the yaw
//! orbit around the Core (the tilt never changes). Press a
//! number key (1..N, one per catalogued tower) to pick a tower to build, then tap
//! the ground (or press Space) to place it at the ghost. Tap a placed tower to
//! select it and press U to upgrade it. Escape gives up. On a touchscreen the
//! same taps and drags work.
//!
//! Run it: `cargo run --example 12_bastion` (add `--features debug` for the
//! inspector and the headless autopilot/screenshot harness).

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
use clap::Parser;
use rand::Rng;
use serde::Deserialize;

#[derive(Parser)]
#[command(name = "12_bastion")]
#[command(version = "1.0.0")]
#[command(
    about = "Defend the Core: place towers around it and hold off waves that close in from every side. Drag to orbit, number keys to pick a tower, tap or Space to build, tap a tower and U to upgrade.",
    long_about = None
)]
struct Cli;

// ---------------------------------------------------------------------------
// Tuning
// ---------------------------------------------------------------------------

/// Radius of the circular arena. Enemies spawn on this ring; the Core sits at the
/// origin.
const ARENA_RADIUS: f32 = 16.0;

/// Radius of the Core. Reaching within `CORE_RADIUS + enemy radius` counts as a
/// hit on the Core.
const CORE_RADIUS: f32 = 1.6;

/// The Core's hit points; also the max the integrity gauge shows.
const CORE_HEALTH: f32 = 100.0;

/// Camera framing. The camera child sits back and up from the pivot and looks at
/// the Core, giving an angled top-down view; the pivot's rotation (driven by
/// `point_rotation`) orbits this whole rig.
const CAM_BACK: f32 = 26.0;
const CAM_UP: f32 = 20.0;

/// Orbit control. The camera is a pure yaw orbit: A/D and pointer drag spin the
/// view around the vertical axis (full 360, unclamped) and the pitch never
/// changes, so the pleasant angled top-down framing (set by the fixed camera
/// child offset) is preserved no matter how far you orbit.
const ORBIT_YAW_RATE: f32 = 1.6; // radians/sec from A/D
const ORBIT_DRAG_RATE: f32 = 0.005; // radians per pixel dragged

/// A press that moves less than this many pixels before release is a tap (place /
/// select); more than this is a drag (orbit). Keeps one pointer doing both.
const TAP_MOVE_THRESHOLD: f32 = 8.0;

/// Credits the player starts a run with, and the reward/cost economy.
const START_CREDITS: u32 = 120;

/// Minimum spacing between a new tower and the Core / other towers, so towers do
/// not stack or block the Core.
const TOWER_MIN_SPACING: f32 = 2.2;
const TOWER_MIN_CORE_DIST: f32 = CORE_RADIUS + 1.4;

/// Wave pacing. A wave is released in *packs*: a burst of several enemies that
/// spawn together at spread-out bearings, with a short breather between packs and
/// a longer one between waves. Both the pack size and the number of packs grow
/// with the wave number, so the total enemy count ramps faster than linearly (see
/// `pack_size`, `packs_in_wave`, `wave_size`), and their hp/speed scale on top of
/// that -- the difficulty is felt on several axes at once.
const PACK_SIZE_BASE: usize = 3; // enemies per pack at wave 1
const PACK_SIZE_PER_WAVE: f32 = 0.5; // +0.5 enemy/pack per wave (floored)
const PACKS_BASE: usize = 2; // packs in wave 1
const PACKS_PER_WAVE: usize = 1; // +1 pack per wave
const WAVE_HP_PER: f32 = 0.22; // +22% hp per wave
const WAVE_SPEED_PER: f32 = 0.06; // +6% speed per wave
const PACK_GAP: f32 = 2.2; // seconds between packs within a wave
const WAVE_GAP: f32 = 3.5; // seconds of calm between waves

/// Streak window: kills within this many seconds of each other chain a combo that
/// multiplies the credit reward.
const STREAK_WINDOW: f32 = 2.5;

/// Seconds the death sequence holds (Core gone, red flash) before the game-over
/// screen appears.
const DYING_BEAT: f32 = 0.9;

/// Camera-shake trauma added by events (clamped to 1.0 internally).
const SHAKE_KILL: f32 = 0.18;
const SHAKE_CORE_HIT: f32 = 0.5;
const SHAKE_DEATH: f32 = 1.0;

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Top-level game flow: the menu, the playable run, and the game-over screen.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

fn main() {
    let _ = Cli::parse();
    let mut app = App::new();

    // On the web the game runs inside a canvas: fit it to its parent element so it
    // fills the (portrait, mobile-ish) frame the showcase site embeds it in. These
    // fields are ignored on native.
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

    // No physics simulation here, but the debug inspector's gizmos need avian's
    // resources, so add the plugins (as 07/11 do) and zero gravity.
    app.add_plugins(PhysicsPlugins::default());
    app.insert_resource(Gravity(Vec3::ZERO));

    // Dusk backdrop.
    app.insert_resource(ClearColor(Color::srgb(0.02, 0.03, 0.06)));

    #[cfg(feature = "debug")]
    app.add_plugins(InspectorDebugPlugin);

    // Headless verification harness (dev tooling, `debug` feature). Inert unless
    // BCS_AUTOPILOT / BCS_SHOT is set; see `docs/dev-harness.md`.
    #[cfg(feature = "debug")]
    {
        app.add_plugins(
            AutopilotPlugin::new()
                .hold(GameState::Menu, 0.6)
                .hold(GameState::Playing, 5.0)
                .hold(GameState::GameOver, 0.8)
                .input(|world, elapsed| {
                    // Only drive input while playing; the menu / game-over screens
                    // advance on "any press", so poking keys there would skip them.
                    if *world.resource::<State<GameState>>().get() != GameState::Playing {
                        return;
                    }
                    // Every ~0.8s: select a tower type, orbit a little, and place a
                    // tower at the ring-front via Space (the keyboard placement
                    // path). `just_pressed` is edge-triggered, so reset each frame.
                    let beat = (elapsed / 0.8) as u32;
                    let even = beat.is_multiple_of(2);
                    let mut keys = world.resource_mut::<ButtonInput<KeyCode>>();
                    keys.reset_all();
                    keys.press(if even {
                        KeyCode::Digit1
                    } else {
                        KeyCode::Digit2
                    });
                    keys.press(KeyCode::KeyD);
                    // Place on every other beat so orbit has moved the ring-front.
                    if !even {
                        keys.press(KeyCode::Space);
                    }
                }),
        );
        app.add_plugins(ScreenshotPlugin::new(GameState::Playing).settle_frames(45));
    }

    if !app.is_plugin_added::<bevy::diagnostic::FrameTimeDiagnosticsPlugin>() {
        app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
    }

    // Crate plugins.
    app.add_plugins(PointRotationPlugin);
    app.add_plugins(SmoothLookRotationPlugin);
    app.add_plugins(ExplodeMeshPlugin);
    app.add_plugins(PostProcessingDefaultPlugin);
    app.add_plugins(CameraShakePlugin);
    app.add_plugins(FlashPlugin);
    app.add_plugins(ScreenFlashPlugin);
    app.add_plugins(PopupPlugin);
    app.add_plugins(MenuPlugin);
    app.add_plugins(TempEntityPlugin);
    app.add_plugins(StatusBarPlugin);
    app.add_plugins(HealthPlugin);
    app.add_plugins(SfxPlugin);
    app.add_plugins(UnifiedPointerPlugin);

    app.init_state::<GameState>();

    // Tower/enemy stats are data-driven: load them from JSON before any system
    // runs (native reads the editable file, wasm the embedded copy).
    app.insert_resource(load_catalog());

    app.init_resource::<Credits>();
    app.init_resource::<Score>();
    app.init_resource::<HighScore<u32>>();
    app.init_resource::<WaveState>();
    app.init_resource::<Build>();
    app.init_resource::<Selection>();
    app.init_resource::<Combo>();
    app.init_resource::<DyingTimer>();
    app.init_resource::<DragState>();

    // Persistent scene: camera rig, lights, ground, Core, FPS overlay, sfx.
    app.add_systems(Startup, setup);

    // The orbit control runs in every state: it overwrites the rig's input from
    // this frame's deltas, so with no drag/keys the input is zero and the view
    // holds. (If it stopped running in a state, the last nonzero delta would be
    // left in place and `PointRotationPlugin` would keep spinning the camera.)
    app.add_systems(Update, orbit_camera);

    // Main menu.
    app.add_systems(OnEnter(GameState::Menu), spawn_menu);
    app.add_systems(Update, menu_click.run_if(in_state(GameState::Menu)));

    // Playing.
    app.add_systems(OnEnter(GameState::Playing), (start_game, spawn_hud).chain());
    app.add_systems(
        Update,
        (
            select_build,
            update_ghost,
            place_or_select,
            upgrade_selected,
            advance_waves,
            move_enemies,
            aim_and_fire_towers,
            tick_tracers,
            tick_combo,
            draw_arena,
            update_hud,
            advance_dying,
            set_state_on_key(KeyCode::Escape, GameState::GameOver),
        )
            // `place_or_select` / `update_ghost` read the DragState that
            // `orbit_camera` computes this frame, so pin them after it.
            .after(orbit_camera)
            .run_if(in_state(GameState::Playing)),
    );

    // Game over screen.
    app.add_systems(
        OnEnter(GameState::GameOver),
        (record_high_score, spawn_game_over, play_game_over_sfx).chain(),
    );
    app.add_systems(Update, gameover_click.run_if(in_state(GameState::GameOver)));

    app.add_observer(on_enemy_killed);
    app.add_observer(on_fragments_spawned);
    app.add_observer(on_core_died);

    app.run();
}

// ---------------------------------------------------------------------------
// Spec catalog (data-driven: loaded from assets/bastion/catalog.json at startup)
// ---------------------------------------------------------------------------
//
// The tower and enemy stat tables live in an external JSON file, deserialized
// once into the `Catalog` resource. Towers and enemies are referenced by their
// index in the catalog `Vec`s, and both the build key bindings and the enemy
// spawn mix iterate the catalog, so a new tower or enemy can be added purely in
// JSON (native reads the file at startup, no recompile) and it just shows up.

/// A buildable tower archetype (pure data, deserialized from JSON).
#[derive(Clone, Deserialize)]
struct TowerSpec {
    name: String,
    /// Credit cost to place one.
    cost: u32,
    /// Credit cost to upgrade one (flat, per level).
    upgrade_cost: u32,
    /// Firing range in world units.
    range: f32,
    /// Seconds between shots.
    fire_interval: f32,
    /// Damage per shot.
    damage: f32,
    /// Turret slew rate, radians/sec (the `smooth_look_rotation` speed).
    turn_speed: f32,
    /// Body tint as linear `[r, g, b]` in 0..1.
    color: [f32; 3],
}

/// An enemy archetype (base stats; waves scale hp/speed off these).
#[derive(Clone, Deserialize)]
struct EnemySpec {
    name: String,
    hp: f32,
    /// Ground speed in world units/sec.
    speed: f32,
    /// Damage dealt to the Core on arrival.
    core_damage: f32,
    /// Credit reward for a kill (before combo multiplier).
    reward: u32,
    /// Collision / visual radius.
    radius: f32,
    /// Relative spawn frequency: an enemy is picked with probability
    /// proportional to `spawn_weight * (1 + wave * wave_weight)`, so setting
    /// `wave_weight > 0` makes it appear more often in later waves. A new enemy
    /// added to the catalog participates in the mix via these two fields alone.
    spawn_weight: f32,
    /// How much the wave number ramps this enemy's spawn weight (see above).
    #[serde(default)]
    wave_weight: f32,
    /// Body tint as linear `[r, g, b]` in 0..1.
    color: [f32; 3],
}

impl TowerSpec {
    /// The body tint as a Bevy `Color`.
    fn color(&self) -> Color {
        let [r, g, b] = self.color;
        Color::srgb(r, g, b)
    }
}

impl EnemySpec {
    /// The body tint as a Bevy `Color`.
    fn color(&self) -> Color {
        let [r, g, b] = self.color;
        Color::srgb(r, g, b)
    }

    /// This enemy's spawn weight at the given wave (see `spawn_weight`).
    fn weight_at(&self, wave: usize) -> f32 {
        (self.spawn_weight * (1.0 + wave as f32 * self.wave_weight)).max(0.0)
    }
}

/// The tower and enemy stat tables, deserialized from JSON at startup. Towers
/// and enemies are referenced by index into these `Vec`s.
#[derive(Resource, Clone, Deserialize)]
struct Catalog {
    towers: Vec<TowerSpec>,
    enemies: Vec<EnemySpec>,
}

/// The catalog compiled into the binary, used as the default and the wasm
/// source (where there is no filesystem). Native startup prefers the on-disk
/// `assets/bastion/catalog.json` so the stats can be edited without a rebuild.
const EMBEDDED_CATALOG: &str = include_str!("../assets/bastion/catalog.json");

/// Path of the editable catalog (native only; wasm has no filesystem).
#[cfg(not(target_arch = "wasm32"))]
const CATALOG_PATH: &str = "assets/bastion/catalog.json";

/// Load the catalog: on native, read `assets/bastion/catalog.json` from disk so
/// edits take effect on the next run with no recompile; fall back to the
/// compiled-in copy if the file is missing/unreadable (and always on wasm, which
/// has no filesystem). A parse error in the on-disk file is logged and falls back
/// to the embedded copy rather than panicking.
fn load_catalog() -> Catalog {
    let source = read_catalog_source();
    let catalog = match serde_json::from_str::<Catalog>(&source) {
        Ok(catalog) => catalog,
        Err(err) => {
            error!("12_bastion: catalog parse error ({err}); using embedded default");
            serde_json::from_str(EMBEDDED_CATALOG).expect("embedded catalog is valid JSON")
        }
    };
    let towers: Vec<&str> = catalog.towers.iter().map(|t| t.name.as_str()).collect();
    let enemies: Vec<&str> = catalog.enemies.iter().map(|e| e.name.as_str()).collect();
    info!(
        "12_bastion: catalog loaded -- {} towers {:?}, {} enemies {:?}",
        towers.len(),
        towers,
        enemies.len(),
        enemies,
    );
    catalog
}

/// The raw catalog JSON: the on-disk file on native (falling back to embedded),
/// always embedded on wasm.
fn read_catalog_source() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    match std::fs::read_to_string(CATALOG_PATH) {
        Ok(text) => return text,
        Err(err) => info!("12_bastion: no on-disk catalog ({err}); using embedded default"),
    }
    EMBEDDED_CATALOG.to_string()
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Spendable credits for building and upgrading.
#[derive(Resource, Default, Deref, DerefMut)]
struct Credits(u32);

/// Total credits earned this run (the score).
#[derive(Resource, Default, Deref, DerefMut)]
struct Score(u32);

/// Which tower type is currently armed for building, if any.
#[derive(Resource, Default)]
struct Build {
    /// Index into the catalog's `towers`, or `None` when not in build mode.
    spec: Option<usize>,
}

/// The currently selected placed tower, if any (for upgrading).
#[derive(Resource, Default)]
struct Selection {
    tower: Option<Entity>,
}

/// Kill combo: chaining kills within the streak window multiplies the reward.
#[derive(Resource)]
struct Combo(Streak);

impl Default for Combo {
    fn default() -> Self {
        Self(Streak::new(STREAK_WINDOW))
    }
}

/// Wave scheduling. A wave releases `packs_left` packs of `pack_size` enemies
/// each; a pack spawns all at once when `pack_timer` elapses.
#[derive(Resource)]
struct WaveState {
    /// Current wave number (1-based once playing).
    number: usize,
    /// Packs still to release in the current wave.
    packs_left: usize,
    /// Enemies per pack this wave (fixed when the wave opens).
    pack_size: usize,
    /// Seconds until the next pack spawns.
    pack_timer: f32,
    /// Seconds of calm before the next wave starts (0 while a wave is active).
    gap_timer: f32,
}

impl Default for WaveState {
    fn default() -> Self {
        Self {
            number: 0,
            packs_left: 0,
            pack_size: 0,
            pack_timer: 0.0,
            gap_timer: 0.0,
        }
    }
}

/// Countdown before the game-over screen after the Core is destroyed.
#[derive(Resource, Default)]
struct DyingTimer {
    remaining: Option<f32>,
}

/// Tracks the active pointer press for the tap-vs-drag distinction and orbit drag.
/// `orbit_camera` owns this; `place_or_select` reads `released_tap`.
#[derive(Resource, Default)]
struct DragState {
    /// Screen pos where the current press began (None when not pressed), and
    /// whether it has moved far enough to count as a drag.
    start: Option<Vec2>,
    last: Vec2,
    moved: bool,
    /// Whether the pointer was pressed last frame (to detect release edges).
    was_pressed: bool,
    /// True for exactly the frame a press is released without having dragged: a
    /// tap. `place_or_select` consumes this.
    released_tap: bool,
    /// Screen pos of the tap when `released_tap` is set.
    tap_pos: Option<Vec2>,
}

/// Shared meshes/materials so spawning is cheap.
#[derive(Resource)]
struct GameAssets {
    enemy_mesh: Handle<Mesh>,
    /// One material per enemy spec, indexed by spec position.
    enemy_materials: Vec<Handle<StandardMaterial>>,
    tower_base_mesh: Handle<Mesh>,
    turret_mesh: Handle<Mesh>,
    ghost_material: Handle<StandardMaterial>,
    ghost_bad_material: Handle<StandardMaterial>,
    spark_mesh: Handle<Mesh>,
    spark_material: Handle<StandardMaterial>,
}

/// One gameplay-event sound, keyed into the crate's `SoundBank`. The semantic
/// key decouples from the file (`Build -> pickup.wav`, `Shot -> launch.wav`,
/// `Kill -> bomb.wav`, `CoreHit -> hurt.wav`, `Wave -> level_up.wav`). Files
/// under `assets/sounds/` are generated placeholders (see
/// `assets/sounds/README.md`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Sfx {
    /// Starting a run from the menu.
    MenuSelect,
    /// A tower is placed or upgraded.
    Build,
    /// A tower fires.
    Shot,
    /// An enemy is killed.
    Kill,
    /// The Core takes a hit.
    CoreHit,
    /// A new wave begins.
    Wave,
    /// The run ends.
    GameOver,
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// The orbit pivot at the Core; carries `PointRotation` and is the camera's parent.
#[derive(Component)]
struct CameraRig;

/// The gameplay camera (child of the rig). Pointer projection queries this.
#[derive(Component)]
struct MainCamera;

/// The Core entity (has `Health`).
#[derive(Component)]
struct Core;

/// The translucent placement preview shown while in build mode.
#[derive(Component)]
struct Ghost;

/// A placed tower.
#[derive(Component)]
struct Tower {
    spec: usize,
    level: u32,
    range: f32,
    damage: f32,
    fire: Cooldown,
    turret: Entity,
}

/// A tower's turret child (carries `SmoothLookRotation`).
#[derive(Component)]
struct Turret;

/// A live enemy walking toward the Core.
#[derive(Component)]
struct Enemy {
    hp: f32,
    speed: f32,
    core_damage: f32,
    reward: u32,
    radius: f32,
    /// Set true once it has hit the Core, to avoid double-hits before despawn.
    spent: bool,
}

/// A short-lived hitscan tracer line, drawn by gizmos until `ttl` elapses.
#[derive(Component)]
struct Tracer {
    from: Vec3,
    to: Vec3,
    ttl: f32,
}

/// HUD text tags.
#[derive(Component)]
struct HudText;

// ---------------------------------------------------------------------------
// Setup
// ---------------------------------------------------------------------------

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    catalog: Res<Catalog>,
) {
    // Sounds (paths are relative to `assets/`). Reuse the shared placeholder wavs.
    commands.insert_resource(SoundBank::load(
        &asset_server,
        [
            (Sfx::MenuSelect, "menu_select"),
            (Sfx::Build, "pickup"),
            (Sfx::Shot, "launch"),
            (Sfx::Kill, "bomb"),
            (Sfx::CoreHit, "hurt"),
            (Sfx::Wave, "level_up"),
            (Sfx::GameOver, "game_over"),
        ],
    ));

    // Shared render assets.
    let enemy_mesh = meshes.add(TriangleMeshBuilder::new_octahedron(2).build());
    let enemy_materials = catalog
        .enemies
        .iter()
        .map(|spec| {
            materials.add(StandardMaterial {
                perceptual_roughness: 0.7,
                ..glowing_material(spec.color(), LinearRgba::from(spec.color()) * 0.25)
            })
        })
        .collect();
    let tower_base_mesh = meshes.add(Cylinder::new(0.7, 0.6).mesh().resolution(16).build());
    // A barrel pointing along +X (the turret's local forward we aim).
    let turret_mesh = meshes.add(Cuboid::new(1.4, 0.35, 0.35));
    let spark_mesh = meshes.add(Sphere::new(0.25).mesh().ico(2).unwrap());

    let ghost_material = materials.add(StandardMaterial {
        alpha_mode: AlphaMode::Blend,
        ..glowing_material(
            Color::srgba(0.5, 0.9, 1.0, 0.4),
            LinearRgba::rgb(0.1, 0.4, 0.6),
        )
    });
    let ghost_bad_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.3, 0.3, 0.4),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    // Emissive HDR (NOT unlit -- unlit skips the lighting pass where emissive is
    // applied, so it would not bloom under `camera/post`).
    let spark_material = materials.add(glowing_material(
        Color::srgb(1.0, 0.95, 0.7),
        LinearRgba::rgb(6.0, 5.0, 2.0),
    ));

    commands.insert_resource(GameAssets {
        enemy_mesh,
        enemy_materials,
        tower_base_mesh,
        turret_mesh,
        ghost_material,
        ghost_bad_material,
        spark_mesh,
        spark_material,
    });

    // Camera rig: a pivot at the Core carrying `PointRotation` (the orbit control),
    // with the camera as a child at a fixed angled offset looking back at the
    // pivot. Rotating the pivot orbits the whole view; the fixed child offset keeps
    // the pleasant starting angle regardless of orbit.
    commands
        .spawn((
            Name::new("Camera Rig"),
            CameraRig,
            PointRotation::default(),
            Transform::default(),
            Visibility::default(),
        ))
        .with_children(|rig| {
            rig.spawn((
                Name::new("Main Camera"),
                MainCamera,
                Camera3d::default(),
                Transform::from_xyz(0.0, CAM_UP, CAM_BACK).looking_at(Vec3::ZERO, Vec3::Y),
                PostProcessingCamera,
                CameraShake {
                    max_offset: Vec3::splat(0.4),
                    ..default()
                },
                AmbientLight {
                    color: Color::srgb(0.7, 0.8, 1.0),
                    brightness: 180.0,
                    ..default()
                },
            ));
        });

    commands.spawn((
        DirectionalLight {
            illuminance: 6500.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, -0.5, 0.0)),
    ));

    // Ground disk.
    commands.spawn((
        Name::new("Ground"),
        Mesh3d(
            meshes.add(
                Cylinder::new(ARENA_RADIUS, 0.4)
                    .mesh()
                    .resolution(48)
                    .build(),
            ),
        ),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.10, 0.12, 0.18),
            perceptual_roughness: 0.95,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.2, 0.0),
    ));

    // The Core: a glowing octahedron with a Health pool. Persistent across states;
    // its health is reset in `start_game`.
    commands.spawn((
        Name::new("Core"),
        Core,
        Health::new(CORE_HEALTH),
        Mesh3d(meshes.add(TriangleMeshBuilder::new_octahedron(2).build())),
        MeshMaterial3d(materials.add(glowing_material(
            Color::srgb(0.6, 0.95, 1.0),
            LinearRgba::rgb(0.6, 2.2, 3.0),
        ))),
        Transform::from_xyz(0.0, CORE_RADIUS, 0.0).with_scale(Vec3::splat(CORE_RADIUS)),
    ));

    // Status bar: FPS only; credits / wave / integrity live in the in-game HUD.
    commands.spawn((status_bar(StatusBarRootConfig::default()),));
    commands.spawn(status_bar_with_fps());
}

// ---------------------------------------------------------------------------
// Orbit camera (transform/point_rotation)
// ---------------------------------------------------------------------------

/// Feed a yaw delta into the rig's `PointRotationInput` from A/D + left/right
/// arrow keys and pointer horizontal drag. Pitch is never touched, so the view
/// only orbits around the vertical axis. Also maintains `DragState` so
/// `place_or_select` can tell a tap from an orbit drag.
fn orbit_camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    pointer: Res<UnifiedPointer>,
    mut drag: ResMut<DragState>,
    mut q_rig: Query<
        (
            &PointRotationOutput,
            &mut PointRotationInput,
            &mut Transform,
        ),
        With<CameraRig>,
    >,
) {
    let Ok((out, mut input, mut transform)) = q_rig.single_mut() else {
        return;
    };
    let dt = time.delta_secs();

    // Apply the accumulated orbit rotation to the pivot. `PointRotationPlugin`
    // integrates `PointRotationInput` into `PointRotationOutput` (a Quat) in
    // PostUpdate but never touches the Transform, so the example must copy it
    // across; without this the camera never actually orbits.
    transform.rotation = out.0;

    let mut yaw = 0.0;

    // Keyboard orbit (yaw only).
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        yaw += ORBIT_YAW_RATE * dt;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        yaw -= ORBIT_YAW_RATE * dt;
    }

    // Pointer drag orbit + tap/drag bookkeeping.
    drag.released_tap = false;
    if pointer.just_pressed {
        drag.start = pointer.screen_pos;
        drag.last = pointer.screen_pos.unwrap_or(Vec2::ZERO);
        drag.moved = false;
    }
    if pointer.pressed {
        if let (Some(pos), Some(start)) = (pointer.screen_pos, drag.start) {
            let delta = pos - drag.last;
            drag.last = pos;
            if pos.distance(start) > TAP_MOVE_THRESHOLD {
                drag.moved = true;
            }
            if drag.moved {
                // Horizontal drag only: yaw orbit, pitch left untouched.
                yaw -= delta.x * ORBIT_DRAG_RATE;
            }
        }
    }
    // Release edge: a press that never dragged is a tap.
    if drag.was_pressed && !pointer.pressed {
        if drag.start.is_some() && !drag.moved {
            drag.released_tap = true;
            drag.tap_pos = Some(drag.last);
        }
        drag.start = None;
        drag.moved = false;
    }
    drag.was_pressed = pointer.pressed;

    // Yaw only: the pitch axis of `PointRotationInput` is always zero, so the
    // camera never tilts up or down from input.
    input.0 = Vec2::new(yaw, 0.0);
}

// ---------------------------------------------------------------------------
// Menu
// ---------------------------------------------------------------------------

fn spawn_menu(mut commands: Commands, high: Res<HighScore<u32>>) {
    commands.spawn((
        Name::new("Main Menu"),
        DespawnOnExit(GameState::Menu),
        centered_screen(),
        children![
            (
                screen_text("BASTION", 76.0, Color::srgb(0.6, 0.95, 1.0)),
                TitlePulse::new(Color::srgb(0.6, 0.95, 1.0)),
            ),
            screen_text("Tap or press Space to defend the Core", 30.0, Color::WHITE),
            screen_text(
                format!("Best: {}", high.best()),
                24.0,
                Color::srgb(0.6, 0.9, 1.0),
            ),
            screen_text(
                "Drag / A-D to orbit - number keys pick a tower - tap or Space to build - tap a tower + U to upgrade",
                18.0,
                Color::srgb(0.7, 0.7, 0.75),
            ),
        ],
    ));
}

fn menu_click(
    mut commands: Commands,
    start: AnyStartPress,
    sfx: Res<SoundBank<Sfx>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if start.just_pressed() {
        commands.play_sfx_volume(sfx.get(Sfx::MenuSelect), 0.7);
        next.set(GameState::Playing);
    }
}

// ---------------------------------------------------------------------------
// Run lifecycle
// ---------------------------------------------------------------------------

/// Reset all run state and the Core when a run begins.
fn start_game(
    mut commands: Commands,
    mut credits: ResMut<Credits>,
    mut score: ResMut<Score>,
    mut wave: ResMut<WaveState>,
    mut build: ResMut<Build>,
    mut selection: ResMut<Selection>,
    mut combo: ResMut<Combo>,
    mut dying: ResMut<DyingTimer>,
    mut q_core: Query<(Entity, &mut Health), With<Core>>,
    q_leftover: Query<Entity, Or<(With<Enemy>, With<Tower>, With<Ghost>, With<Tracer>)>>,
) {
    **credits = START_CREDITS;
    **score = 0;
    *wave = WaveState::default();
    build.spec = None;
    selection.tower = None;
    combo.0.reset();
    dying.remaining = None;

    // Clear any entities left over from a previous run and un-mark the Core.
    for e in q_leftover.iter() {
        commands.entity(e).despawn();
    }
    if let Ok((core, mut health)) = q_core.single_mut() {
        health.current = health.max;
        commands.entity(core).remove::<HealthZeroMarker>();
    }
}

fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        Name::new("HUD"),
        DespawnOnExit(GameState::Playing),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        },
        children![(screen_text("", 26.0, Color::WHITE), HudText)],
    ));
}

#[allow(clippy::too_many_arguments)]
fn update_hud(
    credits: Res<Credits>,
    score: Res<Score>,
    wave: Res<WaveState>,
    combo: Res<Combo>,
    build: Res<Build>,
    selection: Res<Selection>,
    catalog: Res<Catalog>,
    q_core: Query<&Health, With<Core>>,
    q_towers: Query<&Tower>,
    mut q_text: Query<&mut Text, With<HudText>>,
) {
    let Ok(mut text) = q_text.single_mut() else {
        return;
    };
    let integrity = q_core.single().map(|h| h.current).unwrap_or(0.0);
    let specs = &catalog.towers;
    let combo_line = if combo.0.count() >= 2 {
        format!("   COMBO x{}", combo.0.count())
    } else {
        String::new()
    };

    // Second line: what a tap does right now -- upgrade a selected tower, place an
    // armed tower, or the default hint.
    let action_line = match selection.tower.and_then(|e| q_towers.get(e).ok()) {
        Some(tower) => {
            let cost = upgrade_cost(&catalog, tower.spec, tower.level);
            let affordable = if **credits >= cost {
                ""
            } else {
                " -- need more"
            };
            format!(
                "Selected {} Lv{}   press U to upgrade ({}c){}",
                specs[tower.spec].name, tower.level, cost, affordable,
            )
        }
        None => match build.spec {
            Some(i) => format!(
                "Building {} ({}c) -- tap the ground to place, or tap a tower to select",
                specs[i].name, specs[i].cost,
            ),
            None => format!(
                "1-{} pick a tower to build, or tap a tower to select",
                specs.len()
            ),
        },
    };

    text.0 = format!(
        "Core {:.0}%   Credits {}   Wave {}   Score {}{}\n{}",
        (integrity / CORE_HEALTH * 100.0).max(0.0),
        **credits,
        wave.number,
        **score,
        combo_line,
        action_line,
    );
}

// ---------------------------------------------------------------------------
// Waves + enemies
// ---------------------------------------------------------------------------

/// Advance the wave scheduler: release the wave's enemies pack by pack (a whole
/// pack spawns at once, then a `PACK_GAP` breather), and once every pack is out
/// and the field is clear, wait `WAVE_GAP` and open the next (bigger) wave.
fn advance_waves(
    time: Res<Time>,
    mut commands: Commands,
    mut wave: ResMut<WaveState>,
    assets: Res<GameAssets>,
    catalog: Res<Catalog>,
    sfx: Res<SoundBank<Sfx>>,
    q_enemies: Query<(), With<Enemy>>,
) {
    let dt = time.delta_secs();

    // A wave is active while it still has packs to release. When the pack timer
    // elapses, spawn a whole pack at once: `spawn_enemy` picks an independent
    // random ring bearing per enemy, so the pack fans out across the border.
    if wave.packs_left > 0 {
        wave.pack_timer -= dt;
        if wave.pack_timer <= 0.0 {
            wave.pack_timer = PACK_GAP;
            wave.packs_left -= 1;
            for _ in 0..wave.pack_size {
                spawn_enemy(&mut commands, &assets, &catalog, wave.number);
            }
        }
        return;
    }

    // Every pack is out. Once the field is clear, wait out the gap and open the
    // next (bigger) wave.
    if q_enemies.iter().next().is_none() {
        if wave.gap_timer > 0.0 {
            wave.gap_timer -= dt;
            return;
        }
        wave.number += 1;
        wave.pack_size = pack_size(wave.number);
        wave.packs_left = packs_in_wave(wave.number);
        wave.pack_timer = 0.0; // release the first pack immediately
        wave.gap_timer = WAVE_GAP;
        if wave.number > 1 {
            commands.play_sfx_volume(sfx.get(Sfx::Wave), 0.7);
            // NOTE: the juice task (20260705-085338) owns the wave-start camera
            // shake so it is not double-added; only the sound cue lives here.
        }
    }
}

/// Spawn one enemy at a random point on the arena ring, walking toward the Core.
/// The type is a weighted pick over the whole catalog (see `weighted_enemy_index`),
/// so tougher enemies mix in more often as the wave climbs and a new enemy added
/// to the JSON participates automatically.
fn spawn_enemy(commands: &mut Commands, assets: &GameAssets, catalog: &Catalog, wave: usize) {
    let mut rng = rand::rng();
    let idx = weighted_enemy_index(catalog, wave, rng.random::<f32>());
    let spec = &catalog.enemies[idx];

    let angle = rng.random_range(0.0..TAU);
    let mut pos = ring_point(angle);
    pos.y = spec.radius;
    let hp = spec.hp * (1.0 + WAVE_HP_PER * wave as f32);
    let speed = spec.speed * (1.0 + WAVE_SPEED_PER * wave as f32);

    commands.spawn((
        Name::new(spec.name.clone()),
        Enemy {
            hp,
            speed,
            core_damage: spec.core_damage,
            reward: spec.reward,
            radius: spec.radius,
            spent: false,
        },
        Mesh3d(assets.enemy_mesh.clone()),
        MeshMaterial3d(assets.enemy_materials[idx].clone()),
        Transform::from_translation(pos).with_scale(Vec3::splat(spec.radius)),
    ));
}

/// Pick an enemy index by spawn weight at `wave`, given a uniform `roll` in
/// `0..1`. Each enemy's weight is `EnemySpec::weight_at(wave)`, so an enemy with
/// a positive `spawn_weight` in the catalog is selectable with no code change,
/// and a positive `wave_weight` makes it more common in later waves. Falls back
/// to index 0 if every weight is zero.
fn weighted_enemy_index(catalog: &Catalog, wave: usize, roll: f32) -> usize {
    let weights: Vec<f32> = catalog.enemies.iter().map(|e| e.weight_at(wave)).collect();
    let total: f32 = weights.iter().sum();
    if total <= 0.0 {
        return 0;
    }
    let target = roll.clamp(0.0, 1.0) * total;
    let mut acc = 0.0;
    for (i, w) in weights.iter().enumerate() {
        acc += w;
        if target < acc {
            return i;
        }
    }
    catalog.enemies.len() - 1
}

/// Step each enemy toward the Core; on arrival, damage the Core and despawn.
fn move_enemies(
    time: Res<Time>,
    mut commands: Commands,
    sfx: Res<SoundBank<Sfx>>,
    mut q_shake: Query<&mut CameraShakeInput>,
    q_core: Query<Entity, With<Core>>,
    mut q_enemies: Query<(Entity, &mut Transform, &mut Enemy)>,
) {
    let dt = time.delta_secs();
    let Ok(core) = q_core.single() else {
        return;
    };
    for (entity, mut transform, mut enemy) in q_enemies.iter_mut() {
        if enemy.spent {
            continue;
        }
        let mut pos = transform.translation;
        let to_core = Vec3::new(-pos.x, 0.0, -pos.z);
        let dist = to_core.length();
        if dist <= CORE_RADIUS + enemy.radius {
            // Reached the Core: damage it and vanish.
            enemy.spent = true;
            commands.trigger(HealthApplyDamage {
                entity: core,
                source: Some(entity),
                amount: enemy.core_damage,
            });
            commands.play_sfx_volume(sfx.get(Sfx::CoreHit), 0.8);
            if let Ok(mut shake) = q_shake.single_mut() {
                shake.add_trauma += SHAKE_CORE_HIT;
            }
            commands.entity(entity).despawn();
            continue;
        }
        let dir = to_core / dist;
        pos += dir * enemy.speed * dt;
        pos.y = enemy.radius;
        transform.translation = pos;
        // Face travel direction (spin a touch for life).
        transform.rotation = Quat::from_rotation_y(dir.x.atan2(dir.z))
            * Quat::from_rotation_x(time.elapsed_secs() * 1.5);
    }
}

// ---------------------------------------------------------------------------
// Towers: placement, targeting, firing (camera/project + smooth_look_rotation)
// ---------------------------------------------------------------------------

/// Pick / clear the armed tower type with the number keys. The bindings iterate
/// the catalog (tower `i` -> `Digit(i + 1)`), so a tower added to the JSON is
/// buildable with the next number key and no code change.
fn select_build(keys: Res<ButtonInput<KeyCode>>, catalog: Res<Catalog>, mut build: ResMut<Build>) {
    for i in 0..catalog.towers.len() {
        if let Some(key) = digit_key(i) {
            if keys.just_pressed(key) {
                build.spec = Some(i);
            }
        }
    }
    if keys.just_pressed(KeyCode::KeyQ) {
        build.spec = None;
    }
}

/// The number key that arms build slot `i` (0-based): index 0 -> `Digit1`, up to
/// `Digit9`. Returns `None` past nine towers.
fn digit_key(i: usize) -> Option<KeyCode> {
    Some(match i {
        0 => KeyCode::Digit1,
        1 => KeyCode::Digit2,
        2 => KeyCode::Digit3,
        3 => KeyCode::Digit4,
        4 => KeyCode::Digit5,
        5 => KeyCode::Digit6,
        6 => KeyCode::Digit7,
        7 => KeyCode::Digit8,
        8 => KeyCode::Digit9,
        _ => return None,
    })
}

/// Where a given screen position (or, if `None`, the ring in front of the camera)
/// lands on the ground plane. Used for the ghost preview and for placement.
fn placement_point(
    camera: &Camera,
    cam_gt: &GlobalTransform,
    screen: Option<Vec2>,
) -> Option<Vec3> {
    if let Some(screen) = screen {
        pointer_on_plane(
            camera,
            cam_gt,
            screen,
            Vec3::ZERO,
            InfinitePlane3d::new(Vec3::Y),
        )
    } else {
        // Keyboard/autopilot fallback: a point on a ring in front of the camera,
        // toward the Core. The camera's forward projected onto the ground gives
        // the current "front" bearing, so orbiting moves the placement spot.
        let fwd = cam_gt.forward().as_vec3();
        let flat = Vec3::new(fwd.x, 0.0, fwd.z);
        if flat.length_squared() < 1e-4 {
            return Some(Vec3::new(0.0, 0.0, ARENA_RADIUS * 0.5));
        }
        Some(flat.normalize() * (ARENA_RADIUS * 0.5))
    }
}

/// True if a tower may be placed at `p` (inside the arena, clear of the Core and
/// other towers).
fn placement_valid(p: Vec3, towers: &[Vec3]) -> bool {
    if p.length() > ARENA_RADIUS - 0.5 {
        return false;
    }
    if p.length() < TOWER_MIN_CORE_DIST {
        return false;
    }
    towers.iter().all(|t| t.distance(p) >= TOWER_MIN_SPACING)
}

/// Show / move / hide the placement ghost each frame.
fn update_ghost(
    mut commands: Commands,
    build: Res<Build>,
    assets: Res<GameAssets>,
    pointer: Res<UnifiedPointer>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    q_towers: Query<&Transform, (With<Tower>, Without<Ghost>)>,
    q_ghost: Query<Entity, With<Ghost>>,
) {
    let ghost = q_ghost.single().ok();
    let Some(spec_idx) = build.spec else {
        if let Some(g) = ghost {
            commands.entity(g).despawn();
        }
        return;
    };
    let Ok((camera, cam_gt)) = q_camera.single() else {
        return;
    };
    let Some(point) = placement_point(camera, cam_gt, pointer.screen_pos) else {
        return;
    };
    let towers: Vec<Vec3> = q_towers.iter().map(|t| t.translation).collect();
    let valid = placement_valid(point, &towers);
    let material = if valid {
        assets.ghost_material.clone()
    } else {
        assets.ghost_bad_material.clone()
    };
    let _ = spec_idx;

    let transform = Transform::from_translation(point + Vec3::Y * 0.3);
    match ghost {
        Some(g) => {
            commands
                .entity(g)
                .insert((transform, MeshMaterial3d(material)));
        }
        None => {
            commands.spawn((
                Ghost,
                DespawnOnExit(GameState::Playing),
                Mesh3d(assets.tower_base_mesh.clone()),
                MeshMaterial3d(material),
                transform,
            ));
        }
    }
}

/// A tap (a press released without dragging) places the armed tower, or selects a
/// tower under it; Space also places (keyboard/autopilot path). Dragging is an
/// orbit and never places, so the two gestures do not fight.
#[allow(clippy::too_many_arguments)]
fn place_or_select(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    drag: Res<DragState>,
    mut credits: ResMut<Credits>,
    mut build: ResMut<Build>,
    mut selection: ResMut<Selection>,
    assets: Res<GameAssets>,
    catalog: Res<Catalog>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sfx: Res<SoundBank<Sfx>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    q_towers: Query<(Entity, &Transform), With<Tower>>,
) {
    let space = keys.just_pressed(KeyCode::Space);
    if !drag.released_tap && !space {
        return;
    }

    let Ok((camera, cam_gt)) = q_camera.single() else {
        return;
    };
    // A tap projects its screen position; Space uses the ring-front fallback.
    let screen = if drag.released_tap {
        drag.tap_pos
    } else {
        None
    };
    let Some(point) = placement_point(camera, cam_gt, screen) else {
        return;
    };

    // A real tap (not the Space keyboard-place path) landing on an existing tower
    // always selects it -- even while a tower type is armed -- so upgrading is
    // reachable without first pressing Q to leave build mode.
    if drag.released_tap {
        let mut nearest: Option<(Entity, f32)> = None;
        for (e, t) in q_towers.iter() {
            let d = t.translation.distance(point);
            if d < 1.4 && nearest.map(|(_, bd)| d < bd).unwrap_or(true) {
                nearest = Some((e, d));
            }
        }
        if let Some((e, _)) = nearest {
            selection.tower = Some(e);
            return;
        }
    }

    // Otherwise, if a tower type is armed, try to place one at the point.
    if let Some(spec_idx) = build.spec {
        let towers: Vec<Vec3> = q_towers.iter().map(|(_, t)| t.translation).collect();
        let spec = &catalog.towers[spec_idx];
        if placement_valid(point, &towers) && **credits >= spec.cost {
            **credits -= spec.cost;
            spawn_tower(
                &mut commands,
                &assets,
                &catalog,
                &mut materials,
                spec_idx,
                point,
            );
            commands.play_sfx_volume(sfx.get(Sfx::Build), 0.7);
            // Stay armed so several towers can be placed in a row, unless broke.
            if **credits < spec.cost {
                build.spec = None;
            }
        }
        return;
    }

    // A tap on empty ground with nothing armed clears any selection.
    if drag.released_tap {
        selection.tower = None;
    }
}

/// Spawn a tower: a base body plus a turret child carrying `SmoothLookRotation`.
fn spawn_tower(
    commands: &mut Commands,
    assets: &GameAssets,
    catalog: &Catalog,
    materials: &mut Assets<StandardMaterial>,
    spec_idx: usize,
    pos: Vec3,
) {
    let spec = &catalog.towers[spec_idx];
    let body_material = materials.add(StandardMaterial {
        base_color: spec.color(),
        perceptual_roughness: 0.6,
        ..default()
    });
    let turret_material = materials.add(glowing_material(
        spec.color().mix(&Color::WHITE, 0.3),
        LinearRgba::from(spec.color()) * 0.6,
    ));

    // Turret: a barrel pivoting around Y via `smooth_look_rotation`. Its local
    // forward is +X (the long axis of the barrel), so the aim angle is measured to
    // put +X at the target.
    let turret = commands
        .spawn((
            Name::new("Turret"),
            Turret,
            SmoothLookRotation {
                axis: Vec3::Y,
                speed: spec.turn_speed,
                ..default()
            },
            Mesh3d(assets.turret_mesh.clone()),
            MeshMaterial3d(turret_material),
            Transform::from_xyz(0.0, 0.9, 0.0),
        ))
        .id();

    commands
        .spawn((
            Name::new(spec.name.clone()),
            Tower {
                spec: spec_idx,
                level: 1,
                range: spec.range,
                damage: spec.damage,
                fire: Cooldown::new(spec.fire_interval),
                turret,
            },
            Mesh3d(assets.tower_base_mesh.clone()),
            MeshMaterial3d(body_material),
            Transform::from_translation(pos),
        ))
        .add_child(turret);
}

/// Each tower finds the nearest enemy in range, aims its turret at it with
/// `SmoothLookRotation`, and fires (hitscan) when the cooldown is ready and the
/// turret is roughly on target.
#[allow(clippy::too_many_arguments)]
fn aim_and_fire_towers(
    time: Res<Time>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    sfx: Res<SoundBank<Sfx>>,
    mut q_towers: Query<(&Transform, &mut Tower)>,
    mut q_turret: Query<
        (
            &mut Transform,
            &SmoothLookRotation,
            &mut SmoothLookRotationTarget,
            &SmoothLookRotationOutput,
        ),
        (With<Turret>, Without<Tower>),
    >,
    mut q_enemies: Query<(Entity, &Transform, &mut Enemy), (Without<Tower>, Without<Turret>)>,
) {
    let dt = time.delta_secs();
    for (tower_tf, mut tower) in q_towers.iter_mut() {
        tower.fire.tick(dt);
        let tower_pos = tower_tf.translation;

        // Nearest enemy in range.
        let mut target: Option<(Entity, Vec3, f32)> = None;
        for (e, tf, enemy) in q_enemies.iter() {
            if enemy.spent {
                continue;
            }
            let d = tf.translation.distance(tower_pos);
            if d <= tower.range && target.map(|(_, _, bd)| d < bd).unwrap_or(true) {
                target = Some((e, tf.translation, d));
            }
        }

        let Ok((mut turret_tf, look, mut look_target, look_out)) = q_turret.get_mut(tower.turret)
        else {
            continue;
        };

        let Some((enemy_entity, enemy_pos, _)) = target else {
            continue;
        };

        // Aim: yaw so the turret's local +X points at the enemy (horizontal plane).
        let to = enemy_pos - tower_pos;
        let desired = -to.z.atan2(to.x); // rotation about +Y from +X toward +Z
        **look_target = desired;
        turret_tf.rotation = Quat::from_axis_angle(look.axis, **look_out);

        // Fire when ready and roughly aligned.
        let aligned = angle_diff(**look_out, desired).abs() < 0.2;
        if tower.fire.ready() && aligned {
            tower.fire.trigger();
            // Hitscan damage.
            if let Ok((_, _, mut enemy)) = q_enemies.get_mut(enemy_entity) {
                enemy.hp -= tower.damage;
                if enemy.hp <= 0.0 && !enemy.spent {
                    // Mark spent so a second tower firing the same frame cannot
                    // double-reward it; the observer does rewards + fragments.
                    enemy.spent = true;
                    commands.trigger(EnemyKilled {
                        entity: enemy_entity,
                    });
                }
            }
            // Tracer + muzzle spark + sound.
            let muzzle = tower_pos + Vec3::Y * 0.9;
            commands.spawn((
                Tracer {
                    from: muzzle,
                    to: enemy_pos,
                    ttl: 0.06,
                },
                Transform::default(),
            ));
            commands.spawn((
                Mesh3d(assets.spark_mesh.clone()),
                MeshMaterial3d(assets.spark_material.clone()),
                Transform::from_translation(enemy_pos).with_scale(Vec3::splat(0.6)),
                TempEntity(0.12),
            ));
            let mut rng = rand::rng();
            commands.trigger(
                PlaySfx::new(sfx.get(Sfx::Shot))
                    .with_volume(0.18)
                    .with_speed(rng.random_range(1.2..1.5)),
            );
        }
    }
}

/// Fired when a tower's shot drops an enemy to zero hp.
#[derive(EntityEvent)]
struct EnemyKilled {
    entity: Entity,
}

/// Award credits (with combo multiplier), pop a "+N" over the enemy via
/// `world_to_screen`, kick a little shake, and slice the enemy into fragments.
#[allow(clippy::too_many_arguments)]
fn on_enemy_killed(
    killed: On<EnemyKilled>,
    mut commands: Commands,
    mut credits: ResMut<Credits>,
    mut score: ResMut<Score>,
    mut combo: ResMut<Combo>,
    sfx: Res<SoundBank<Sfx>>,
    mut q_shake: Query<&mut CameraShakeInput>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    q_enemy: Query<(&Transform, &Enemy)>,
) {
    let entity = killed.entity;
    let Ok((transform, enemy)) = q_enemy.get(entity) else {
        return;
    };

    let multiplier = combo.0.hit().max(1) as u32;
    let reward = enemy.reward * multiplier;
    **credits += reward;
    **score += reward;

    // "+N" popup anchored over the enemy in screen space (world_to_screen).
    if let Ok((camera, cam_gt)) = q_camera.single() {
        if let Some(screen) = world_to_screen(camera, cam_gt, transform.translation + Vec3::Y) {
            let color = if multiplier > 1 {
                Color::srgb(1.0, 0.85, 0.3)
            } else {
                Color::srgb(0.7, 1.0, 0.7)
            };
            commands.spawn((
                popup(screen, format!("+{reward}"), 26.0, color),
                DespawnOnExit(GameState::Playing),
            ));
        }
    }

    if let Ok(mut shake) = q_shake.single_mut() {
        shake.add_trauma += SHAKE_KILL;
    }
    commands.trigger(
        PlaySfx::new(sfx.get(Sfx::Kill))
            .with_volume(0.4)
            .with_speed(0.9 + 0.05 * multiplier as f32),
    );

    // Slice it into fragments; `on_fragments_spawned` turns them into brief debris.
    commands
        .entity(entity)
        .remove::<Enemy>()
        .insert(ExplodeMesh { fragment_count: 5 });
}

/// Turn the sliced enemy shards into short-lived debris that flies outward, then
/// despawn the shell.
fn on_fragments_spawned(
    insert: On<Insert, ExplodeFragments>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    assets: Res<GameAssets>,
    q: Query<(&ExplodeFragments, &Transform)>,
) {
    let entity = insert.entity;
    let Ok((fragments, transform)) = q.get(entity) else {
        return;
    };
    for fragment in fragments.iter() {
        let Some(cloned) = meshes.get(&fragment.mesh).cloned() else {
            continue;
        };
        let mesh = meshes.add(cloned);
        let dir = (transform.rotation * fragment.direction.as_vec3()).normalize_or_zero();
        let start = transform.translation + dir * 0.2;
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(assets.spark_material.clone()),
            Transform::from_translation(start)
                .with_scale(transform.scale * 0.6)
                .looking_to(dir, Vec3::Y),
            TempEntity(0.5),
        ));
    }
    commands.entity(entity).despawn();
}

/// Draw and age hitscan tracer lines.
fn tick_tracers(
    time: Res<Time>,
    mut commands: Commands,
    mut gizmos: Gizmos,
    mut q: Query<(Entity, &mut Tracer)>,
) {
    let dt = time.delta_secs();
    for (e, mut tracer) in q.iter_mut() {
        gizmos.line(tracer.from, tracer.to, Color::srgb(1.0, 0.95, 0.6));
        tracer.ttl -= dt;
        if tracer.ttl <= 0.0 {
            commands.entity(e).despawn();
        }
    }
}

/// Expire the kill combo when its window lapses.
fn tick_combo(time: Res<Time>, mut combo: ResMut<Combo>) {
    combo.0.tick(time.delta_secs());
}

// ---------------------------------------------------------------------------
// Upgrades
// ---------------------------------------------------------------------------

/// Press U to upgrade the selected tower: spend credits to raise damage and range.
fn upgrade_selected(
    keys: Res<ButtonInput<KeyCode>>,
    mut credits: ResMut<Credits>,
    selection: Res<Selection>,
    catalog: Res<Catalog>,
    sfx: Res<SoundBank<Sfx>>,
    mut commands: Commands,
    mut q_towers: Query<&mut Tower>,
) {
    if !keys.just_pressed(KeyCode::KeyU) {
        return;
    }
    let Some(entity) = selection.tower else {
        return;
    };
    let Ok(mut tower) = q_towers.get_mut(entity) else {
        return;
    };
    let cost = upgrade_cost(&catalog, tower.spec, tower.level);
    if **credits < cost {
        return;
    }
    **credits -= cost;
    tower.level += 1;
    tower.damage *= 1.5;
    tower.range += 1.0;
    commands.play_sfx_volume(sfx.get(Sfx::Build), 1.0);
}

// ---------------------------------------------------------------------------
// Rendering helpers
// ---------------------------------------------------------------------------

/// Draw the arena border ring, and a range ring + selection ring for context.
fn draw_arena(
    mut gizmos: Gizmos,
    selection: Res<Selection>,
    build: Res<Build>,
    q_towers: Query<(Entity, &Transform, &Tower)>,
) {
    // Arena boundary.
    gizmos.circle(
        Isometry3d::new(Vec3::ZERO, Quat::from_rotation_x(FRAC_PI_2)),
        ARENA_RADIUS,
        Color::srgba(0.3, 0.55, 0.9, 0.7),
    );

    // Range ring for the selected tower (or all towers while building).
    for (e, tf, tower) in q_towers.iter() {
        let show = selection.tower == Some(e) || build.spec.is_some();
        if show {
            let color = if selection.tower == Some(e) {
                Color::srgba(1.0, 0.9, 0.4, 0.6)
            } else {
                Color::srgba(0.4, 0.7, 1.0, 0.25)
            };
            gizmos.circle(
                Isometry3d::new(
                    tf.translation + Vec3::Y * 0.05,
                    Quat::from_rotation_x(FRAC_PI_2),
                ),
                tower.range,
                color,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Death and game over
// ---------------------------------------------------------------------------

/// When the Core's health hits zero, kick a big shake + red flash and start the
/// death beat before the game-over screen.
fn on_core_died(
    add: On<Add, HealthZeroMarker>,
    q_core: Query<(), With<Core>>,
    state: Res<State<GameState>>,
    mut commands: Commands,
    mut q_shake: Query<&mut CameraShakeInput>,
    mut dying: ResMut<DyingTimer>,
) {
    if !q_core.contains(add.entity)
        || dying.remaining.is_some()
        || *state.get() != GameState::Playing
    {
        return;
    }
    if let Ok(mut shake) = q_shake.single_mut() {
        shake.add_trauma += SHAKE_DEATH;
    }
    dying.remaining = Some(DYING_BEAT);
    commands
        .spawn(screen_flash(
            Color::srgb(0.9, 0.1, 0.1),
            0.55,
            1.0 / DYING_BEAT,
        ))
        .insert(DespawnOnExit(GameState::Playing));
}

/// Count down the death beat, then switch to the game-over screen.
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

fn record_high_score(score: Res<Score>, mut high: ResMut<HighScore<u32>>) {
    high.record(**score);
}

fn spawn_game_over(
    mut commands: Commands,
    score: Res<Score>,
    wave: Res<WaveState>,
    high: Res<HighScore<u32>>,
) {
    commands
        .spawn((
            Name::new("Game Over"),
            DespawnOnExit(GameState::GameOver),
            centered_screen(),
        ))
        .with_children(|parent| {
            parent.spawn(screen_text(
                "CORE LOST",
                72.0,
                Color::srgb(0.95, 0.35, 0.35),
            ));
            parent.spawn(screen_text(
                format!("Score: {} - reached wave {}", **score, wave.number),
                34.0,
                Color::srgb(0.6, 0.9, 1.0),
            ));
            if high.is_new_best() {
                parent.spawn(screen_text("New best!", 30.0, Color::srgb(0.4, 0.95, 0.5)));
            } else {
                parent.spawn(screen_text(
                    format!("Best: {}", high.best()),
                    24.0,
                    Color::srgb(0.6, 0.9, 1.0),
                ));
            }
            parent.spawn(screen_text(
                "Tap or press Space to return to the menu",
                26.0,
                Color::WHITE,
            ));
        });
}

fn play_game_over_sfx(mut commands: Commands, sfx: Res<SoundBank<Sfx>>) {
    commands.play_sfx_volume(sfx.get(Sfx::GameOver), 0.9);
}

fn gameover_click(start: AnyStartPress, mut next: ResMut<NextState<GameState>>) {
    if start.just_pressed() {
        next.set(GameState::Menu);
    }
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// Shortest signed difference `a - b` wrapped to `[-PI, PI]`.
fn angle_diff(a: f32, b: f32) -> f32 {
    let mut d = (a - b) % TAU;
    if d > PI {
        d -= TAU;
    } else if d < -PI {
        d += TAU;
    }
    d
}

/// A point on the arena ring at `angle`, at ground height (y = 0).
fn ring_point(angle: f32) -> Vec3 {
    Vec3::new(angle.cos() * ARENA_RADIUS, 0.0, angle.sin() * ARENA_RADIUS)
}

/// Credit cost to upgrade a tower of `spec_idx` that is currently at `level`:
/// the spec's flat upgrade cost scaled by the current level, so each successive
/// upgrade costs more.
fn upgrade_cost(catalog: &Catalog, spec_idx: usize, level: u32) -> u32 {
    catalog.towers[spec_idx].upgrade_cost * level
}

/// Enemies per pack in wave `n` (1-based): a floored linear growth off
/// `PACK_SIZE_BASE`, so packs get denser as the game climbs (always >= 1).
fn pack_size(n: usize) -> usize {
    PACK_SIZE_BASE + (n as f32 * PACK_SIZE_PER_WAVE) as usize
}

/// Number of packs released in wave `n` (1-based): grows by `PACKS_PER_WAVE`
/// each wave off `PACKS_BASE`.
fn packs_in_wave(n: usize) -> usize {
    PACKS_BASE + n.saturating_sub(1) * PACKS_PER_WAVE
}

/// Total enemies wave `n` (1-based) spawns: `pack_size * packs_in_wave`. Because
/// both factors climb with `n`, the total ramps super-linearly overall (wave 10
/// spawns far more than a straight-line extrapolation of the early waves would).
/// Individual per-wave jumps jitter, though, because `pack_size` grows in floored
/// steps -- so the ramp is not strictly convex wave to wave, only over a range.
fn wave_size(n: usize) -> usize {
    pack_size(n) * packs_in_wave(n)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_point_is_on_arena() {
        for k in 0..8 {
            let a = k as f32 / 8.0 * TAU;
            let p = ring_point(a);
            assert!((p.length() - ARENA_RADIUS).abs() < 1e-3);
            assert_eq!(p.y, 0.0);
        }
    }

    #[test]
    fn angle_diff_wraps() {
        assert!((angle_diff(0.1, -0.1) - 0.2).abs() < 1e-6);
        // 350deg vs 10deg is a -20deg (not +340deg) difference.
        let a = 350f32.to_radians();
        let b = 10f32.to_radians();
        assert!((angle_diff(a, b) - (-20f32).to_radians()).abs() < 1e-5);
    }

    #[test]
    fn placement_rejects_core_and_overlap_and_outside() {
        let towers = vec![Vec3::new(5.0, 0.0, 0.0)];
        // Too close to the Core.
        assert!(!placement_valid(Vec3::ZERO, &towers));
        // On top of an existing tower.
        assert!(!placement_valid(Vec3::new(5.3, 0.0, 0.0), &towers));
        // Outside the arena.
        assert!(!placement_valid(Vec3::new(ARENA_RADIUS, 0.0, 0.0), &towers));
        // A clear spot.
        assert!(placement_valid(Vec3::new(8.0, 0.0, 4.0), &towers));
    }

    /// The catalog compiled into the binary, for the data-driven tests.
    fn embedded_catalog() -> Catalog {
        serde_json::from_str(EMBEDDED_CATALOG).expect("embedded catalog parses")
    }

    #[test]
    fn embedded_catalog_parses_with_the_shipped_roster() {
        // The embedded copy is the shipped `catalog.json`. Assert the roster the
        // game ships with, including the Sniper tower and Swarm enemy added purely
        // in JSON to prove the catalog is data-driven.
        let catalog = embedded_catalog();
        let towers: Vec<&str> = catalog.towers.iter().map(|t| t.name.as_str()).collect();
        let enemies: Vec<&str> = catalog.enemies.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(towers, ["Gun", "Cannon", "Sniper"]);
        assert_eq!(enemies, ["Runner", "Brute", "Swarm"]);
    }

    #[test]
    fn upgrade_cost_scales_with_level() {
        let catalog = embedded_catalog();
        // Gun (spec 0) has a flat upgrade cost; it scales by the current level, so
        // the second upgrade costs twice the first.
        let base = catalog.towers[0].upgrade_cost;
        assert_eq!(upgrade_cost(&catalog, 0, 1), base);
        assert_eq!(upgrade_cost(&catalog, 0, 2), base * 2);
        assert_eq!(upgrade_cost(&catalog, 0, 3), base * 3);
        // A pricier tower (Cannon, spec 1) costs strictly more to upgrade.
        assert!(upgrade_cost(&catalog, 1, 1) > upgrade_cost(&catalog, 0, 1));
    }

    #[test]
    fn weighted_enemy_index_respects_weights_and_new_entries() {
        // A roll of 0 lands on the first positive-weight enemy (Runner, index 0);
        // a roll near 1 lands on the last catalogued enemy. Works for whatever
        // roster the shipped catalog carries.
        let catalog = embedded_catalog();
        assert_eq!(weighted_enemy_index(&catalog, 0, 0.0), 0);
        assert_eq!(
            weighted_enemy_index(&catalog, 0, 0.999),
            catalog.enemies.len() - 1
        );

        // An enemy appended to the catalog participates purely by data: with every
        // other weight zeroed it is always chosen, proving spawn selection is not
        // hard-coded to the starter roster.
        let mut extended = embedded_catalog();
        for e in &mut extended.enemies {
            e.spawn_weight = 0.0;
            e.wave_weight = 0.0;
        }
        extended.enemies.push(EnemySpec {
            name: "Sentinel".to_string(),
            hp: 6.0,
            speed: 3.4,
            core_damage: 4.0,
            reward: 3,
            radius: 0.4,
            spawn_weight: 1.0,
            wave_weight: 0.0,
            color: [0.4, 0.9, 0.6],
        });
        let last = extended.enemies.len() - 1;
        assert_eq!(weighted_enemy_index(&extended, 5, 0.5), last);
    }

    #[test]
    fn pack_size_and_count_grow() {
        // Both the pack size and the number of packs strictly grow with the wave,
        // and both stay at least 1 (an empty wave would stall the scheduler).
        assert!(pack_size(1) >= 1);
        assert!(packs_in_wave(1) >= 1);
        assert!(pack_size(5) > pack_size(1));
        assert!(packs_in_wave(5) > packs_in_wave(1));
        // The wave-1 pack size matches the base (floor of 0.5 is 0).
        assert_eq!(pack_size(1), PACK_SIZE_BASE);
        assert_eq!(packs_in_wave(1), PACKS_BASE);
    }

    #[test]
    fn wave_size_ramps_super_linearly() {
        // Total = pack_size * packs. Because BOTH factors climb with the wave, the
        // total grows faster than any straight line through the early waves. Take
        // the wave 1->2 slope and project it out to wave 10; the real wave-10 total
        // must overshoot that linear projection. A linear schedule (constant
        // per-wave increment) would land exactly on it, so this proves the ramp
        // accelerates. (Floored pack growth makes adjacent increments jitter, so a
        // wide-range projection is the robust check, not a neighbour comparison.)
        assert_eq!(wave_size(3), pack_size(3) * packs_in_wave(3));
        assert!(wave_size(5) > wave_size(1));
        let slope = wave_size(2) - wave_size(1);
        let linear_at_10 = wave_size(1) + 9 * slope;
        assert!(
            wave_size(10) > linear_at_10,
            "ramp should steepen: wave_size(10)={} not above linear {}",
            wave_size(10),
            linear_at_10,
        );
    }
}
