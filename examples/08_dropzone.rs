//! 08_dropzone: land a ship on the noise planet with a PD controller.
//!
//! A small lunar-lander style game that is the crate's headline demo of
//! `PDControllerPlugin`. A noise-displaced planet (grown from the mesh in
//! `02_planet`) sits at the origin with its own gravity pulling inward. You fly
//! a ship down onto it: thrust counteracts gravity, and the PD controller
//! rotates the rigid body toward whatever attitude you steer to (avian3d
//! torque). Touch down slow and upright to score; hit too hard or too tilted
//! and the hull breaks apart via `mesh/explode`. Landing on the glowing pad and
//! grabbing fuel cans on the way down both boost the score, so the descent is a
//! route to plan, not just a fall to survive.
//!
//! It stitches several crate pieces together at once:
//! - `physics/pd_controller` - orientation control (the whole point).
//! - `mesh/builder` - the planet mesh, and an avian trimesh collider built from
//!   its triangles.
//! - `mesh/explode` + `helpers/temp` - the crash effect.
//! - `camera/skybox` - a procedurally generated starfield (no asset file).
//! - `camera/post` - bloom, so the thruster flame glows.
//! - `camera/chase` - a third-person camera that follows the ship but stays
//!   level with the terrain instead of rolling with the hull.
//! - `ui/status` - altitude / speed / fuel gauges.
//! - `audio` - one-shot sound effects on the key events.
//!
//! It follows the shape of `06_fruitninja`: Bevy states for menu / playing /
//! result, a wasm-friendly window, and placeholder sounds from
//! `assets/sounds/`.
//!
//! Controls:
//! - Space / Up: fire the main thruster (burns fuel).
//! - W / S: pitch the target attitude forward / back (leans thrust to move).
//! - A / D: roll the target attitude left / right (leans thrust to strafe).
//! - Release the steering keys and the ship self-levels back to upright.
//! - Space to start, and to retry from the result screen; Esc for the menu.

use avian3d::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_common_systems::prelude::*;
use clap::Parser;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use rand::Rng;

// --- Tuning constants ------------------------------------------------------

/// Base radius of the planet before noise displacement (world units).
const PLANET_BASE_RADIUS: f32 = 40.0;
/// Octahedron subdivision *depth* passed to `TriangleMeshBuilder::new_octahedron`.
/// This is recursive: the triangle count is `8 * 4^depth`, so keep it small.
/// Depth 6 is `8 * 4096 = 32768` triangles - smooth enough to land on and a
/// cheap static trimesh. (Do NOT raise this to tens: depth 24 is ~2e15
/// triangles and hangs the mesh build.)
const PLANET_RESOLUTION: u32 = 6;
/// Terrain height as a fraction of the base radius (peaks and valleys).
const TERRAIN_AMPLITUDE: f64 = 0.10;
/// How high above the tallest peak the ship starts.
const START_ALTITUDE: f32 = 22.0;

/// Radial gravity acceleration pulling the ship toward the planet centre.
const GRAVITY: f32 = 5.5;
/// Upward acceleration from the main thruster while firing.
const THRUST_ACCEL: f32 = 13.0;

/// Maximum tilt (radians) the target attitude leans away from upright.
const MAX_LEAN: f32 = 0.45;
/// How fast the lean moves toward the steered target (rad/s).
const LEAN_RATE: f32 = 2.5;
/// How fast the lean returns to upright when no steering key is held (rad/s).
const LEAN_DECAY: f32 = 4.0;

/// Starting fuel units.
const START_FUEL: f32 = 100.0;
/// Fuel burned per second while the thruster is firing.
const FUEL_BURN: f32 = 14.0;

/// Fastest impact speed that still counts as a safe landing (m/s).
const LAND_SPEED_MAX: f32 = 4.5;
/// Largest tilt from upright that still counts as a safe landing (radians).
const LAND_TILT_MAX: f32 = 0.35;

/// PD controller natural frequency (Hz): how snappy the attitude response is.
const PD_FREQUENCY: f32 = 2.2;
/// PD controller damping ratio (1.0 is critically damped, no overshoot).
const PD_DAMPING: f32 = 1.0;
/// Torque clamp so the controller cannot apply absurd spins.
const PD_MAX_TORQUE: f32 = 4000.0;

/// Number of pieces the hull breaks into on a crash.
const FRAGMENT_COUNT: usize = 6;
/// Initial outward speed of crash fragments (m/s).
const FRAGMENT_SPEED: f32 = 6.0;
/// How long crash fragments live before despawning (seconds).
const FRAGMENT_LIFETIME: f32 = 4.0;

// --- Landing pad -----------------------------------------------------------

/// Angular offset of the landing pad from the +Y start pole (radians). The ship
/// spawns straight above the pole, so a nonzero offset forces a deliberate
/// lateral steer during descent to score the proximity bonus - that steer is
/// the whole point of the pad.
const PAD_ANGLE: f32 = 0.32;
/// Surface distance (world units) at which the pad proximity bonus reaches zero.
/// Land farther than this from the pad and it scores no pad bonus.
const PAD_REWARD_RADIUS: f32 = 26.0;
/// Maximum pad proximity bonus, for a bullseye touchdown on the pad centre.
const PAD_PROXIMITY_MAX: f32 = 400.0;

// --- Fuel pickups ----------------------------------------------------------

/// Fuel units restored by flying the ship through one fuel can.
const FUEL_CAN_AMOUNT: f32 = 25.0;
/// Distance from the ship centre within which a fuel can is collected.
const FUEL_CAN_PICKUP_RADIUS: f32 = 2.6;
/// How far off the efficient descent line the cans sit (world units); grabbing
/// one is a deliberate detour, not free candy.
const FUEL_CAN_OFFSET: f32 = 9.0;

// --- Descent timer ---------------------------------------------------------

/// "Par" descent time (seconds). Landing faster earns a time bonus that decays
/// to zero at par; a slower landing simply gets no time bonus (never a
/// penalty), so a careful, unhurried approach is never punished.
const PAR_TIME: f32 = 30.0;
/// Maximum time bonus, for an instantaneous (t = 0) landing.
const TIME_BONUS_MAX: f32 = 150.0;

// --- Juice -----------------------------------------------------------------

/// Number of dust puffs kicked up on contact (landing or crash).
const DUST_COUNT: usize = 10;
/// Dust puff lifetime before despawning (seconds).
const DUST_LIFETIME: f32 = 1.1;
/// Camera-shake trauma (0..1) added on a soft landing and on a crash. A crash
/// hits harder. Ported from `07_orbit`.
const LAND_TRAUMA: f32 = 0.3;
const CRASH_TRAUMA: f32 = 0.75;
/// Camera-shake feel: peak jolt offset (world units) and trauma decay per sec.
const SHAKE_MAX_OFFSET: f32 = 0.9;
const SHAKE_DECAY: f32 = 1.6;
/// Floating "+FUEL" popup feel: lifetime (seconds) and rise speed (px/s).
const POPUP_LIFETIME: f32 = 0.9;
const POPUP_RISE_SPEED: f32 = 60.0;

