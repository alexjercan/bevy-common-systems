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
//! The descent is also dangerous: rock monoliths ring the pad so the final
//! approach has to thread a gap, asteroids drift through the corridor on the
//! `transform/random_sphere_orbit` driver, and a time-varying wind shoves the
//! ship sideways so you have to lean into it. Grazing a rock or asteroid costs
//! ship integrity (a `Health` pool) rather than instantly killing -- a light
//! graze chips it, a hard smash empties it and ends the run. Every hazard scales
//! off one `HAZARD_DIFFICULTY` knob.
//!
//! It stitches several crate pieces together at once:
//! - `physics/pd_controller` - orientation control (the whole point).
//! - `mesh/builder` - the planet mesh, an avian trimesh collider built from its
//!   triangles, and the lumpy asteroid mesh.
//! - `mesh/explode` + `helpers/temp` - the crash effect and the asteroid shatter.
//! - `transform/random_sphere_orbit` - the drifting asteroids.
//! - `health` - ship structural integrity: grazes drain it, zero ends the run.
//! - `camera/skybox` - a procedurally generated starfield (no asset file).
//! - `camera/post` - bloom, so the thruster flame glows.
//! - `camera/chase` - a third-person camera that follows the ship but stays
//!   level with the terrain instead of rolling with the hull.
//! - `ui/status` - altitude / speed / fuel / hull / wind gauges.
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
//!
//! Touch (mobile): hold the left side of the screen to thrust; drag a floating
//! stick on the right side to lean (deflection = lean angle, release to level).
//! Touch is an additional writer of the same steering state, so the keyboard
//! keeps working unchanged.

use avian3d::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    input::touch::Touch,
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

/// The landing pad spawns at a random spot each run, at a polar angle in this
/// band from the +Y start pole (radians). The lower bound keeps it off the exact
/// spawn point (so reaching it is always a deliberate steer); the upper bound
/// keeps it within a fuelled lateral reach and well clear of the antipode, where
/// the `from_rotation_arc` upright target would be singular.
const PAD_ANGLE_MIN: f32 = 0.18;
const PAD_ANGLE_MAX: f32 = 0.5;
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
/// How many fuel cans to keep floating in the play area at once. The spawner
/// tops up to this over time as cans are collected, so there are never zero and
/// never a swarm.
const FUEL_CAN_TARGET: usize = 3;
/// Seconds between fuel-can top-up spawns while below the target count.
const FUEL_CAN_SPAWN_INTERVAL: f32 = 2.5;
/// Polar-angle cap (radians) of the region above the planet the cans scatter in.
const FUEL_CAN_SPREAD: f32 = 0.55;

// --- Direction guide -------------------------------------------------------

/// How far above the ship the diegetic guide arrow hovers (world units).
const GUIDE_ARROW_HEIGHT: f32 = 2.6;

// --- Touch controls (mobile) -----------------------------------------------

/// Fraction of the window width given to the left thrust zone; the rest is the
/// right steer zone. A touch is routed by the zone its first contact lands in.
const THRUST_ZONE_FRAC: f32 = 0.4;
/// Radius (logical px) of the steer stick: a full-deflection drag from the touch
/// origin to this distance maps to `MAX_LEAN`.
const STEER_RADIUS_PX: f32 = 110.0;
/// Dead zone (logical px) around the steer origin where no lean is applied, so
/// a resting thumb reads as "level".
const STEER_DEAD_PX: f32 = 16.0;

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

// --- Hazards ----------------------------------------------------------------

/// One scalar every hazard knob multiplies by, so the whole descent's danger can
/// be tuned (and later ramped) from a single place. 1.0 is the shipped baseline;
/// an on-difficulty ramp is deliberately out of scope for this pass, but the
/// constants below are all expressed relative to this so a ramp is a one-liner.
const HAZARD_DIFFICULTY: f32 = 1.0;

/// The ship's collision radius used by the proximity checks against asteroids
/// (the hull is a 1.6-cube, so a little over its half-extent).
const SHIP_COLLISION_RADIUS: f32 = 1.1;

// Rough terrain: rock monoliths ringed around the pad so the final approach has
// to thread a gap. Discrete colliders are used instead of cranking the global
// terrain amplitude, which would also disturb the pad's flush placement and the
// landing feel (see the docs note for this decision).
/// Number of rock monoliths around the pad (before the difficulty scale).
const ROCK_COUNT: usize = 6;
/// Angular radius of the rock ring from the pad direction (radians).
const ROCK_RING_ANGLE: f32 = 0.085;
/// Fraction of the ring the rocks span, leaving the remainder as a threadable
/// gap (0.78 -> ~280 degrees of rocks, ~80 of clear approach).
const ROCK_RING_SPAN_FRAC: f32 = 0.78;
/// Rock monolith size: square footprint half-width and height (world units).
const ROCK_HALF_WIDTH: f32 = 1.2;
const ROCK_HEIGHT: f32 = 6.0;

// Asteroids: debris drifting through the descent corridor, driven by the crate's
// `RandomSphereOrbit` (the same family `07_orbit` uses for its wanderers).
/// Number of drifting asteroids (before the difficulty scale).
const ASTEROID_COUNT: usize = 7;
/// Asteroid collision radius (world units) for the proximity hit check.
const ASTEROID_RADIUS: f32 = 1.3;
/// Altitude band above the base surface the asteroids wander in, so they clutter
/// the corridor rather than orbiting uselessly high or scraping the ground. The
/// top stays below the ship's spawn altitude so a run never opens inside one.
const ASTEROID_ALT_MIN: f32 = 5.0;
const ASTEROID_ALT_MAX: f32 = START_ALTITUDE - 6.0;
/// Asteroid drift speed range (radians/sec along the surface).
const ASTEROID_SPEED_MIN: f32 = 0.25;
const ASTEROID_SPEED_MAX: f32 = 0.6;
/// Fragments an asteroid shatters into when the ship strikes it.
const ASTEROID_FRAGMENTS: usize = 5;

// Ship structural integrity: a `Health` pool that a graze chips and a hard hit
// empties. Terrain contact stays instant-crash (see docs); only obstacles and
// asteroids route through integrity.
/// Ship integrity (Health) at the start of a run.
const SHIP_MAX_INTEGRITY: f32 = 100.0;
/// Integrity lost per m/s of impact speed on an obstacle or asteroid, so a slow
/// graze costs a little and a fast smash can empty the pool in one hit.
const INTEGRITY_DAMAGE_PER_SPEED: f32 = 9.0;
/// A damage floor, so even a crawling contact costs something.
const INTEGRITY_MIN_DAMAGE: f32 = 8.0;

// Wind: a time-varying tangential acceleration the player fights with lean. One
// more world-space acceleration folded into the gravity channel, telegraphed by
// drifting streak particles and a HUD gauge.
/// Peak tangential wind acceleration at full gust (world units/s^2), before the
/// difficulty scale. Compare GRAVITY (5.5) and THRUST_ACCEL (13.0): strong
/// enough to shove, weak enough to counter with lean.
const WIND_PEAK_ACCEL: f32 = 3.2;
/// How fast the gust envelope pulses and the wind bearing rotates (rad/sec). Both
/// are slow and smooth so the wind is readable and counterable, not a coin flip.
const WIND_GUST_FREQ: f32 = 0.55;
const WIND_TURN_SPEED: f32 = 0.16;
/// Wind-streak telegraph: spawn interval at full gust (seconds, longer when the
/// gust is weaker), particle lifetime, and drift speed.
const WIND_STREAK_INTERVAL: f32 = 0.11;
const WIND_STREAK_LIFETIME: f32 = 0.75;
const WIND_STREAK_SPEED: f32 = 9.0;

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