// --- CLI -------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "08_dropzone")]
#[command(version = "1.0.0")]
#[command(
    about = "Land a ship on the noise planet using the PD controller",
    long_about = None
)]
struct Cli;

// --- State -----------------------------------------------------------------

/// The three top-level game states, mirroring `06_fruitninja`.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    /// Title screen with the controls and a prompt to start.
    #[default]
    Menu,
    /// A run in progress: the ship is airborne and under player control.
    Playing,
    /// The run is over (landed or crashed); show the outcome and let the
    /// player retry.
    Result,
}

// --- Resources -------------------------------------------------------------

/// Per-frame steering state, written by the input system and read by the
/// physics systems. `thrust` is edge/level input; the leans are smoothed
/// toward their target so the ship eases into and out of a tilt.
#[derive(Resource, Default)]
struct ShipInput {
    thrust: bool,
    lean_pitch: f32,
    lean_roll: f32,
}

/// Remaining fuel for the current run.
#[derive(Resource, Deref, DerefMut)]
struct Fuel(f32);

/// Live telemetry mirrored into a resource so the status-bar closures (which
/// only get `&World`) can read it cheaply without querying the ship entity.
#[derive(Resource, Default)]
struct Telemetry {
    altitude: f32,
    speed: f32,
    fuel: f32,
    /// Great-circle surface distance from the ship's ground track to the landing
    /// pad (world units), shown on the HUD as a homing hint.
    pad_dist: f32,
}

/// The landing pad the player aims for: a fixed unit direction from the planet
/// centre to the pad on the surface. Proximity to it at touchdown drives a score
/// bonus (see [`landing_score`]). Spawned once and persistent across runs.
#[derive(Resource)]
struct LandingPad {
    /// Unit direction from the planet centre to the pad.
    dir: Vec3,
}

/// Camera-shake energy (trauma, 0..1); decays to zero, jittering the camera
/// while positive. A touchdown (landing or crash) tops it up. Ported from
/// `07_orbit`.
#[derive(Resource, Default)]
struct CameraShake {
    trauma: f32,
}

/// Seconds elapsed in the current run, shown on the HUD and rewarded: a faster
/// safe landing scores a small time bonus (see [`landing_score`]).
#[derive(Resource, Default)]
struct RunTimer(f32);

/// The result of the last run, shown on the result screen.
#[derive(Resource, Default)]
struct Outcome(Option<Landing>);

/// Details of a completed landing attempt.
#[derive(Clone, Copy)]
struct Landing {
    /// Whether the touchdown was within the safe speed and tilt limits.
    landed: bool,
    /// Points awarded (zero on a crash).
    score: i32,
    /// Impact speed at first contact (m/s).
    speed: f32,
    /// Tilt from upright at first contact (radians).
    tilt: f32,
}

/// Handles to the one-shot sound effects, loaded once at startup. The files
/// are the shared placeholder sounds under `assets/sounds/` (see
/// `06_fruitninja`).
#[derive(Resource)]
struct SfxAssets {
    start: Handle<AudioSource>,
    land: Handle<AudioSource>,
    crash: Handle<AudioSource>,
    pickup: Handle<AudioSource>,
}

// --- Markers ---------------------------------------------------------------

/// The player-controlled ship (an avian rigid body).
#[derive(Component)]
struct Ship;

/// The static planet body.
#[derive(Component)]
struct Planet;

/// The glowing thruster flame, a child of the ship scaled by thrust each frame.
#[derive(Component)]
struct Thruster;

/// A floating fuel can; flying the ship through it restores fuel and despawns
/// the can.
#[derive(Component)]
struct FuelCan;

/// The main camera (marker for the camera-shake and popup-projection queries).
#[derive(Component)]
struct MainCamera;

/// A short-lived UI text that rises and fades out (the "+FUEL" pickup popup).
/// Ported from `07_orbit`.
#[derive(Component)]
struct FloatingText {
    /// Seconds since the popup was spawned.
    age: f32,
    /// Total lifetime in seconds; the popup despawns once `age` reaches it.
    lifetime: f32,
    /// Upward screen speed in pixels per second.
    rise_speed: f32,
    /// Base color; its alpha ramps down as the popup ages.
    color: Color,
}

/// The ship's speed captured once per render frame in `PreUpdate`, before the
/// fixed-physics loop runs. On the frame the ship touches down avian's solver
/// has already killed most of the impact velocity, so judging the landing by the
/// live `LinearVelocity` in `resolve_landing` would under-report the impact (a
/// hard crash could even read as a soft landing). Capturing in `PreUpdate` keeps
/// this robust even when a stuttering frame runs several physics substeps: the
/// value predates all of that frame's collisions. `resolve_landing` uses it.
#[derive(Component, Default)]
struct ApproachSpeed(f32);

/// A crash fragment moving under its own simple integrator (decoupled from the
/// physics world, like `05_explode`).
#[derive(Component)]
struct FragmentMotion {
    velocity: Vec3,
    spin: Vec3,
}

// --- Noise wrapper ---------------------------------------------------------

/// Scales a noise function's output. Applying the raw fractal to the unit
/// sphere and then scaling to the planet radius would make mountains as tall as
/// the fractal's full range; damping it here keeps the terrain gentle enough to
/// land on while preserving the fractal's shape.
struct ScaledNoise<N> {
    inner: N,
    amplitude: f64,
}

impl<N: NoiseFn<f64, 3>> NoiseFn<f64, 3> for ScaledNoise<N> {
    fn get(&self, point: [f64; 3]) -> f64 {
        self.inner.get(point) * self.amplitude
    }
}

// --- main ------------------------------------------------------------------

fn main() {
    let _ = Cli::parse();
    let mut app = App::new();

    // On the web the game runs inside a canvas: fit it to its parent so it
    // fills the showcase frame. Ignored on native. Matches `06_fruitninja`.
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

    // Real physics simulation this time (not just the debug renderer): the ship
    // is a dynamic body and the planet a static trimesh.
    app.add_plugins(PhysicsPlugins::default());
    // Planet gravity is radial and applied per-body, so disable the global one.
    app.insert_resource(Gravity(Vec3::ZERO));

    #[cfg(feature = "debug")]
    app.add_plugins(InspectorDebugPlugin);

    // Feeds the status bar's FPS item.
    if !app.is_plugin_added::<bevy::diagnostic::FrameTimeDiagnosticsPlugin>() {
        app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
    }

    // Crate plugins this example drives.
    app.add_plugins(PDControllerPlugin);
    app.add_plugins(SkyboxPlugin);
    app.add_plugins(PostProcessingDefaultPlugin);
    app.add_plugins(ChaseCameraPlugin);
    app.add_plugins(ExplodeMeshPlugin);
    app.add_plugins(TempEntityPlugin);
    app.add_plugins(StatusBarPlugin);
    app.add_plugins(SfxPlugin);

    app.init_state::<GameState>();
    app.init_resource::<ShipInput>();
    app.init_resource::<Telemetry>();
    app.init_resource::<Outcome>();
    app.init_resource::<CameraShake>();
    app.init_resource::<RunTimer>();
    app.insert_resource(Fuel(START_FUEL));

    app.add_systems(Startup, setup);

    // Menu.
    app.add_systems(OnEnter(GameState::Menu), spawn_menu);
    app.add_systems(Update, menu_input.run_if(in_state(GameState::Menu)));

    // Playing.
    app.add_systems(OnEnter(GameState::Playing), start_run);
    app.add_systems(
        Update,
        (
            read_input,
            update_telemetry,
            update_thruster_flame,
            tick_run_timer,
            collect_fuel_cans,
            resolve_landing,
        )
            .run_if(in_state(GameState::Playing)),
    );
    // Attitude target must be set before the controller computes torque, and the
    // torque applied after; both run in FixedUpdate around the controller.
    app.add_systems(
        FixedUpdate,
        (
            set_attitude_target.before(PDControllerSystems::Sync),
            apply_ship_forces.after(PDControllerSystems::Sync),
        )
            .run_if(in_state(GameState::Playing)),
    );
    // Capture the approach speed once per render frame in PreUpdate, before the
    // fixed-physics loop runs, so no collision substep can overwrite it. See
    // `ApproachSpeed`.
    app.add_systems(
        PreUpdate,
        track_approach_speed.run_if(in_state(GameState::Playing)),
    );

    // Result.
    app.add_systems(OnEnter(GameState::Result), spawn_result);
    app.add_systems(Update, result_input.run_if(in_state(GameState::Result)));
    // Clear the landed/parked ship (kept visible through the Result screen) when
    // leaving Result, before the next run spawns a fresh one. A crashed hull is
    // already gone (it despawns on leaving Playing), so this only bites for a
    // successful landing.
    app.add_systems(OnExit(GameState::Result), despawn_ships);

    // These run in every state: fragments and dust keep animating into the
    // result screen, popups fade, the camera frames the planet on menu/result
    // too, and the shake settles wherever it was last kicked.
    app.add_systems(
        Update,
        (move_fragments, animate_floating_text, drive_chase_camera),
    );
    // The camera punch is additive on top of the chase camera's transform, so it
    // must run after the chase sync (PostUpdate). The next chase sync overwrites
    // translation, so the offset never accumulates.
    app.add_systems(
        PostUpdate,
        apply_camera_shake.after(ChaseCameraSystems::Sync),
    );

    app.add_observer(on_fragments_spawned);

    app.run();
}