/// Touch state distilled from `Touches` each frame, merged into `ShipInput` by
/// `read_input` as an additional writer alongside the keyboard. A left-zone
/// touch drives thrust; a right-zone touch is a floating steer stick whose
/// deflection sets the lean target (see [`touch_lean`]). Both act at once.
#[derive(Resource, Default)]
struct TouchControl {
    /// A thrust-zone touch is currently held.
    thrust: bool,
    /// A steer-zone touch is currently held (its lean target is `lean`, which is
    /// zero inside the dead zone).
    steering: bool,
    /// Steer lean target as `(roll, pitch)` radians, already clamped to
    /// `MAX_LEAN`.
    lean: Vec2,
    /// Pointer id of the steer touch currently driving the stick, if any.
    steer_id: Option<u64>,
    /// Floating origin of the steer stick (logical px); slides to keep the
    /// finger within `STEER_RADIUS_PX`.
    steer_origin: Vec2,
    /// Current steer finger position (logical px), for the HUD knob.
    steer_pos: Vec2,
}

/// Set true the first time any touch is seen, and never cleared. The virtual pad
/// is only shown once this is set, so a keyboard/PC session never sees it while a
/// phone reveals it the instant a thumb lands - no platform detection needed.
#[derive(Resource, Default)]
struct TouchSeen(bool);

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
    /// Ship structural integrity as a percentage of the max (0..100), mirrored
    /// from the ship's `Health` so the status-bar closure stays cheap.
    integrity: f32,
    /// Great-circle surface distance from the ship's ground track to the landing
    /// pad (world units), shown on the HUD as a homing hint.
    pad_dist: f32,
}

/// The landing pad the player aims for, re-rolled to a random spot each run.
/// Proximity to it at touchdown drives a score bonus (see [`landing_score`]) and
/// it is the target the guide arrow points to.
#[derive(Resource)]
struct LandingPad {
    /// Unit direction from the planet centre to the pad (used for scoring by
    /// great-circle distance).
    dir: Vec3,
    /// World position of the pad on the surface (used by the guide arrow).
    pos: Vec3,
}

/// The planet's terrain noise, kept so a new run can sample the surface radius at
/// the freshly-rolled pad direction (matching how the mesh was displaced).
#[derive(Resource)]
struct PlanetNoise(ScaledNoise<Fbm<Perlin>>);

/// Shared mesh/material for fuel cans, built once so the spawner does not
/// allocate new assets on every top-up.
#[derive(Resource)]
struct FuelCanAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

/// Countdown to the next fuel-can top-up spawn (see [`maintain_fuel_cans`]).
#[derive(Resource, Default)]
struct FuelSpawner {
    timer: f32,
}

/// Shared asteroid and rock assets, built once so per-run spawning does not
/// allocate new meshes/materials. The streak assets telegraph the wind.
#[derive(Resource)]
struct HazardAssets {
    asteroid_mesh: Handle<Mesh>,
    asteroid_material: Handle<StandardMaterial>,
    rock_mesh: Handle<Mesh>,
    rock_material: Handle<StandardMaterial>,
    streak_mesh: Handle<Mesh>,
    streak_material: Handle<StandardMaterial>,
}

/// The time-varying tangential wind. A single phase drives both the gust
/// envelope (magnitude) and the slowly rotating bearing (direction); the actual
/// world-space acceleration is recomputed each frame in the ship's tangent plane
/// and cached in `accel` for `apply_ship_forces`, the streak telegraph and the
/// HUD gauge to read. Reset each run in `start_run`.
#[derive(Resource, Default)]
struct Wind {
    /// Phase accumulator (seconds of flight) driving bearing and gust.
    phase: f32,
    /// Current world-space wind acceleration (tangential to the surface).
    accel: Vec3,
    /// Countdown to the next telegraph streak.
    streak_timer: f32,
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
    /// Whether the run ended because integrity was depleted (an asteroid/obstacle
    /// structural failure) rather than a hard terrain impact. Implies `!landed`.
    destroyed: bool,
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

/// The landing-pad beacon (a per-run entity), kept visible into the result
/// screen and cleared on leaving it.
#[derive(Component)]
struct Pad;

/// The diegetic guide arrow that hovers by the ship and points to the pad.
#[derive(Component)]
struct GuideArrow;

/// A drifting asteroid, driven around the planet by `RandomSphereOrbit`. Grazing
/// one damages the ship's integrity and shatters the asteroid.
#[derive(Component)]
struct Asteroid;

/// A static rock monolith ringed around the pad; a solid obstacle whose contact
/// costs integrity (a hard hit ends the run). Marker so collision handling can
/// tell it apart from the planet surface and the asteroids.
#[derive(Component)]
struct Obstacle;

/// The main camera (marker for the camera-shake and popup-projection queries).
#[derive(Component)]
struct MainCamera;

/// Root of the touch-HUD overlay; its visibility is gated on [`TouchSeen`] so the
/// pad only appears once the device is actually touched.
#[derive(Component)]
struct TouchHud;

/// Marker for the steer-stick ring node of the touch HUD (moves to the live
/// origin while steering).
#[derive(Component)]
struct SteerRingUi;

/// Marker for the steer-stick knob node of the touch HUD (follows the finger
/// while steering, hidden otherwise).
#[derive(Component)]
struct SteerKnobUi;

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
    // Hazards: drifting asteroids ride the random-orbit driver, and a graze routes
    // through the ship's integrity Health pool.
    app.add_plugins(SphereRandomOrbitPlugin);
    app.add_plugins(HealthPlugin);

    app.init_state::<GameState>();
    app.init_resource::<ShipInput>();
    app.init_resource::<TouchControl>();
    app.init_resource::<TouchSeen>();
    app.init_resource::<Telemetry>();
    app.init_resource::<Outcome>();
    app.init_resource::<CameraShake>();
    app.init_resource::<RunTimer>();
    app.init_resource::<FuelSpawner>();
    app.init_resource::<Wind>();
    app.insert_resource(Fuel(START_FUEL));

    app.add_systems(Startup, setup);

    // Menu.
    app.add_systems(OnEnter(GameState::Menu), spawn_menu);
    app.add_systems(Update, menu_input.run_if(in_state(GameState::Menu)));