// --- Startup ---------------------------------------------------------------

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    // Planet mesh: the 02_planet recipe, but with damped noise so it is landable.
    let noise = ScaledNoise {
        inner: Fbm::<Perlin>::new(1)
            .set_frequency(1.1)
            .set_persistence(0.5)
            .set_lacunarity(2.2)
            .set_octaves(6),
        amplitude: TERRAIN_AMPLITUDE,
    };
    let mut builder = TriangleMeshBuilder::new_octahedron(PLANET_RESOLUTION);
    builder.apply_noise(&noise);
    let builder = builder.with_scale(Vec3::splat(PLANET_BASE_RADIUS));

    // An avian trimesh collider straight from the builder's triangles. There is
    // no crate helper for this yet (only one game needs it); if a second game
    // does, lift this into `mesh/builder`.
    let (vertices, indices) = builder.vertices_and_indices();
    let triangles: Vec<[u32; 3]> = indices
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();
    let planet_collider = Collider::trimesh(vertices, triangles);

    commands.spawn((
        Name::new("Planet"),
        Planet,
        Mesh3d(meshes.add(builder.build())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.42, 0.36, 0.30),
            perceptual_roughness: 0.95,
            ..default()
        })),
        Transform::default(),
        RigidBody::Static,
        planet_collider,
    ));

    // Landing pad: a fixed target on the surface, offset from the +Y start pole
    // so reaching it takes a deliberate lateral steer. Placed flush on the real
    // terrain by evaluating the same noise the mesh used (surface radius at a
    // unit direction is `R * (1 + noise(dir))`, matching `apply_noise`).
    let pad_dir = (Quat::from_axis_angle(Vec3::X, PAD_ANGLE) * Vec3::Y).normalize();
    let pad_height = noise.get([pad_dir.x as f64, pad_dir.y as f64, pad_dir.z as f64]) as f32;
    let pad_surface_r = PLANET_BASE_RADIUS * (1.0 + pad_height);
    let pad_pos = pad_dir * pad_surface_r;
    let pad_rot = Quat::from_rotation_arc(Vec3::Y, pad_dir);
    let pad_glow = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.9, 1.0),
        emissive: LinearRgba::rgb(0.3, 5.0, 6.0),
        ..default()
    });
    commands
        .spawn((
            Name::new("Landing Pad"),
            Transform::from_translation(pad_pos).with_rotation(pad_rot),
            Visibility::default(),
        ))
        .with_children(|parent| {
            // A flat glowing ring flush on the surface.
            parent.spawn((
                Name::new("Pad Ring"),
                Mesh3d(meshes.add(Cylinder {
                    radius: 3.0,
                    half_height: 0.12,
                })),
                MeshMaterial3d(pad_glow.clone()),
                Transform::from_xyz(0.0, 0.12, 0.0),
            ));
            // A tall thin beacon so the pad is visible from the start altitude.
            parent.spawn((
                Name::new("Pad Beacon"),
                Mesh3d(meshes.add(Cylinder {
                    radius: 0.22,
                    half_height: 12.0,
                })),
                MeshMaterial3d(pad_glow.clone()),
                Transform::from_xyz(0.0, 12.0, 0.0),
            ));
        });
    commands.insert_resource(LandingPad { dir: pad_dir });

    // A sun plus the ambient fill set in `main` keeps the night side readable.
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 12000.0,
            ..default()
        },
        Transform::from_xyz(50.0, 80.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Procedural starfield skybox: a single stacked cubemap image (6 square
    // faces, height = 6 * width), generated in code so there is no binary asset
    // to ship. The SkyboxPlugin reinterprets it into a cubemap on insert.
    let starfield = images.add(starfield_cubemap(256));

    // The follow camera: skybox behind, bloom on, chasing the ship in a frame
    // that stays level with the terrain.
    commands.spawn((
        Name::new("Main Camera"),
        MainCamera,
        Camera3d::default(),
        Transform::from_xyz(0.0, PLANET_BASE_RADIUS + START_ALTITUDE, -20.0)
            .looking_at(Vec3::Y * PLANET_BASE_RADIUS, Vec3::Y),
        ChaseCamera {
            offset: Vec3::new(0.0, 5.0, -15.0),
            focus_offset: Vec3::new(0.0, -3.0, 5.0),
            smoothing: 0.12,
        },
        SkyboxConfig {
            cubemap: starfield,
            brightness: 600.0,
        },
        PostProcessingCamera,
        // Ambient fill so the planet's night side stays readable. In Bevy 0.19
        // AmbientLight is a per-camera component, not a global resource.
        AmbientLight {
            color: Color::srgb(0.6, 0.7, 0.9),
            brightness: 120.0,
            ..default()
        },
    ));

    // Status bar: FPS, altitude, speed and fuel gauges.
    commands.spawn((status_bar(StatusBarRootConfig::default()),));
    commands.spawn((status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: status_fps_value_fn(),
        color_fn: status_fps_color_fn(),
        prefix: "".to_string(),
        suffix: "fps".to_string(),
    }),));
    commands.spawn((status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: |world: &World| {
            world
                .get_resource::<Telemetry>()
                .map(|t| std::sync::Arc::new(t.altitude.max(0.0).round() as i32) as _)
        },
        color_fn: |_v| Some(Color::srgb(0.7, 0.85, 1.0)),
        prefix: "alt ".to_string(),
        suffix: "m".to_string(),
    }),));
    commands.spawn((status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: |world: &World| {
            world
                .get_resource::<Telemetry>()
                .map(|t| std::sync::Arc::new(t.speed.round() as i32) as _)
        },
        color_fn: |v| {
            let speed = (*v).downcast_ref::<i32>()?;
            Some(if (*speed as f32) > LAND_SPEED_MAX {
                Color::srgb(1.0, 0.3, 0.3)
            } else {
                Color::srgb(0.4, 1.0, 0.5)
            })
        },
        prefix: "spd ".to_string(),
        suffix: "m/s".to_string(),
    }),));
    commands.spawn((status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: |world: &World| {
            world
                .get_resource::<Telemetry>()
                .map(|t| std::sync::Arc::new(t.fuel.max(0.0).round() as u32) as _)
        },
        color_fn: |v| {
            let fuel = (*v).downcast_ref::<u32>()?;
            Some(if *fuel < 20 {
                Color::srgb(1.0, 0.3, 0.3)
            } else if *fuel < 40 {
                Color::srgb(1.0, 0.9, 0.3)
            } else {
                Color::srgb(0.5, 0.9, 1.0)
            })
        },
        prefix: "fuel ".to_string(),
        suffix: "%".to_string(),
    }),));
    commands.spawn((status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: |world: &World| {
            world
                .get_resource::<Telemetry>()
                .map(|t| std::sync::Arc::new(t.pad_dist.round() as i32) as _)
        },
        color_fn: |_v| Some(Color::srgb(0.3, 0.9, 1.0)),
        prefix: "pad ".to_string(),
        suffix: "m".to_string(),
    }),));
    commands.spawn((status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: |world: &World| {
            world
                .get_resource::<RunTimer>()
                .map(|t| std::sync::Arc::new(t.0.round() as i32) as _)
        },
        color_fn: |_v| Some(Color::srgb(0.82, 0.85, 0.9)),
        prefix: "t ".to_string(),
        suffix: "s".to_string(),
    }),));

    commands.insert_resource(SfxAssets {
        start: asset_server.load("sounds/launch.wav"),
        land: asset_server.load("sounds/golden.wav"),
        crash: asset_server.load("sounds/bomb.wav"),
        pickup: asset_server.load("sounds/pickup.wav"),
    });
}

/// Build a stacked cubemap starfield: `size` square faces stacked vertically
/// into one `size` x `6*size` RGBA image. Each face is near-black space with a
/// scatter of stars; the plugin reinterprets the stack into a real cubemap.
fn starfield_cubemap(size: u32) -> Image {
    let width = size;
    let height = size * 6;
    let mut data = vec![0u8; (width * height * 4) as usize];

    // Faint blue-black base so the sky is not pure black.
    for px in data.chunks_exact_mut(4) {
        px[0] = 2;
        px[1] = 3;
        px[2] = 8;
        px[3] = 255;
    }

    // Scatter stars across the whole stacked image. Density is per-pixel so all
    // six faces get roughly the same star count.
    let mut rng = rand::rng();
    let star_count = (width * height) / 140;
    for _ in 0..star_count {
        let x = rng.random_range(0..width);
        let y = rng.random_range(0..height);
        // Mostly white, a few warm/cool tints, varied brightness.
        let b = rng.random_range(150..=255) as u8;
        let (r, g, bl) = match rng.random_range(0..10) {
            0 => (b, (b as f32 * 0.8) as u8, (b as f32 * 0.6) as u8), // warm
            1 => ((b as f32 * 0.7) as u8, (b as f32 * 0.8) as u8, b), // cool
            _ => (b, b, b),                                           // white
        };
        let i = ((y * width + x) * 4) as usize;
        data[i] = r;
        data[i + 1] = g;
        data[i + 2] = bl;
        data[i + 3] = 255;
    }

    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
}

// --- Menu ------------------------------------------------------------------

fn spawn_menu(mut commands: Commands) {
    commands
        .spawn((
            Name::new("Menu UI"),
            DespawnOnExit(GameState::Menu),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(14.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("DROP ZONE"),
                TextFont {
                    font_size: FontSize::Px(72.0),
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.9, 1.0)),
            ));
            parent.spawn((
                Text::new("Land softly on the glowing pad"),
                TextFont {
                    font_size: FontSize::Px(26.0),
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.85, 0.9)),
            ));
            parent.spawn((
                Text::new(
                    "Space/Up: thrust    W/S: pitch    A/D: roll\n\
                     Grab fuel cans on the way down. Land on the beacon for a bonus.\n\n\
                     Press SPACE to launch",
                ),
                TextFont {
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.75, 0.8)),
                TextLayout {
                    justify: Justify::Center,
                    ..default()
                },
            ));
        });
}

fn menu_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left) {
        next.set(GameState::Playing);
    }
}

// --- Playing: setup --------------------------------------------------------

fn start_run(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut fuel: ResMut<Fuel>,
    mut input: ResMut<ShipInput>,
    mut outcome: ResMut<Outcome>,
    mut timer: ResMut<RunTimer>,
    pad: Res<LandingPad>,
    sfx: Res<SfxAssets>,
) {
    fuel.0 = START_FUEL;
    outcome.0 = None;
    timer.0 = 0.0;
    *input = ShipInput::default();
    commands.play_sfx(sfx.start.clone());

    // Fuel cans strung down the descent, pushed off the efficient line so
    // grabbing one is a real routing choice (altitude/control for fuel).
    let can_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.85, 0.4),
        emissive: LinearRgba::rgb(0.2, 2.5, 0.6),
        metallic: 0.3,
        ..default()
    });
    let can_mesh = meshes.add(Cylinder {
        radius: 0.5,
        half_height: 0.7,
    });
    for pos in fuel_can_positions(pad.dir) {
        commands.spawn((
            Name::new("Fuel Can"),
            FuelCan,
            DespawnOnExit(GameState::Playing),
            Mesh3d(can_mesh.clone()),
            MeshMaterial3d(can_material.clone()),
            Transform::from_translation(pos),
        ));
    }

    // Spawn just above the tallest peak, upright (radial up at the +Y pole is
    // world up, so identity is upright).
    let start_pos = ship_start_pos();

    let hull_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.75, 0.78, 0.82),
        metallic: 0.7,
        perceptual_roughness: 0.4,
        ..default()
    });

    let ship = commands
        .spawn((
            Name::new("Ship"),
            Ship,
            // No DespawnOnExit here: a soft landing keeps the parked hull visible
            // on the result screen (cleaned up by `despawn_ships` on leaving
            // Result). A crash re-adds DespawnOnExit(Playing) so the shattered
            // hull vanishes as its fragments fly.
            // A boxy lander body; centred on the origin so `ExplodeMesh` can
            // slice it, and flat-bottomed so it rests instead of rolling.
            Mesh3d(meshes.add(Cuboid::new(1.6, 1.1, 1.6))),
            MeshMaterial3d(hull_material.clone()),
            Transform::from_translation(start_pos),
            // Physics body.
            RigidBody::Dynamic,
            Collider::cuboid(1.6, 1.1, 1.6),
            LinearDamping(0.15),
            AngularDamping(3.0),
            // Force channels we overwrite every FixedUpdate (nested to keep the
            // spawn tuple under Bevy's bundle arity limit).
            (
                ConstantLinearAcceleration::default(),
                ConstantLocalLinearAcceleration::default(),
                ConstantTorque::default(),
            ),
            // Attitude control.
            PDController {
                frequency: PD_FREQUENCY,
                damping_ratio: PD_DAMPING,
                max_torque: PD_MAX_TORQUE,
            },
            // Landing / crash detection.
            CollisionEventsEnabled,
            ApproachSpeed::default(),
        ))
        .id();
    // The controller reads the body it is attached to.
    commands.entity(ship).insert(PDControllerTarget(ship));

    // A nose cone so "upright" reads at a glance.
    let nose_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.35, 0.3),
        ..default()
    });
    commands.entity(ship).with_children(|parent| {
        parent.spawn((
            Name::new("Nose"),
            Mesh3d(meshes.add(Cone {
                radius: 0.7,
                height: 1.0,
            })),
            MeshMaterial3d(nose_material),
            Transform::from_xyz(0.0, 1.0, 0.0),
        ));

        // Emissive flame under the ship; bloom (post) makes it glow. Scaled to
        // near-zero until the thruster fires.
        parent.spawn((
            Name::new("Thruster"),
            Thruster,
            Mesh3d(meshes.add(Sphere::new(0.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.6, 0.1),
                emissive: LinearRgba::rgb(8.0, 3.0, 0.4),
                ..default()
            })),
            Transform::from_xyz(0.0, -0.9, 0.0).with_scale(Vec3::splat(0.001)),
        ));
    });
}

// --- Playing: input --------------------------------------------------------

/// Move `current` toward `target` by at most `max_delta`.
fn move_toward(current: f32, target: f32, max_delta: f32) -> f32 {
    let diff = target - current;
    if diff.abs() <= max_delta {
        target
    } else {
        current + diff.signum() * max_delta
    }
}

fn read_input(keys: Res<ButtonInput<KeyCode>>, time: Res<Time>, mut input: ResMut<ShipInput>) {
    let dt = time.delta_secs();

    input.thrust = keys.pressed(KeyCode::Space) || keys.pressed(KeyCode::ArrowUp);

    // W leans the nose forward (thrust pushes forward), S back.
    let mut target_pitch = 0.0;
    if keys.pressed(KeyCode::KeyW) {
        target_pitch -= MAX_LEAN;
    }
    if keys.pressed(KeyCode::KeyS) {
        target_pitch += MAX_LEAN;
    }
    // A/D roll to strafe.
    let mut target_roll = 0.0;
    if keys.pressed(KeyCode::KeyA) {
        target_roll += MAX_LEAN;
    }
    if keys.pressed(KeyCode::KeyD) {
        target_roll -= MAX_LEAN;
    }

    let pitch_rate = if target_pitch != 0.0 {
        LEAN_RATE
    } else {
        LEAN_DECAY
    };
    let roll_rate = if target_roll != 0.0 {
        LEAN_RATE
    } else {
        LEAN_DECAY
    };
    input.lean_pitch = move_toward(input.lean_pitch, target_pitch, pitch_rate * dt);
    input.lean_roll = move_toward(input.lean_roll, target_roll, roll_rate * dt);
}

// --- Playing: physics ------------------------------------------------------

/// Feed the PD controller its target attitude: upright relative to the planet
/// surface, tilted by the current lean. Runs before the controller.
fn set_attitude_target(
    input: Res<ShipInput>,
    mut q_ship: Query<(&Position, &mut PDControllerInput), With<Ship>>,
) {
    let Ok((position, mut pd_input)) = q_ship.single_mut() else {
        return;
    };

    let radial_up = position.0.normalize_or(Vec3::Y);
    let upright = Quat::from_rotation_arc(Vec3::Y, radial_up);
    let lean = Quat::from_axis_angle(Vec3::X, input.lean_pitch)
        * Quat::from_axis_angle(Vec3::Z, input.lean_roll);
    **pd_input = upright * lean;
}