    // Playing.
    app.add_systems(OnEnter(GameState::Playing), (start_run, spawn_touch_hud));
    app.add_systems(
        Update,
        (
            // Distil touches into TouchControl before read_input merges them.
            update_touch_control.before(read_input),
            read_input,
            update_telemetry,
            update_thruster_flame,
            tick_run_timer,
            maintain_fuel_cans,
            collect_fuel_cans,
            update_guide_arrow,
            update_touch_hud.after(update_touch_control),
            update_wind,
            spawn_wind_streaks.after(update_wind),
            resolve_asteroid_hits,
            resolve_collisions,
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
    // Clear the parked ship and the pad beacon (both kept visible through the
    // Result screen) when leaving Result, before the next run spawns fresh ones.
    // A crashed hull is already gone (it despawns on leaving Playing).
    app.add_systems(OnExit(GameState::Result), cleanup_run_scene);

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
    // Copy each asteroid's random-orbit position onto its Transform, after the
    // orbit driver has advanced it (same handoff as `07_orbit`).
    app.add_systems(
        PostUpdate,
        apply_asteroid_transforms
            .after(SphereRandomOrbitSystems::Sync)
            .run_if(in_state(GameState::Playing)),
    );

    app.add_observer(on_fragments_spawned);
    // Structural failure: when the ship's integrity Health hits zero, end the run
    // as a crash (explode the hull, go to the result screen).
    app.add_observer(on_ship_destroyed);

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

    // The pad is re-rolled each run in `start_run`, which samples the terrain
    // noise to sit the beacon flush on the surface; keep the noise for that.
    commands.insert_resource(PlanetNoise(noise));
    // Shared fuel-can assets, so the spawner does not allocate per top-up.
    commands.insert_resource(FuelCanAssets {
        mesh: meshes.add(Cylinder {
            radius: 0.5,
            half_height: 0.7,
        }),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.85, 0.4),
            emissive: LinearRgba::rgb(0.2, 2.5, 0.6),
            metallic: 0.3,
            ..default()
        }),
    });
    // Placeholder pad so the resource always exists before the first run reads
    // it; `start_run` overwrites it with the rolled position.
    commands.insert_resource(LandingPad {
        dir: Vec3::Y,
        pos: Vec3::Y * PLANET_BASE_RADIUS,
    });

    // Shared hazard assets. The asteroid is a lumpy unit octahedron (the crate's
    // own `TriangleMeshBuilder`, so it also slices cleanly for `mesh/explode`);
    // scale to size per-instance. The rock is a monolith cuboid, and the streak a
    // thin translucent shard drifting downwind.
    let mut asteroid_builder = TriangleMeshBuilder::new_octahedron(2);
    asteroid_builder.apply_noise(&ScaledNoise {
        inner: Fbm::<Perlin>::new(7).set_octaves(3).set_frequency(1.6),
        amplitude: 0.35,
    });
    commands.insert_resource(HazardAssets {
        asteroid_mesh: meshes.add(asteroid_builder.build()),
        asteroid_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.34, 0.30, 0.28),
            perceptual_roughness: 1.0,
            ..default()
        }),
        rock_mesh: meshes.add(Cuboid::new(
            ROCK_HALF_WIDTH * 2.0,
            ROCK_HEIGHT,
            ROCK_HALF_WIDTH * 2.0,
        )),
        rock_material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.30, 0.27, 0.24),
            perceptual_roughness: 0.98,
            ..default()
        }),
        streak_mesh: meshes.add(Cuboid::new(0.08, 0.08, 0.7)),
        streak_material: materials.add(StandardMaterial {
            base_color: Color::srgba(0.75, 0.85, 1.0, 0.5),
            emissive: LinearRgba::new(0.4, 0.6, 1.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
    });

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
                .map(|t| std::sync::Arc::new(t.integrity.max(0.0).round() as u32) as _)
        },
        color_fn: |v| {
            let integrity = (*v).downcast_ref::<u32>()?;
            Some(if *integrity < 25 {
                Color::srgb(1.0, 0.3, 0.3)
            } else if *integrity < 55 {
                Color::srgb(1.0, 0.9, 0.3)
            } else {
                Color::srgb(0.6, 0.9, 0.7)
            })
        },
        prefix: "hull ".to_string(),
        suffix: "%".to_string(),
    }),));
    commands.spawn((status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: |world: &World| {
            // Wind as a 0..100 fraction of peak gust, so the gauge reads like the
            // others; the drifting streaks show the direction.
            world.get_resource::<Wind>().map(|w| {
                let frac = w.accel.length() / (WIND_PEAK_ACCEL * HAZARD_DIFFICULTY);
                std::sync::Arc::new((frac.clamp(0.0, 1.0) * 100.0).round() as u32) as _
            })
        },
        color_fn: |v| {
            let wind = (*v).downcast_ref::<u32>()?;
            Some(if *wind > 66 {
                Color::srgb(1.0, 0.4, 0.4)
            } else if *wind > 33 {
                Color::srgb(1.0, 0.85, 0.4)
            } else {
                Color::srgb(0.6, 0.85, 1.0)
            })
        },
        prefix: "wind ".to_string(),
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
                     Follow the arrow to the beacon. Grab fuel cans on the way down.\n\n\
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
    touches: Res<Touches>,
    mut next: ResMut<NextState<GameState>>,
) {
    // A tap also starts, so the wasm build is enterable on a phone (winit-on-web
    // delivers taps as touches, not synthesized mouse clicks).
    if keys.just_pressed(KeyCode::Space)
        || mouse.just_pressed(MouseButton::Left)
        || touches.any_just_pressed()
    {
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
    mut shake: ResMut<CameraShake>,
    mut spawner: ResMut<FuelSpawner>,
    mut touch: ResMut<TouchControl>,
    mut wind: ResMut<Wind>,
    noise: Res<PlanetNoise>,
    can_assets: Res<FuelCanAssets>,
    hazards: Res<HazardAssets>,
    sfx: Res<SfxAssets>,
) {
    fuel.0 = START_FUEL;
    outcome.0 = None;
    timer.0 = 0.0;
    shake.trauma = 0.0;
    spawner.timer = 0.0;
    *input = ShipInput::default();
    *touch = TouchControl::default();
    // Wind is a cross-run resource, so it must be reset here (fresh gust phase),
    // like the other per-run resources above.
    *wind = Wind::default();
    commands.play_sfx(sfx.start.clone());

    // Roll a fresh landing pad somewhere in the reachable cap around the pole,
    // and place its beacon flush on the real terrain (surface radius at a unit
    // direction is `R * (1 + noise(dir))`, matching `apply_noise`).
    let mut rng = rand::rng();
    let pad_dir = random_cap_dir(&mut rng, PAD_ANGLE_MIN, PAD_ANGLE_MAX);
    let pad_height = noise
        .0
        .get([pad_dir.x as f64, pad_dir.y as f64, pad_dir.z as f64]) as f32;
    let pad_pos = pad_dir * (PLANET_BASE_RADIUS * (1.0 + pad_height));
    commands.insert_resource(LandingPad {
        dir: pad_dir,
        pos: pad_pos,
    });
    let pad_glow = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.9, 1.0),
        emissive: LinearRgba::rgb(0.3, 5.0, 6.0),
        ..default()
    });
    commands
        .spawn((
            Name::new("Landing Pad"),
            Pad,
            Transform::from_translation(pad_pos)
                .with_rotation(Quat::from_rotation_arc(Vec3::Y, pad_dir)),
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

    // Ring rock monoliths around the pad, spanning most of the circle so a gap is
    // left to thread on final approach. Each stands radially on the real terrain.
    let rock_count = ((ROCK_COUNT as f32) * HAZARD_DIFFICULTY).round() as usize;
    if rock_count > 0 {
        // A tangent basis at the pad, so a bearing maps to a direction offset.
        let east = pad_dir.cross(Vec3::Y).normalize_or(Vec3::X);
        let north = pad_dir.cross(east).normalize_or(Vec3::Z);
        let ring_base = rng.random_range(0.0..std::f32::consts::TAU);
        for i in 0..rock_count {
            let bearing = ring_base
                + std::f32::consts::TAU * ROCK_RING_SPAN_FRAC * (i as f32 / rock_count as f32);
            let tangent = east * bearing.cos() + north * bearing.sin();
            let rock_dir =
                (pad_dir * ROCK_RING_ANGLE.cos() + tangent * ROCK_RING_ANGLE.sin()).normalize();
            let surf_height = noise
                .0
                .get([rock_dir.x as f64, rock_dir.y as f64, rock_dir.z as f64])
                as f32;
            let surface = rock_dir * (PLANET_BASE_RADIUS * (1.0 + surf_height));
            // Sink the base slightly into the surface so no gap shows under it.
            let pos = surface + rock_dir * (ROCK_HEIGHT * 0.5 - 0.4);
            // Random yaw around the monolith's up axis so the ring is not uniform.
            let yaw = Quat::from_axis_angle(rock_dir, rng.random_range(0.0..std::f32::consts::TAU));
            commands.spawn((
                Name::new("Rock"),
                Obstacle,
                DespawnOnExit(GameState::Playing),
                Mesh3d(hazards.rock_mesh.clone()),
                MeshMaterial3d(hazards.rock_material.clone()),
                Transform::from_translation(pos)
                    .with_rotation(yaw * Quat::from_rotation_arc(Vec3::Y, rock_dir)),
                RigidBody::Static,
                Collider::cuboid(ROCK_HALF_WIDTH * 2.0, ROCK_HEIGHT, ROCK_HALF_WIDTH * 2.0),
            ));
        }
    }

    // Scatter drifting asteroids through the descent corridor, each wandering on
    // the sphere via the crate's `RandomSphereOrbit` (as `07_orbit` does). They
    // start near the pole (high phi) so they clutter where the ship descends.
    let asteroid_count = ((ASTEROID_COUNT as f32) * HAZARD_DIFFICULTY).round() as usize;
    for _ in 0..asteroid_count {
        let alt = rng.random_range(ASTEROID_ALT_MIN..ASTEROID_ALT_MAX);
        let radius = PLANET_BASE_RADIUS * (1.0 + TERRAIN_AMPLITUDE as f32) + alt;
        let theta = rng.random_range(0.0..std::f32::consts::TAU);
        // Bias elevation toward the +Y pole (phi -> FRAC_PI_2) so asteroids seed
        // inside the descent cap rather than around the far equator.
        let phi = std::f32::consts::FRAC_PI_2 - rng.random_range(0.0..0.7);
        let angular_speed = rng.random_range(ASTEROID_SPEED_MIN..ASTEROID_SPEED_MAX);
        commands.spawn((
            Name::new("Asteroid"),
            Asteroid,
            DespawnOnExit(GameState::Playing),
            RandomSphereOrbit {
                radius,
                angular_speed,
                center: Vec3::ZERO,
                initial_theta: theta,
                initial_phi: phi,
            },
            Mesh3d(hazards.asteroid_mesh.clone()),
            MeshMaterial3d(hazards.asteroid_material.clone()),
            Transform::from_scale(Vec3::splat(ASTEROID_RADIUS)),
        ));
    }

    // Seed the fuel field to the target count at random spots; the maintain
    // system tops it back up over time as cans are collected.
    for _ in 0..FUEL_CAN_TARGET {
        spawn_fuel_can(&mut commands, &can_assets, random_fuel_can_pos(&mut rng));
    }

    // Diegetic guide arrow: hovers by the ship each frame and points to the pad.
    commands.spawn((
        Name::new("Guide Arrow"),
        GuideArrow,
        DespawnOnExit(GameState::Playing),
        Mesh3d(meshes.add(Cone {
            radius: 0.35,
            height: 1.1,
        })),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.85, 0.2),
            emissive: LinearRgba::rgb(5.0, 3.5, 0.3),
            ..default()
        })),
        Transform::from_translation(ship_start_pos()),
    ));

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
            // Structural integrity: grazes chip it, a hard hit empties it. Zero
            // integrity ends the run via the `on_ship_destroyed` observer.
            Health::new(SHIP_MAX_INTEGRITY),
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

fn read_input(
    keys: Res<ButtonInput<KeyCode>>,
    touch: Res<TouchControl>,
    time: Res<Time>,
    mut input: ResMut<ShipInput>,
) {
    let dt = time.delta_secs();

    // Thrust is either input; touch is an additive writer alongside the keyboard.
    input.thrust = keys.pressed(KeyCode::Space) || keys.pressed(KeyCode::ArrowUp) || touch.thrust;

    // Lean target: while a steer touch is held the touch stick fully preempts the
    // keyboard (not additive) -- including the dead zone, where a resting thumb
    // reads as "level" and so suppresses A/D/W/S. Releasing the stick falls back
    // to the keyboard, which self-levels when nothing is pressed.
    let (target_roll, target_pitch) = if touch.steering {
        (touch.lean.x, touch.lean.y)
    } else {
        // W leans the nose forward (thrust pushes forward), S back; A/D roll.
        let mut pitch = 0.0;
        if keys.pressed(KeyCode::KeyW) {
            pitch -= MAX_LEAN;
        }
        if keys.pressed(KeyCode::KeyS) {
            pitch += MAX_LEAN;
        }
        let mut roll = 0.0;
        if keys.pressed(KeyCode::KeyA) {
            roll += MAX_LEAN;
        }
        if keys.pressed(KeyCode::KeyD) {
            roll -= MAX_LEAN;
        }
        (roll, pitch)
    };

    // Ease toward the target, self-levelling faster when the target is centred
    // (shared by both input paths -- "released = level").
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

/// Map a steer-stick deflection (finger offset from the floating origin, logical
/// px) to a lean target `(roll, pitch)` in radians, clamped to `MAX_LEAN`, with
/// a dead zone. Screen +x (drag right) rolls right (like `D`), and screen +y
/// (drag down, since UI y grows downward) pitches back (like `S`). The combined
/// deflection *vector* is clamped to `MAX_LEAN`, so a diagonal drag tops out at
/// `MAX_LEAN` total -- slightly less than the keyboard, which reaches `MAX_LEAN`
/// on each axis independently (~1.41x on the diagonal). Pure, unit-tested below.
fn touch_lean(offset: Vec2, radius: f32, dead: f32) -> Vec2 {
    let len = offset.length();
    if len <= dead {
        return Vec2::ZERO;
    }
    let dir = offset / len;
    let mag = ((len - dead) / (radius - dead)).clamp(0.0, 1.0);
    let deflect = dir * mag;
    // drag right (+x) -> roll like D (negative roll); drag down (+y) -> pitch
    // back like S (positive pitch).
    Vec2::new(-deflect.x * MAX_LEAN, deflect.y * MAX_LEAN)
}

/// Distil the raw `Touches` into `TouchControl`: route each touch by the zone its
/// first contact landed in (never re-checked as it moves), so a lean drag that
/// wanders into the thrust zone never misfires thrust and vice versa.
fn update_touch_control(
    touches: Res<Touches>,
    windows: Query<&Window>,
    mut ctl: ResMut<TouchControl>,
    mut seen: ResMut<TouchSeen>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    // The first touch marks this as a touch device and reveals the pad.
    if touches.any_just_pressed() {
        seen.0 = true;
    }
    let split_x = window.width() * THRUST_ZONE_FRAC;
    let in_thrust_zone = |t: &Touch| t.start_position().x < split_x;

    // State is derived fresh from the currently-pressed touches each frame (keyed
    // by where each STARTED), not latched to one id. So a second finger in a zone
    // and a finger held across a run restart both keep working, and lifting one
    // finger never cuts input while another is still down.
    ctl.thrust = touches.iter().any(in_thrust_zone);

    // Steer: keep the tracked touch while it is pressed and still a steer-zone
    // touch, otherwise adopt another pressed steer-zone touch (re-centring the
    // floating origin on it).
    let steer = ctl
        .steer_id
        .and_then(|id| touches.get_pressed(id))
        .filter(|t| !in_thrust_zone(t))
        .or_else(|| touches.iter().find(|t| !in_thrust_zone(t)));
    if let Some(touch) = steer {
        let pos = touch.position();
        if ctl.steer_id != Some(touch.id()) {
            // Newly adopted stick: origin starts under the finger.
            ctl.steer_id = Some(touch.id());
            ctl.steer_origin = pos;
        }
        // Slide the origin so the finger never leaves the stick radius.
        let offset = pos - ctl.steer_origin;
        if offset.length() > STEER_RADIUS_PX {
            ctl.steer_origin = pos - offset.normalize() * STEER_RADIUS_PX;
        }
        ctl.steer_pos = pos;
        ctl.lean = touch_lean(pos - ctl.steer_origin, STEER_RADIUS_PX, STEER_DEAD_PX);
        ctl.steering = true;
    } else {
        ctl.steer_id = None;
        ctl.steering = false;
        ctl.lean = Vec2::ZERO;
    }
}

// --- Playing: touch HUD ----------------------------------------------------

/// Spawn the on-screen virtual-pad overlay for the run: a faint left thrust zone
/// and a right-side steer stick (ring + knob). Spawned hidden and revealed by
/// `update_touch_hud` once a touch is seen ([`TouchSeen`]), so a PC session never
/// shows it and a phone reveals it the instant a thumb lands -- no platform
/// detection needed.
fn spawn_touch_hud(mut commands: Commands) {
    let ring_d = STEER_RADIUS_PX * 2.0;
    commands
        .spawn((
            Name::new("Touch HUD"),
            TouchHud,
            DespawnOnExit(GameState::Playing),
            // Hidden until the first touch reveals it (see `TouchSeen`), so a PC
            // session never shows the pad.
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Left thrust zone, labelled.
            parent
                .spawn((
                    Name::new("Thrust Zone"),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        width: Val::Percent(THRUST_ZONE_FRAC * 100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexEnd,
                        padding: UiRect::bottom(Val::Px(48.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.4, 0.7, 1.0, 0.05)),
                ))
                .with_children(|zone| {
                    zone.spawn((
                        Text::new("THRUST"),
                        TextFont {
                            font_size: FontSize::Px(20.0),
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.35)),
                    ));
                });
            // Steer stick ring (repositioned each frame by `update_touch_hud`).
            parent.spawn((
                Name::new("Steer Ring"),
                SteerRingUi,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(ring_d),
                    height: Val::Px(ring_d),
                    border: UiRect::all(Val::Px(3.0)),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.22)),
                BackgroundColor(Color::srgba(0.4, 0.7, 1.0, 0.04)),
            ));
            // Steer stick knob (follows the finger while steering).
            parent.spawn((
                Name::new("Steer Knob"),
                SteerKnobUi,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(34.0),
                    height: Val::Px(34.0),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.35)),
                Visibility::Hidden,
            ));
        });
}