/// Apply radial gravity, thrust, and the controller's torque to the ship, and
/// burn fuel. Runs after the controller has produced its torque.
fn apply_ship_forces(
    input: Res<ShipInput>,
    time: Res<Time>,
    mut fuel: ResMut<Fuel>,
    mut q_ship: Query<
        (
            &Position,
            &PDControllerOutput,
            &mut ConstantLinearAcceleration,
            &mut ConstantLocalLinearAcceleration,
            &mut ConstantTorque,
        ),
        With<Ship>,
    >,
) {
    let Ok((position, torque, mut gravity, mut thrust, mut spin)) = q_ship.single_mut() else {
        return;
    };

    // Radial gravity toward the planet centre.
    let radial_up = position.0.normalize_or(Vec3::Y);
    gravity.0 = -radial_up * GRAVITY;

    // Thrust along the ship's local up while firing and fuelled.
    if input.thrust && fuel.0 > 0.0 {
        thrust.0 = Vec3::Y * THRUST_ACCEL;
        fuel.0 = (fuel.0 - FUEL_BURN * time.delta_secs()).max(0.0);
    } else {
        thrust.0 = Vec3::ZERO;
    }

    // Attitude torque from the PD controller.
    spin.0 = torque.0;
}

/// Record the ship's speed once per render frame in `PreUpdate`, before the
/// fixed-physics loop. On the touchdown frame this predates every collision
/// substep, so it holds the speed just before contact - see [`ApproachSpeed`].
fn track_approach_speed(mut q_ship: Query<(&LinearVelocity, &mut ApproachSpeed), With<Ship>>) {
    if let Ok((velocity, mut approach)) = q_ship.single_mut() {
        approach.0 = velocity.0.length();
    }
}

// --- Playing: presentation -------------------------------------------------

fn update_telemetry(
    fuel: Res<Fuel>,
    pad: Res<LandingPad>,
    mut telemetry: ResMut<Telemetry>,
    q_ship: Query<(&Transform, &LinearVelocity), With<Ship>>,
) {
    let Ok((transform, velocity)) = q_ship.single() else {
        return;
    };
    telemetry.altitude = transform.translation.length() - PLANET_BASE_RADIUS;
    telemetry.speed = velocity.0.length();
    telemetry.fuel = fuel.0;
    // Great-circle surface distance from the ship's ground track to the pad.
    let ground_dir = transform.translation.normalize_or(Vec3::Y);
    telemetry.pad_dist = ground_dir.angle_between(pad.dir) * PLANET_BASE_RADIUS;
}

fn update_thruster_flame(
    input: Res<ShipInput>,
    fuel: Res<Fuel>,
    time: Res<Time>,
    mut q_flame: Query<&mut Transform, With<Thruster>>,
) {
    let Ok(mut transform) = q_flame.single_mut() else {
        return;
    };
    let firing = input.thrust && fuel.0 > 0.0;
    // A little flicker so the flame is not a static blob.
    let flicker = 1.0 + 0.15 * (time.elapsed_secs() * 30.0).sin();
    let target = if firing {
        Vec3::new(0.7, 2.2 * flicker, 0.7)
    } else {
        Vec3::splat(0.001)
    };
    transform.scale = transform.scale.lerp(target, 0.4);
}

/// Where the ship spawns each run: straight above the +Y pole, clear of the
/// tallest peak. Also the camera's fallback anchor when there is no ship (Menu /
/// Result), so those screens frame the planet from above instead of parking the
/// camera inside it.
fn ship_start_pos() -> Vec3 {
    Vec3::Y * (PLANET_BASE_RADIUS * (1.0 + TERRAIN_AMPLITUDE as f32) + START_ALTITUDE)
}

/// Drive the chase camera every frame in every state: follow the ship when it
/// exists, otherwise sit at the spawn vantage. Running in the menu too lets the
/// smoothed camera state settle on the vantage before a run starts, so Playing
/// opens on the ship instead of swooping out from the planet centre.
fn drive_chase_camera(
    q_ship: Query<&Transform, With<Ship>>,
    mut q_cam: Query<&mut ChaseCameraInput>,
) {
    let Ok(mut input) = q_cam.single_mut() else {
        return;
    };
    let anchor_pos = q_ship
        .single()
        .map(|ship| ship.translation)
        .unwrap_or_else(|_| ship_start_pos());
    // Orient the camera frame to the terrain (radial up) rather than the hull,
    // so the view does not roll when we lean.
    let radial_up = anchor_pos.normalize_or(Vec3::Y);
    input.anchor_pos = anchor_pos;
    input.anchor_rot = Quat::from_rotation_arc(Vec3::Y, radial_up);
}

// --- Playing: fuel pickups & timer -----------------------------------------

/// Where the fuel cans sit for a run: strung down the start -> pad descent and
/// pushed off that line so collecting one is a deliberate sideways detour.
fn fuel_can_positions(pad_dir: Vec3) -> [Vec3; 3] {
    let start = ship_start_pos();
    let pad = pad_dir * PLANET_BASE_RADIUS;
    let along = (pad - start).normalize_or(Vec3::NEG_Y);
    // A unit vector perpendicular to the descent chord: the "off the line" axis.
    let side = along.any_orthonormal_vector();
    [0.3, 0.5, 0.7].map(|t| start.lerp(pad, t) + side * FUEL_CAN_OFFSET)
}

/// Count the run timer up while the ship is flying.
fn tick_run_timer(time: Res<Time>, mut timer: ResMut<RunTimer>) {
    timer.0 += time.delta_secs();
}

/// Collect any fuel can the ship flies through: restore fuel (capped at the
/// starting tank so the `%` gauge stays sane), chirp, and float a "+FUEL" popup.
fn collect_fuel_cans(
    mut commands: Commands,
    mut fuel: ResMut<Fuel>,
    sfx: Res<SfxAssets>,
    q_ship: Query<&Transform, With<Ship>>,
    q_cans: Query<(Entity, &Transform), With<FuelCan>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let Ok(ship) = q_ship.single() else {
        return;
    };
    for (can, transform) in q_cans.iter() {
        if ship.translation.distance(transform.translation) > FUEL_CAN_PICKUP_RADIUS {
            continue;
        }
        commands.entity(can).despawn();
        fuel.0 = (fuel.0 + FUEL_CAN_AMOUNT).min(START_FUEL);
        commands.trigger(PlaySfx::new(sfx.pickup.clone()).with_volume(0.8));

        // Float a "+FUEL" popup at the can's screen position (skip if off-screen
        // or behind the camera).
        if let Ok((camera, cam_transform)) = q_camera.single() {
            if let Ok(viewport_pos) = camera.world_to_viewport(cam_transform, transform.translation)
            {
                spawn_floating_text(
                    &mut commands,
                    viewport_pos,
                    "+FUEL",
                    30.0,
                    Color::srgb(0.6, 1.0, 0.7),
                );
            }
        }
    }
}

/// Spawn a floating "+FUEL" popup at a viewport position, scoped to `Playing`.
/// It rises and fades via `animate_floating_text`. Ported from `07_orbit`.
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
            font_size: FontSize::Px(size),
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

/// Advance floating popups: rise up the screen, fade out, and despawn at the end
/// of their lifetime. Ported from `07_orbit`.
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

// --- Playing: landing / crash ----------------------------------------------

#[allow(clippy::too_many_arguments)]
fn resolve_landing(
    mut collisions: MessageReader<CollisionStart>,
    q_ship: Query<(Entity, &Transform, &ApproachSpeed), With<Ship>>,
    fuel: Res<Fuel>,
    pad: Res<LandingPad>,
    timer: Res<RunTimer>,
    sfx: Res<SfxAssets>,
    mut shake: ResMut<CameraShake>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut outcome: ResMut<Outcome>,
    mut next: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    let Ok((ship, transform, approach)) = q_ship.single() else {
        return;
    };

    // The ship's only possible collision is with the planet, so any event
    // touching it is a touchdown.
    let touched = collisions
        .read()
        .any(|c| c.collider1 == ship || c.collider2 == ship);
    if !touched {
        return;
    }

    // Use the pre-impact speed, not the live velocity: by now the solver has
    // already absorbed the collision, so the live value under-reports the hit.
    let speed = approach.0;
    let up = transform.translation.normalize_or(Vec3::Y);
    let tilt = (transform.rotation * Vec3::Y).angle_between(up);
    // Great-circle surface distance from the touchdown point to the pad centre.
    let pad_dist = up.angle_between(pad.dir) * PLANET_BASE_RADIUS;

    // Kick up dust at the contact point either way.
    spawn_dust(
        &mut commands,
        &mut meshes,
        &mut materials,
        transform.translation,
        up,
    );

    if speed <= LAND_SPEED_MAX && tilt <= LAND_TILT_MAX {
        let score = landing_score(fuel.0, speed, tilt, pad_dist, timer.0);
        outcome.0 = Some(Landing {
            landed: true,
            score,
            speed,
            tilt,
        });
        commands.play_sfx(sfx.land.clone());
        shake.trauma = (shake.trauma + LAND_TRAUMA).min(1.0);
        // Freeze the hull where it touched down so it stays visibly parked on
        // the pad through the result screen (it has no DespawnOnExit, so it
        // survives the state change; `despawn_ships` clears it on leaving
        // Result). Static + zeroed velocity stops any post-contact drift.
        commands.entity(ship).insert((
            RigidBody::Static,
            LinearVelocity::default(),
            AngularVelocity::default(),
        ));
    } else {
        outcome.0 = Some(Landing {
            landed: false,
            score: 0,
            speed,
            tilt,
        });
        // Break the hull apart. The `on_fragments_spawned` observer turns the
        // slices into flying debris; DespawnOnExit(Playing) here removes the
        // shell as the state changes to Result, after the fragments spawn.
        commands.entity(ship).insert((
            ExplodeMesh {
                fragment_count: FRAGMENT_COUNT,
            },
            DespawnOnExit(GameState::Playing),
        ));
        commands.play_sfx(sfx.crash.clone());
        shake.trauma = (shake.trauma + CRASH_TRAUMA).min(1.0);
    }

    next.set(GameState::Result);
}

/// Reward a gentle, upright, fuel-efficient touchdown, landed near the pad and
/// flown briskly. Pure function, unit-tested below.
fn landing_score(fuel: f32, speed: f32, tilt: f32, pad_dist: f32, run_time: f32) -> i32 {
    let fuel_bonus = fuel * 3.0;
    let soft_bonus = (LAND_SPEED_MAX - speed).max(0.0) * 40.0;
    let level_bonus = (LAND_TILT_MAX - tilt).max(0.0) * 200.0;
    // Closer to the pad centre is worth more, fading linearly to zero at the
    // reward radius; nothing beyond it.
    let proximity_bonus = (1.0 - pad_dist / PAD_REWARD_RADIUS).max(0.0) * PAD_PROXIMITY_MAX;
    // A brisk descent earns a bonus that fades to zero at par time; a slower
    // landing just gets none (never negative), so care is not punished.
    let time_bonus = ((PAR_TIME - run_time) / PAR_TIME).clamp(0.0, 1.0) * TIME_BONUS_MAX;
    (100.0 + fuel_bonus + soft_bonus + level_bonus + proximity_bonus + time_bonus).round() as i32
}

/// Kick up a short-lived puff of dust particles at a contact point, biased
/// outward along the surface normal. Reuses the `FragmentMotion` integrator
/// (`move_fragments`) and `helpers/temp` for auto-despawn, exactly like the
/// crash debris, so no new machinery is needed.
fn spawn_dust(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    at: Vec3,
    up: Vec3,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.62, 0.57, 0.5),
        perceptual_roughness: 1.0,
        ..default()
    });
    let mesh = meshes.add(Sphere::new(0.18));
    let mut rng = rand::rng();
    for _ in 0..DUST_COUNT {
        let scatter = Vec3::new(
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
            rng.random_range(-1.0..1.0),
        );
        let dir = (up + scatter * 0.8).normalize_or(up);
        commands.spawn((
            Name::new("Dust"),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(at),
            FragmentMotion {
                velocity: dir * rng.random_range(2.0..5.0),
                spin: Vec3::ZERO,
            },
            TempEntity(DUST_LIFETIME),
        ));
    }
}

/// Despawn any leftover ship (the parked hull from a soft landing) when leaving
/// the result screen, before the next run spawns a fresh one.
fn despawn_ships(mut commands: Commands, q_ships: Query<Entity, With<Ship>>) {
    for ship in q_ships.iter() {
        commands.entity(ship).despawn();
    }
}

/// Jolt the camera by a decaying random offset while trauma is positive. Runs
/// after the chase camera writes its transform (PostUpdate), so the offset is
/// additive; the next chase sync overwrites translation, so it never
/// accumulates. Ported from `07_orbit`.
fn apply_camera_shake(
    time: Res<Time>,
    mut shake: ResMut<CameraShake>,
    mut q_camera: Query<&mut Transform, With<MainCamera>>,
) {
    shake.trauma = (shake.trauma - SHAKE_DECAY * time.delta_secs()).max(0.0);
    // Square the trauma so small residual energy fades to nothing quickly.
    let amount = shake.trauma * shake.trauma;
    if amount <= 0.0 {
        return;
    }
    let Ok(mut transform) = q_camera.single_mut() else {
        return;
    };
    let mut rng = rand::rng();
    let offset = Vec3::new(
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
    ) * SHAKE_MAX_OFFSET
        * amount;
    transform.translation += offset;
}