/// Position the steer ring at the live floating origin (or a resting spot in the
/// steer zone when idle) and the knob at the finger, hidden when not steering.
fn update_touch_hud(
    ctl: Res<TouchControl>,
    seen: Res<TouchSeen>,
    windows: Query<&Window>,
    mut q_root: Query<&mut Visibility, (With<TouchHud>, Without<SteerKnobUi>)>,
    mut q_ring: Query<&mut Node, (With<SteerRingUi>, Without<SteerKnobUi>)>,
    mut q_knob: Query<(&mut Node, &mut Visibility), (With<SteerKnobUi>, Without<TouchHud>)>,
) {
    // Reveal the whole overlay only once a touch has been seen.
    if let Ok(mut vis) = q_root.single_mut() {
        *vis = if seen.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let ring_center = if ctl.steering {
        ctl.steer_origin
    } else {
        Vec2::new(window.width() * 0.72, window.height() * 0.72)
    };
    if let Ok(mut node) = q_ring.single_mut() {
        node.left = Val::Px(ring_center.x - STEER_RADIUS_PX);
        node.top = Val::Px(ring_center.y - STEER_RADIUS_PX);
    }
    if let Ok((mut node, mut vis)) = q_knob.single_mut() {
        if ctl.steering {
            *vis = Visibility::Inherited;
            node.left = Val::Px(ctl.steer_pos.x - 17.0);
            node.top = Val::Px(ctl.steer_pos.y - 17.0);
        } else {
            *vis = Visibility::Hidden;
        }
    }
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
    wind: Res<Wind>,
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

    // Radial gravity toward the planet centre, plus the tangential wind gust: both
    // share the world-space acceleration channel (wind is one more term, exactly
    // the pattern the task calls for).
    let radial_up = position.0.normalize_or(Vec3::Y);
    gravity.0 = -radial_up * GRAVITY + wind.accel;

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
    q_ship: Query<(&Transform, &LinearVelocity, &Health), With<Ship>>,
) {
    let Ok((transform, velocity, health)) = q_ship.single() else {
        return;
    };
    telemetry.altitude = transform.translation.length() - PLANET_BASE_RADIUS;
    telemetry.speed = velocity.0.length();
    telemetry.fuel = fuel.0;
    telemetry.integrity = if health.max > 0.0 {
        health.current / health.max * 100.0
    } else {
        0.0
    };
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
/// tallest peak. Also the camera's fallback anchor when there is no ship (the
/// menu, or the result screen after a crash), so those screens frame the planet
/// from above instead of parking the camera inside it. (After a soft landing the
/// hull survives into Result, so there the camera follows the parked ship.)
fn ship_start_pos() -> Vec3 {
    Vec3::Y * (PLANET_BASE_RADIUS * (1.0 + TERRAIN_AMPLITUDE as f32) + START_ALTITUDE)
}

/// Drive the chase camera every frame in every state: follow the ship when it
/// exists (in Playing, and on the result screen after a soft landing, where the
/// parked hull is kept), otherwise sit at the spawn vantage. Running in the menu
/// too lets the smoothed camera state settle on the vantage before a run starts,
/// so Playing opens on the ship instead of swooping out from the planet centre.
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

/// A unit direction inside a polar-angle cap around the +Y pole, uniform in
/// azimuth. `min`/`max` bound the polar angle (radians) from +Y.
fn random_cap_dir(rng: &mut impl Rng, min: f32, max: f32) -> Vec3 {
    let theta = rng.random_range(min..max);
    let phi = rng.random_range(0.0..std::f32::consts::TAU);
    Vec3::new(
        theta.sin() * phi.cos(),
        theta.cos(),
        theta.sin() * phi.sin(),
    )
    .normalize()
}

/// A random fuel-can position: somewhere in the descent cap above the planet,
/// between just above the surface and the start altitude, so it is reachable.
fn random_fuel_can_pos(rng: &mut impl Rng) -> Vec3 {
    let alt = rng.random_range(4.0..START_ALTITUDE);
    let radius = PLANET_BASE_RADIUS * (1.0 + TERRAIN_AMPLITUDE as f32) + alt;
    random_cap_dir(rng, 0.0, FUEL_CAN_SPREAD) * radius
}

/// Spawn one fuel can at `pos`, scoped to the current run.
fn spawn_fuel_can(commands: &mut Commands, assets: &FuelCanAssets, pos: Vec3) {
    commands.spawn((
        Name::new("Fuel Can"),
        FuelCan,
        DespawnOnExit(GameState::Playing),
        Mesh3d(assets.mesh.clone()),
        MeshMaterial3d(assets.material.clone()),
        Transform::from_translation(pos),
    ));
}

/// Keep roughly `FUEL_CAN_TARGET` cans floating: while at target the refill
/// timer stays primed; once a can is collected the timer counts down and spawns
/// one replacement at a random spot every `FUEL_CAN_SPAWN_INTERVAL` seconds, so
/// the field refills over time rather than instantly or emptying out.
fn maintain_fuel_cans(
    mut commands: Commands,
    time: Res<Time>,
    mut spawner: ResMut<FuelSpawner>,
    assets: Res<FuelCanAssets>,
    q_cans: Query<(), With<FuelCan>>,
) {
    if q_cans.iter().count() >= FUEL_CAN_TARGET {
        // At target: hold the timer primed so the next drop waits a full interval.
        spawner.timer = FUEL_CAN_SPAWN_INTERVAL;
        return;
    }
    spawner.timer -= time.delta_secs();
    if spawner.timer > 0.0 {
        return;
    }
    let mut rng = rand::rng();
    spawn_fuel_can(&mut commands, &assets, random_fuel_can_pos(&mut rng));
    spawner.timer = FUEL_CAN_SPAWN_INTERVAL;
}

/// Restore fuel from a collected can, capped at the starting tank so the `%`
/// gauge never exceeds 100. Pure, unit-tested below.
fn add_fuel(current: f32, amount: f32) -> f32 {
    (current + amount).min(START_FUEL)
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
        fuel.0 = add_fuel(fuel.0, FUEL_CAN_AMOUNT);
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

/// Hover the diegetic guide arrow above the ship and point it along the ship's
/// ground track toward the pad, so the player always knows which way to steer.
fn update_guide_arrow(
    pad: Res<LandingPad>,
    q_ship: Query<&Transform, (With<Ship>, Without<GuideArrow>)>,
    mut q_arrow: Query<&mut Transform, (With<GuideArrow>, Without<Ship>)>,
) {
    let Ok(ship) = q_ship.single() else {
        return;
    };
    let Ok(mut arrow) = q_arrow.single_mut() else {
        return;
    };
    let up = ship.translation.normalize_or(Vec3::Y);
    // Horizontal (tangent-plane) direction from the ship toward the pad; when the
    // ship is directly over the pad this collapses to `up` (arrow points up).
    let to_pad = pad.pos - ship.translation;
    let tangent = (to_pad - up * to_pad.dot(up)).normalize_or(up);
    arrow.translation = ship.translation + up * GUIDE_ARROW_HEIGHT;
    // The cone's axis is +Y; aim it along the tangent so it points to the pad.
    arrow.rotation = Quat::from_rotation_arc(Vec3::Y, tangent);
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

fn resolve_collisions(
    mut collisions: MessageReader<CollisionStart>,
    q_ship: Query<(Entity, &Transform, &ApproachSpeed), With<Ship>>,
    q_planet: Query<(), With<Planet>>,
    q_obstacle: Query<(), With<Obstacle>>,
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
    // Once a run has resolved (landed, crashed or destroyed), ignore any trailing
    // contact events from the same frame.
    if outcome.0.is_some() {
        return;
    }

    // Classify the ship's contacts this frame: hitting the planet surface is a
    // touchdown (landing eval), hitting a rock monolith is a structural impact
    // (integrity damage). Asteroids carry no collider -- they are handled by the
    // proximity check in `resolve_asteroid_hits`.
    let mut planet_touch = false;
    let mut obstacle_touch = false;
    for c in collisions.read() {
        let other = if c.collider1 == ship {
            c.collider2
        } else if c.collider2 == ship {
            c.collider1
        } else {
            continue;
        };
        if q_planet.contains(other) {
            planet_touch = true;
        } else if q_obstacle.contains(other) {
            obstacle_touch = true;
        }
    }

    // Use the pre-impact speed, not the live velocity: by now the solver has
    // already absorbed the collision, so the live value under-reports the hit.
    let speed = approach.0;
    let up = transform.translation.normalize_or(Vec3::Y);
    let tilt = (transform.rotation * Vec3::Y).angle_between(up);

    if planet_touch {
        // Great-circle surface distance from the touchdown point to the pad.
        let pad_dist = up.angle_between(pad.dir) * PLANET_BASE_RADIUS;

        // Kick up dust at the contact patch (just under the hull) either way.
        spawn_dust(
            &mut commands,
            &mut meshes,
            &mut materials,
            transform.translation - up * 0.8,
            up,
        );

        if speed <= LAND_SPEED_MAX && tilt <= LAND_TILT_MAX {
            // A touchdown resolves the run here; if an asteroid graze in the same
            // frame also triggered lethal damage, this LANDED outcome is set
            // first and the deferred `on_ship_destroyed` is guarded out. Landing
            // takes precedence over a simultaneous graze -- rare, and the kinder
            // resolution.
            let score = landing_score(fuel.0, speed, tilt, pad_dist, timer.0);
            outcome.0 = Some(Landing {
                landed: true,
                destroyed: false,
                score,
                speed,
                tilt,
            });
            commands.play_sfx(sfx.land.clone());
            shake.trauma = (shake.trauma + LAND_TRAUMA).min(1.0);
            // Freeze the hull where it touched down so it stays visibly parked on
            // the pad through the result screen (it has no DespawnOnExit, so it
            // survives the state change; `cleanup_run_scene` clears it on leaving
            // Result). Static + zeroed velocity stops any post-contact drift.
            commands.entity(ship).insert((
                RigidBody::Static,
                LinearVelocity::default(),
                AngularVelocity::default(),
            ));
        } else {
            outcome.0 = Some(Landing {
                landed: false,
                destroyed: false,
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
    } else if obstacle_touch {
        // Solid rock: chip (or empty) integrity by impact speed. The run only
        // ends if this zeroes the pool, which the `on_ship_destroyed` observer
        // turns into a crash; a survivable graze just costs integrity and shakes.
        commands.trigger(HealthApplyDamage {
            entity: ship,
            source: None,
            amount: impact_damage(speed),
        });
        commands.trigger(PlaySfx::new(sfx.crash.clone()).with_volume(0.6));
        shake.trauma = (shake.trauma + LAND_TRAUMA).min(1.0);
        spawn_dust(
            &mut commands,
            &mut meshes,
            &mut materials,
            transform.translation - up * 0.8,
            up,
        );
    }
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

/// Integrity lost from an obstacle or asteroid contact at `speed` m/s: linear in
/// impact speed with a floor, so a slow graze still stings and a fast smash can
/// empty the pool (`SHIP_MAX_INTEGRITY`) in one hit. Pure, unit-tested below.
fn impact_damage(speed: f32) -> f32 {
    (speed * INTEGRITY_DAMAGE_PER_SPEED).max(INTEGRITY_MIN_DAMAGE)
}

// --- Playing: hazards ------------------------------------------------------

/// Copy each asteroid's `RandomSphereOrbit` position onto its `Transform`, after
/// the orbit driver has advanced it. Same handoff as `07_orbit`'s wanderers;
/// scale is left untouched (set once at spawn).
fn apply_asteroid_transforms(
    mut q_asteroids: Query<(&RandomSphereOrbitOutput, &mut Transform), With<Asteroid>>,
) {
    for (output, mut transform) in q_asteroids.iter_mut() {
        transform.translation = **output;
    }
}

/// Damage the ship and shatter any asteroid it grazes. Proximity check (not a
/// physics collider), matching `07_orbit` and sidestepping the tunneling risk of
/// teleporting a kinematic body along an orbit each frame. The graze routes
/// through `HealthApplyDamage`; a fatal blow is turned into a crash by
/// `on_ship_destroyed`.
fn resolve_asteroid_hits(
    q_ship: Query<(Entity, &Transform, &ApproachSpeed), With<Ship>>,
    q_asteroids: Query<(Entity, &Transform), With<Asteroid>>,
    outcome: Res<Outcome>,
    sfx: Res<SfxAssets>,
    mut shake: ResMut<CameraShake>,
    mut commands: Commands,
) {
    if outcome.0.is_some() {
        return;
    }
    let Ok((ship, ship_transform, approach)) = q_ship.single() else {
        return;
    };
    let hit_dist = SHIP_COLLISION_RADIUS + ASTEROID_RADIUS;
    for (asteroid, transform) in q_asteroids.iter() {
        if ship_transform.translation.distance(transform.translation) > hit_dist {
            continue;
        }
        // Integrity damage scaled by the ship's speed (asteroids drift slowly, so
        // the ship's speed is a good stand-in for the relative impact speed).
        commands.trigger(HealthApplyDamage {
            entity: ship,
            source: Some(asteroid),
            amount: impact_damage(approach.0),
        });
        // Shatter the asteroid so it cannot hit again: drop the marker (stops
        // repeat hits and the transform sync), slice it into debris via
        // `mesh/explode`, and hide + auto-despawn the spent shell.
        commands.entity(asteroid).remove::<Asteroid>().insert((
            ExplodeMesh {
                fragment_count: ASTEROID_FRAGMENTS,
            },
            Visibility::Hidden,
            TempEntity(0.2),
        ));
        commands.trigger(PlaySfx::new(sfx.crash.clone()).with_volume(0.7));
        shake.trauma = (shake.trauma + LAND_TRAUMA).min(1.0);
    }
}

/// Evolve the tangential wind: a slowly rotating bearing and a smooth gust
/// envelope, both driven off one phase so the wind is readable and counterable.
/// Caches the world-space acceleration for `apply_ship_forces`, the streak
/// telegraph and the HUD gauge.
fn update_wind(time: Res<Time>, mut wind: ResMut<Wind>, q_ship: Query<&Position, With<Ship>>) {
    wind.phase += time.delta_secs();
    let radial_up = q_ship
        .single()
        .map(|p| p.0.normalize_or(Vec3::Y))
        .unwrap_or(Vec3::Y);
    // A tangent basis at the ship; the wind bearing rotates within it. Near the
    // pole `radial_up` is ~+Y, so fall back to a different reference axis.
    let mut east = radial_up.cross(Vec3::Y);
    if east.length_squared() < 1e-4 {
        east = radial_up.cross(Vec3::X);
    }
    let east = east.normalize_or(Vec3::X);
    let north = radial_up.cross(east).normalize_or(Vec3::Z);
    let bearing = wind.phase * WIND_TURN_SPEED;
    let dir = east * bearing.cos() + north * bearing.sin();
    // Gust envelope in 0..1, smooth so gusts build and ease predictably.
    let gust = 0.5 + 0.5 * (wind.phase * WIND_GUST_FREQ).sin();
    wind.accel = dir * (WIND_PEAK_ACCEL * HAZARD_DIFFICULTY * gust);
}

/// Blow translucent streak particles downwind past the ship so the wind's
/// direction and strength are visible. Denser when the gust is stronger. Reuses
/// the `FragmentMotion` integrator and `helpers/temp`, like the dust.
fn spawn_wind_streaks(
    time: Res<Time>,
    mut wind: ResMut<Wind>,
    hazards: Res<HazardAssets>,
    q_ship: Query<&Transform, With<Ship>>,
    mut commands: Commands,
) {
    let peak = WIND_PEAK_ACCEL * HAZARD_DIFFICULTY;
    let strength = wind.accel.length();
    if strength < peak * 0.08 {
        return; // near-calm: no streaks
    }
    wind.streak_timer -= time.delta_secs();
    if wind.streak_timer > 0.0 {
        return;
    }
    // Stronger gust -> shorter interval (denser streaks).
    let frac = (strength / peak).clamp(0.1, 1.0);
    wind.streak_timer = WIND_STREAK_INTERVAL / frac;
    let Ok(ship) = q_ship.single() else {
        return;
    };
    let dir = wind.accel.normalize_or(Vec3::X);
    // Spawn upwind of the ship, jittered across the wind and in altitude, so the
    // streak drifts visibly past the hull.
    let mut rng = rand::rng();
    let across = dir
        .cross(ship.translation.normalize_or(Vec3::Y))
        .normalize_or(Vec3::X);
    let up = ship.translation.normalize_or(Vec3::Y);
    let offset = across * rng.random_range(-3.0..3.0) + up * rng.random_range(-2.0..3.0);
    let pos = ship.translation - dir * 4.0 + offset;
    commands.spawn((
        Name::new("Wind Streak"),
        Mesh3d(hazards.streak_mesh.clone()),
        MeshMaterial3d(hazards.streak_material.clone()),
        Transform::from_translation(pos).with_rotation(Quat::from_rotation_arc(Vec3::Z, dir)),
        FragmentMotion {
            velocity: dir * WIND_STREAK_SPEED,
            spin: Vec3::ZERO,
        },
        TempEntity(WIND_STREAK_LIFETIME),
    ));
}

/// End the run when the ship's integrity Health hits zero: a structural failure
/// crash. Explodes the hull and routes to the result screen, distinct from a
/// hard terrain impact so the result screen can name the cause.
fn on_ship_destroyed(
    add: On<Add, HealthZeroMarker>,
    q_ship: Query<(Entity, &Transform, &ApproachSpeed), With<Ship>>,
    state: Res<State<GameState>>,
    sfx: Res<SfxAssets>,
    mut shake: ResMut<CameraShake>,
    mut outcome: ResMut<Outcome>,
    mut next: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    if *state.get() != GameState::Playing {
        return;
    }
    let Ok((ship, transform, approach)) = q_ship.single() else {
        return;
    };
    if add.entity != ship || outcome.0.is_some() {
        return;
    }
    let up = transform.translation.normalize_or(Vec3::Y);
    let tilt = (transform.rotation * Vec3::Y).angle_between(up);
    outcome.0 = Some(Landing {
        landed: false,
        destroyed: true,
        score: 0,
        speed: approach.0,
        tilt,
    });
    commands.entity(ship).insert((
        ExplodeMesh {
            fragment_count: FRAGMENT_COUNT,
        },
        DespawnOnExit(GameState::Playing),
    ));
    commands.play_sfx(sfx.crash.clone());
    shake.trauma = (shake.trauma + CRASH_TRAUMA).min(1.0);
    next.set(GameState::Result);
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

/// Despawn the parked hull (from a soft landing) and the pad beacon when leaving
/// the result screen, before the next run rolls and spawns fresh ones.
fn cleanup_run_scene(mut commands: Commands, q_scene: Query<Entity, Or<(With<Ship>, With<Pad>)>>) {
    for entity in q_scene.iter() {
        commands.entity(entity).despawn();
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
            ..
        }) => (
            "LANDED!".to_string(),
            Color::srgb(0.5, 1.0, 0.6),
            format!(
                "Score {score}\nTouchdown {:.1} m/s at {:.0} deg tilt",
                speed,
                tilt.to_degrees()
            ),
        ),
        Some(Landing {
            destroyed: true, ..
        }) => (
            "DESTROYED".to_string(),
            Color::srgb(1.0, 0.4, 0.4),
            "Hull integrity depleted\nby asteroid and obstacle strikes".to_string(),
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

fn result_input(
    keys: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut next: ResMut<NextState<GameState>>,
) {
    // Space or a tap retries; Esc returns to the menu (keyboard only, since a tap
    // is reserved for the common "retry" action on a phone).
    if keys.just_pressed(KeyCode::Space) || touches.any_just_pressed() {
        next.set(GameState::Playing);
    } else if keys.just_pressed(KeyCode::Escape) {
        next.set(GameState::Menu);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A perfect touchdown: full tank, dead stop, upright, dead centre on the
    /// pad, instant. It beats an identical flight that lands one reward-radius
    /// away and at par time by exactly the proximity + time bonus it forgoes.
    #[test]
    fn landing_score_bullseye_beats_a_far_slow_landing() {
        let bullseye = landing_score(START_FUEL, 0.0, 0.0, 0.0, 0.0);
        let far_slow = landing_score(START_FUEL, 0.0, 0.0, PAD_REWARD_RADIUS, PAR_TIME);
        assert!(
            bullseye > far_slow,
            "a bullseye ({bullseye}) should beat a far, slow landing ({far_slow})"
        );
        let expected_gap = (PAD_PROXIMITY_MAX + TIME_BONUS_MAX).round() as i32;
        assert_eq!(bullseye - far_slow, expected_gap);
    }

    /// Collecting a can tops fuel up but never overfills past the starting tank,
    /// so the `%` gauge stays in range.
    #[test]
    fn add_fuel_caps_at_the_starting_tank() {
        assert_eq!(add_fuel(50.0, FUEL_CAN_AMOUNT), 50.0 + FUEL_CAN_AMOUNT);
        assert_eq!(add_fuel(START_FUEL - 5.0, FUEL_CAN_AMOUNT), START_FUEL);
        assert_eq!(add_fuel(START_FUEL, FUEL_CAN_AMOUNT), START_FUEL);
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

    /// Randomly generated fuel cans are always finite, sit above the surface
    /// (never buried), and stay within the reachable descent cap above the pole.
    #[test]
    fn random_fuel_cans_are_reachable_and_above_the_surface() {
        let mut rng = rand::rng();
        let surface = PLANET_BASE_RADIUS * (1.0 + TERRAIN_AMPLITUDE as f32);
        for _ in 0..200 {
            let pos = random_fuel_can_pos(&mut rng);
            assert!(pos.is_finite());
            assert!(
                pos.length() > surface,
                "fuel can at r={} should clear the surface r={surface}",
                pos.length()
            );
            // Inside the descent cap: polar angle from +Y within the spread.
            let polar = pos.normalize().angle_between(Vec3::Y);
            assert!(
                polar <= FUEL_CAN_SPREAD + 1e-4,
                "fuel can polar angle {polar} should be within the spread"
            );
        }
    }

    /// The pad is rolled inside the reachable band around the pole, always a
    /// unit direction and never at the exact spawn point or the antipode.
    #[test]
    fn random_pad_dir_stays_in_the_reachable_band() {
        let mut rng = rand::rng();
        for _ in 0..200 {
            let dir = random_cap_dir(&mut rng, PAD_ANGLE_MIN, PAD_ANGLE_MAX);
            assert!((dir.length() - 1.0).abs() < 1e-4, "pad dir must be unit");
            let polar = dir.angle_between(Vec3::Y);
            assert!(
                (PAD_ANGLE_MIN - 1e-4..=PAD_ANGLE_MAX + 1e-4).contains(&polar),
                "pad polar angle {polar} should be in [{PAD_ANGLE_MIN}, {PAD_ANGLE_MAX}]"
            );
        }
    }

    /// The touch steer stick is a self-centring, deflection-to-position mapping:
    /// inside the dead zone it is level, at/over the radius it is full `MAX_LEAN`,
    /// and it never exceeds `MAX_LEAN` on either axis.
    #[test]
    fn touch_lean_maps_deflection_to_clamped_target() {
        let r = STEER_RADIUS_PX;
        let d = STEER_DEAD_PX;

        // Dead zone -> level.
        assert_eq!(touch_lean(Vec2::new(d * 0.5, 0.0), r, d), Vec2::ZERO);
        assert_eq!(touch_lean(Vec2::ZERO, r, d), Vec2::ZERO);

        // Full deflection right (+x) rolls right like `D` (negative roll), at
        // exactly MAX_LEAN; a drag past the radius stays clamped.
        let right = touch_lean(Vec2::new(r, 0.0), r, d);
        assert!((right.x - -MAX_LEAN).abs() < 1e-4, "{right:?}");
        assert!(right.y.abs() < 1e-4);
        let past = touch_lean(Vec2::new(r * 3.0, 0.0), r, d);
        assert!((past.x - -MAX_LEAN).abs() < 1e-4, "clamped: {past:?}");

        // Full deflection down (+y, UI y grows downward) pitches back like `S`
        // (positive pitch).
        let down = touch_lean(Vec2::new(0.0, r), r, d);
        assert!((down.y - MAX_LEAN).abs() < 1e-4, "{down:?}");

        // Linear ramp: an offset halfway between the dead zone and the radius
        // yields half of MAX_LEAN (catches a broken magnitude formula).
        let mid = touch_lean(Vec2::new(d + (r - d) * 0.5, 0.0), r, d);
        assert!(
            (mid.x - -MAX_LEAN * 0.5).abs() < 1e-4,
            "half deflection: {mid:?}"
        );

        // Never exceeds MAX_LEAN on either axis for any offset.
        for &(x, y) in &[(r, r), (-r, r), (r * 5.0, -r * 5.0), (d + 1.0, d + 1.0)] {
            let lean = touch_lean(Vec2::new(x, y), r, d);
            assert!(lean.x.abs() <= MAX_LEAN + 1e-4 && lean.y.abs() <= MAX_LEAN + 1e-4);
        }
    }

    /// Integrity damage scales with impact speed above a floor: a slow graze
    /// still costs `INTEGRITY_MIN_DAMAGE`, a brisk hit scales linearly, and a
    /// fast enough smash empties the whole pool in one blow.
    #[test]
    fn impact_damage_floors_then_scales_with_speed() {
        // A crawling contact takes the floor, not the (smaller) linear term.
        assert_eq!(impact_damage(0.0), INTEGRITY_MIN_DAMAGE);
        let slow = INTEGRITY_MIN_DAMAGE / INTEGRITY_DAMAGE_PER_SPEED * 0.5;
        assert_eq!(impact_damage(slow), INTEGRITY_MIN_DAMAGE);

        // Above the floor speed it is linear in impact speed.
        let brisk = 6.0;
        assert!((impact_damage(brisk) - brisk * INTEGRITY_DAMAGE_PER_SPEED).abs() < 1e-4);

        // A fast enough hit is lethal in one blow (>= the whole integrity pool).
        let lethal_speed = SHIP_MAX_INTEGRITY / INTEGRITY_DAMAGE_PER_SPEED;
        assert!(impact_damage(lethal_speed) >= SHIP_MAX_INTEGRITY);
    }
}