/// Turn each mesh slice into an independent flying fragment (see `05_explode`).
fn on_fragments_spawned(
    insert: On<Insert, ExplodeFragments>,
    q_fragments: Query<(&ExplodeFragments, &Transform)>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok((fragments, transform)) = q_fragments.get(insert.entity) else {
        return;
    };

    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.72, 0.75),
        metallic: 0.6,
        perceptual_roughness: 0.5,
        ..default()
    });

    let mut rng = rand::rng();
    for fragment in fragments.iter() {
        // Fragment meshes are in the ship's local frame; place them at the ship
        // and blast them outward along their slice direction.
        let world_dir = transform.rotation * fragment.direction.as_vec3();
        commands.spawn((
            Name::new("Fragment"),
            Mesh3d(fragment.mesh.clone()),
            MeshMaterial3d(material.clone()),
            *transform,
            FragmentMotion {
                velocity: world_dir * FRAGMENT_SPEED,
                spin: Vec3::new(
                    rng.random_range(-4.0..4.0),
                    rng.random_range(-4.0..4.0),
                    rng.random_range(-4.0..4.0),
                ),
            },
            TempEntity(FRAGMENT_LIFETIME),
        ));
    }
}

fn move_fragments(time: Res<Time>, mut q: Query<(&mut Transform, &mut FragmentMotion)>) {
    let dt = time.delta_secs();
    for (mut transform, mut motion) in &mut q {
        // Pull fragments toward the planet centre so debris settles.
        let down = -transform.translation.normalize_or(Vec3::Y);
        motion.velocity += down * GRAVITY * dt;
        transform.translation += motion.velocity * dt;
        let spin = motion.spin;
        transform.rotate_local_x(spin.x * dt);
        transform.rotate_local_y(spin.y * dt);
        transform.rotate_local_z(spin.z * dt);
    }
}

// --- Result ----------------------------------------------------------------

fn spawn_result(mut commands: Commands, outcome: Res<Outcome>) {
    let (title, title_color, detail) = match outcome.0 {
        Some(Landing {
            landed: true,
            score,
            speed,
            tilt,
        }) => (
            "LANDED!".to_string(),
            Color::srgb(0.5, 1.0, 0.6),
            format!(
                "Score {score}\nTouchdown {:.1} m/s at {:.0} deg tilt",
                speed,
                tilt.to_degrees()
            ),
        ),
        Some(Landing { speed, tilt, .. }) => (
            "CRASHED".to_string(),
            Color::srgb(1.0, 0.4, 0.4),
            format!(
                "Hit at {:.1} m/s, {:.0} deg tilt\n(limits: {:.1} m/s, {:.0} deg)",
                speed,
                tilt.to_degrees(),
                LAND_SPEED_MAX,
                LAND_TILT_MAX.to_degrees()
            ),
        ),
        None => ("RESULT".to_string(), Color::WHITE, String::new()),
    };

    commands
        .spawn((
            Name::new("Result UI"),
            DespawnOnExit(GameState::Result),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title),
                TextFont {
                    font_size: FontSize::Px(64.0),
                    ..default()
                },
                TextColor(title_color),
            ));
            parent.spawn((
                Text::new(detail),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.88, 0.92)),
                TextLayout {
                    justify: Justify::Center,
                    ..default()
                },
            ));
            parent.spawn((
                Text::new("SPACE to retry    ESC for menu"),
                TextFont {
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.75, 0.8)),
            ));
        });
}

fn result_input(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Space) {
        next.set(GameState::Playing);
    } else if keys.just_pressed(KeyCode::Escape) {
        next.set(GameState::Menu);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A perfect touchdown: full tank, dead stop, upright, dead centre on the
    /// pad, instant. Every bonus maxes out.
    #[test]
    fn landing_score_bullseye_beats_a_far_scrappy_landing() {
        let bullseye = landing_score(START_FUEL, 0.0, 0.0, 0.0, 0.0);
        // Same flight, but landed one reward-radius away and at par time: it
        // loses the whole proximity and time bonus.
        let far_slow = landing_score(START_FUEL, 0.0, 0.0, PAD_REWARD_RADIUS, PAR_TIME);
        assert!(
            bullseye > far_slow,
            "a bullseye ({bullseye}) should beat a far, slow landing ({far_slow})"
        );
        // The gap is exactly the proximity + time bonus that the far/slow run forgoes.
        let expected_gap = (PAD_PROXIMITY_MAX + TIME_BONUS_MAX).round() as i32;
        assert_eq!(bullseye - far_slow, expected_gap);
    }

    /// The proximity bonus decays to zero at the reward radius and never goes
    /// negative past it (landing far away simply forgoes the bonus).
    #[test]
    fn pad_proximity_bonus_clamps_at_the_reward_radius() {
        let at_edge = landing_score(
            0.0,
            LAND_SPEED_MAX,
            LAND_TILT_MAX,
            PAD_REWARD_RADIUS,
            PAR_TIME,
        );
        let way_past = landing_score(
            0.0,
            LAND_SPEED_MAX,
            LAND_TILT_MAX,
            PAD_REWARD_RADIUS * 4.0,
            PAR_TIME,
        );
        // Both forgo every optional bonus, so both are the flat base score.
        assert_eq!(at_edge, 100);
        assert_eq!(way_past, 100);
    }

    /// A slower-than-par landing is never punished: its time bonus floors at
    /// zero rather than going negative.
    #[test]
    fn time_bonus_never_penalizes_a_slow_landing() {
        let at_par = landing_score(START_FUEL, 0.0, 0.0, 0.0, PAR_TIME);
        let well_over_par = landing_score(START_FUEL, 0.0, 0.0, 0.0, PAR_TIME * 3.0);
        assert_eq!(at_par, well_over_par);
    }

    /// The fuel cans thread down the descent, off the efficient line, and stay
    /// clear of both the spawn point and the ground (finite, above the surface).
    #[test]
    fn fuel_cans_sit_off_the_line_and_above_the_surface() {
        let pad_dir = (Quat::from_axis_angle(Vec3::X, PAD_ANGLE) * Vec3::Y).normalize();
        for pos in fuel_can_positions(pad_dir) {
            assert!(pos.is_finite());
            // Above the planet surface (with terrain headroom), so a can is
            // never buried in the ground.
            let surface = PLANET_BASE_RADIUS * (1.0 + TERRAIN_AMPLITUDE as f32);
            assert!(
                pos.length() > surface,
                "fuel can at {pos:?} (r={}) should clear the surface r={surface}",
                pos.length()
            );
            // Genuinely off the descent line: measurably displaced sideways.
            let start = ship_start_pos();
            let pad = pad_dir * PLANET_BASE_RADIUS;
            let along = (pad - start).normalize_or(Vec3::NEG_Y);
            let on_line = start + along * (pos - start).dot(along);
            assert!(
                pos.distance(on_line) > 1.0,
                "fuel can at {pos:?} should sit off the start->pad line"
            );
        }
    }
}
