# AGENTS.md

Orientation for agent sessions working in this repository. Read this first:
it covers what the crate is for, how the code is organized, the conventions
every module follows, and the commands to build and verify changes.

## What This Crate Is

`bevy_common_systems` is a collection of Bevy utilities with one goal: build
games faster. It collects the gameplay components, systems and plugins that
almost every game needs (cameras, health, orbit motion, procedural meshes,
status bar UI, a modding event system) so a new game project does not have
to rewrite them. The crate doc calls it "fully copy-pastable": each module
is self-contained enough to be lifted into a game on its own, and the crate
as a whole works as a normal dependency.

When you add or change code here, optimize for that goal: small, composable,
game-agnostic building blocks with obvious APIs, not framework machinery.

## Repository Layout

- `src/` - the main library crate, one directory (or file) per concern.
- `bevy_common_systems_macros/` - proc-macro subcrate, re-exported from the
  main crate. Currently provides `#[derive(EventKind)]`.
- `examples/` - runnable examples, numbered `NN_name.rs`. These double as
  the integration tests and the quickstart documentation.
- `tasks/` - tatr task tracker files (`tasks/<id>/TASK.md`), versioned with
  the code. Check here for planned and in-progress work.
- `docs/` - project documentation: decision notes at the top level, retros
  in `docs/retros/`.
- `web/` - a static TypeScript + webpack showcase site that serves the example
  games as WebAssembly builds (trunk). See `web/README.md` and
  `docs/wasm-web-builds.md`.
- `flake.nix`, `rust-toolchain.toml`, `rustfmt.toml`, `.cargo/config.toml` -
  toolchain setup, see Environment below.

## Module Map

- `audio` - `SfxPlugin`: fire-and-forget one-shot sound effects. Game code
  triggers `PlaySfx` (or calls `commands.play_sfx(handle)`) with an
  `AudioSource` handle and the plugin spawns a self-despawning `AudioPlayer`;
  a global `SfxMasterVolume` resource scales every sound. One concern: SFX
  only, not music or a mixer. Demoed by `examples/06_fruitninja`.
- `camera/`
  - `chase` - `ChaseCameraPlugin`: third-person chase camera with offset,
    smoothing and look-ahead. Game code writes `ChaseCameraInput`.
  - `post` - `PostProcessingDefaultPlugin`: cameras marked with
    `PostProcessingCamera` automatically get `Tonemapping::TonyMcMapface`
    and `Bloom::NATURAL`.
  - `skybox` - `SkyboxPlugin` + `SkyboxConfig`: turns a single vertically
    stacked 6-face image into a cubemap and attaches Bevy's `Skybox`.
  - `wasd` - `WASDCameraPlugin`: first-person free camera math (yaw, pitch,
    movement smoothing). Input comes from `WASDCameraInput`; the actual
    key/mouse bindings live in `helpers/wasd`.
- `debug/` (only compiled with the `debug` feature)
  - `wireframe` - `WireframeDebugPlugin`: global wireframe rendering,
    toggled with F11, starts enabled.
  - `inspector` - `InspectorDebugPlugin`: bevy-inspector-egui world
    inspector plus avian physics gizmos and diagnostics UI, also toggled
    with F11, starts enabled.
  - `harness` - env-gated headless verification tooling: `AutopilotPlugin`
    (force-drives a game's `States` machine on a `.hold(state, seconds)`
    timeline with an optional per-frame input closure, then exits via
    `AppExit`; activated by `BCS_AUTOPILOT`) and `ScreenshotPlugin` (forces a
    window size, advances to a state, waits N frames, writes a PNG; activated by
    `BCS_SHOT="WxH"`). Both are inert unless their env var is set, so a game
    adds them permanently. Replaces the throwaway harness the Gotchas used to
    prescribe; demoed by `08_dropzone` and `11_overload`.
- `health` - `HealthPlugin`: `Health` component, `HealthApplyDamage` entity
  event (propagates up the hierarchy), `HealthZeroMarker` inserted when
  health hits zero. Trigger damage with `commands.trigger(...)`.
- `helpers/`
  - `despawn` - `DespawnEntityPlugin`: insert `DespawnEntity` to despawn an
    entity immediately.
  - `temp` - `TempEntityPlugin`: `TempEntity(seconds)` auto-despawns after
    the duration.
  - `wasd` - `WASDCameraControllerPlugin`: binds WASD/mouse/space/shift via
    bevy_enhanced_input and writes `WASDCameraInput` for `camera/wasd`.
- `mesh/`
  - `builder` - `TriangleMeshBuilder`: procedural triangle meshes -
    octahedron spheres, face subdivision, noise displacement, plane
    slicing, normals/UVs, conversion to and from `Mesh`.
  - `explode` - `ExplodeMeshPlugin`: insert `ExplodeMesh` to slice an
    entity's mesh (and its children) into `ExplodeFragments` for
    destruction effects (demoed by `examples/05_explode`).
- `meth/` - math helpers (the name is an intentional pun, do not "fix" it)
  - `lerp` - `LerpSnap` trait: exponential lerp with snap-to-target for
    `f32` and `Vec3`; used by the chase camera and the sphere orbit
    systems for smoothing.
  - `sphere` - spherical coordinate conversions and `slerp`.
- `modding/`
  - `events` - a generic, serde-friendly event bus aimed at modding and
    scripting: `EventWorld` (sync game state to/from a resource),
    `EventKind` (named event types), `EventHandler` entities holding
    filters and actions, `GameEventsPlugin<W>`, and a `Commands::fire`
    extension. Event payloads travel as `serde_json::Value`, so events can
    cross a modding or scripting boundary; filters and actions themselves
    are Rust trait objects.
  - `registry` - `EventHandlerRegistry<W>`: maps event / filter / action name
    strings to registered constructors so `EventHandler`s can be authored in
    JSON (`HandlerSpec`) and built at runtime, the data-driven counterpart to
    the Rust `EventHandler` builder. Demoed by `examples/03_modding`.
- `physics/`
  - `pd_controller` - `PDControllerPlugin`: computes the PD torque needed
    to rotate an avian3d rigid body (the `PDControllerTarget` entity)
    toward the `PDControllerInput` rotation; game code applies the
    resulting `PDControllerOutput`.
  - `doom_controller` - `DoomControllerPlugin`: a Doom-style (simple,
    arena-shooter) first-person character controller. `DoomController` config +
    `DoomControllerInput` (look delta + strafe/forward intent) + public
    `DoomControllerState` (yaw/pitch, settable to aim) + `DoomControllerOutput`
    (a planar velocity). It integrates the look (clamping pitch), orients a
    `DoomEye`-marked camera child, and outputs a velocity the game writes into
    the body's `LinearVelocity` (leaving `.y` to gravity, so avian does
    collide-and-slide). Output-only, so it takes NO avian dependency despite
    living here. Named `Doom` on purpose -- it reserves the premium
    `FirstPersonController` name for a future, more capable controller. Requires
    an axis-locked body (`LockedAxes::ROTATION_LOCKED`) + a `DoomEye` child.
    Harvested from `14_breach` (`docs/2026-07-05-fps-controller-harvest.md`).
- `transform/` - motion driver components; each computes an Output that
  your systems apply or read:
  - `sphere_orbit` - orbit a sphere surface from explicit theta/phi input.
  - `directional_sphere_orbit` - orbit toward a direction vector.
  - `random_sphere_orbit` - wander randomly on a sphere surface.
  - `point_rotation` - accumulate a rotation from input deltas (mouse).
  - `smooth_look_rotation` - rotate around an axis toward a target angle
    with speed and optional min/max limits.
- `ui/`
  - `status` - `StatusBarPlugin` plus `status_bar()` / `status_bar_item()`
    bundle builders: a screen-corner metrics overlay (FPS, version, custom
    values) driven by `value_fn` / `color_fn` closures. The value closures
    run inside an exclusive system every frame (they get `&World` and
    block parallelism), so keep them cheap.
  - `animate` - `UiAnimatePlugin` plus opt-in marker components that copy a
    `Tween` into a plain UI field each frame (the UI-node counterpart to the
    material-only `feedback/flash`): `TweenNodeOffset` (`Tween<Vec2>` ->
    `Node.left/top` px), `TweenNodeScale` (`Tween<f32>` -> `Node.width/height`
    percent), `TweenNodeBackground` (`Tween<Vec4>` -> `BackgroundColor`), with
    `color_to_vec4`/`vec4_to_color` helpers and a `node_flash()` constructor.
    Builds on `tween`; harvested from `13_glide` (see
    `docs/2026-07-05-13glide-ui-juice-harvest.md`, which also records why the
    rolling-number readout stayed game-local).

## Conventions

The modules are deliberately uniform. Follow these patterns when adding or
changing code; consistency is the crate's main defense against bloat.

- One concern per module. Modules that add runtime behavior ship one
  plugin per concern, named `*Plugin`; pure utility modules (meth,
  mesh/builder) export plain types and functions instead. Systems that
  need ordering hooks get a public `SystemSet` enum named `*Systems`
  (a few use `*PluginSystems`).
  - Ordering caveat: `.before(SetX)` / `.after(SetX)` orders a system only
    relative to `SetX`'s *current members*. If `SetX` can be empty (e.g. it
    belongs to a sibling plugin that may not be added), those edges vanish and
    give no guarantee. When system A must run before system B regardless, pin
    it with a direct edge -- `configure_sets(schedule, B.after(A))` or
    `chain()` -- never by ordering both against a third set that might be
    empty. This bit `camera/shake`: Restore/Apply were ordered only around
    `ChaseCameraSystems::Sync`, which is empty for static-camera games, so the
    two were unordered and could drift; the passing test masked it because the
    executor resolved the ambiguity in insertion order
    (`docs/retros/20260704-134500-camera-shake-module.md`).
- Config / Input / Output / State component split:
  - a public config component named after the feature (`WASDCamera`,
    `ChaseCamera`, `PDController`, `SphereOrbit`, ...);
  - a public `*Input` component that game code writes each frame;
  - a public `*Output` component (or direct `Transform` writes) that game
    code reads;
  - private `*State` components the plugin manages internally - keep them
    out of the prelude.
  - Ordering contract: a module whose `*Plugin` reads its `*Input` (or writes its
    `*Output`) inside a named `*Systems` set MUST document, on that set's doc, the
    contract for consumers -- write `*Input` `.before(TheSet)` and read `*Output`
    `.after(TheSet)`. Without it every consumer races the set; `physics/doom_controller`
    shipped its first consumer (`14_breach`) with the input feed *unordered* vs its
    `Drive` set (a one-frame input lag) precisely because the write-before edge was
    undocumented, and the autopilot could not catch it -- it force-writes the state,
    bypassing the input path (`docs/retros/20260705-132542-doom-controller-harvest.md`).
- Modules expose their public API through preludes: most files define
  `pub mod prelude`, parent modules aggregate child preludes (a few leaf
  files are re-exported directly by their parent), and `crate::prelude`
  aggregates everything. Users import
  `use bevy_common_systems::prelude::*;`.
- Reactive setup via observers: plugins register `add_observer` for
  `On<Add, X>` / `On<Insert, X>` to attach internal components, instead of
  startup systems. `#[require(...)]` is used where a component only makes
  sense with companions.
- Derive `Reflect` on components (plus `Deref`/`DerefMut` for newtypes).
- Most files carry a module-level `//!` doc comment with a short usage
  snippet, and public items carry doc comments; match that standard in
  new code even though a few existing files fall short.
- Logging: `debug!("XPlugin: build")` in `Plugin::build`, `trace!` in
  systems and observers.
- Formatting is enforced by rustfmt (`imports_granularity = "Crate"`,
  `group_imports = "StdExternalCrate"`); run `cargo fmt` and let it manage
  import blocks. Clippy runs with `type_complexity` and
  `too_many_arguments` allowed crate-wide (see `Cargo.toml [lints.clippy]`).
- Writing style everywhere (code, comments, docs, commits): plain ASCII.
  No em dashes, smart quotes, ellipsis characters or arrows; use `-`,
  `--`, `...`, `->`.

## Features and Dependencies

Features (default: none):

- `debug` - compiles the `debug` module; enables `avian3d/diagnostic_ui`,
  `bevy/track_location` and `bevy-inspector-egui`.
- `dev` - alias that just enables `debug`.

Key dependencies and why they are here:

- `bevy` 0.18 - the engine; this crate tracks current Bevy APIs
  (observers, `EntityEvent`, required components).
- `avian3d` 0.6 - 3D physics; used by the PD controller, the debug
  diagnostics and the examples.
- `bevy_enhanced_input` 0.26 - input contexts/actions for the WASD
  controller in `helpers/wasd`.
- `noise` 0.9 - noise functions consumed by `TriangleMeshBuilder`.
- `rand` 0.9 - randomness for orbits and mesh explosion.
- `serde` / `serde_json` - event payloads in `modding`.
- `bevy-inspector-egui` 0.36 (optional) - the `debug` inspector.
- dev-dependency `clap` 4.5 - every example is a small CLI.

## Environment and Toolchain

- Rust nightly, pinned by `rust-toolchain.toml` (with rustfmt and clippy).
- Nix flake dev shell: `nix develop` provides the nightly toolchain, wasm
  tooling (trunk, wasm-pack, wasm32-unknown-unknown target) and the native
  libraries Bevy needs on Linux (vulkan, wayland/x11, alsa, udev). On
  NixOS, graphical examples generally only run inside this shell because
  it sets `LD_LIBRARY_PATH`.
- `.cargo/config.toml` sets `--cfg=web_sys_unstable_apis` for wasm builds.
- Edition 2021. License: MIT.

## Build, Verify, Run

Verified working as of 2026-07-03:

```
cargo build                                  # library + macros subcrate
cargo fmt --check                            # formatting
cargo clippy --all-targets                   # lints; currently clean, keep it clean
cargo clippy --all-targets --features debug  # lints with the debug module
cargo test                                   # unit tests + doctests
cargo test --features debug                  # same, plus the debug module
cargo test --examples                        # the #[cfg(test)] tests inside examples/
./scripts/check-ascii.sh                     # enforce the plain-ASCII rule
cargo run --example 01_sphere                # opens a window
cargo run --example 01_sphere --features debug   # same, with inspector UI
```

(The only warning surfaced by clippy is a future-incompat note from the
transitive `proc-macro-error2` dependency, not from this crate's code.)

CI (`.github/workflows/ci.yml`) runs this whole suite - fmt, both clippy
configs, both test configs, `cargo test --examples` (the in-example unit
tests, which plain `cargo test` compiles but never runs), and
`scripts/check-ascii.sh` - on every push and pull request. Keep it green; run
the commands above locally before pushing.

Testing convention: pure math and geometry functions get `#[cfg(test)]`
unit tests next to the code (13 today, in `meth/sphere`, `mesh/builder`,
`physics/pd_controller` and `transform/point_rotation`); ECS behavior is
exercised by the examples, which are the de facto integration tests. If
you add a feature, test its pure logic in-module and wire the ECS side
into an existing example or a new numbered one (`examples/NN_name.rs`,
clap CLI header, `DefaultPlugins`, setup system), and make sure everything
compiles via `cargo clippy --all-targets`. If a test comment (or a TASK.md
note) claims the test exercises some behaviour, back that claim with an
assertion in the same edit -- do not write the aspirational comment and test
only the easy half. Reaching a `pub(super)` field or adding a small helper to
actually drive the behaviour is cheaper than the review round that catches the
gap (bit two consecutive cycles: `docs/retros/20260704-165400-overload-example.md`
and `docs/retros/20260703-165439-modding-json-registry.md`).

Examples:

- `01_sphere` - octahedron sphere from `TriangleMeshBuilder` + WASD camera.
- `02_planet` - the same mesh displaced with Fbm/Perlin noise: a planet.
- `03_modding` - the `modding` event bus end to end, including
  `#[derive(EventKind)]`; its handlers are authored as an inlined JSON string
  and built through the `EventHandlerRegistry`, and the event logic prints to
  the console.
- `04_status_item` - status bar UI with FPS and custom shell-command items.
- `05_explode` - the mesh slicer end to end: press Space to slice a mesh
  into `ExplodeFragments` that fly apart and auto-despawn.
- `06_fruitninja` - a fruit-ninja style game from plain shapes: boot into a
  main menu, then octahedron fruit arc up from below; hold Left Mouse Button
  and swipe to slice them into exploding fragments for score. A blade trail
  follows the swipe, each slice pops a rising "+N", and slicing several fruit
  in one swipe builds an escalating combo with a "COMBO xN" banner. Dark bombs
  are mixed in -- slicing one deals lethal damage via `HealthPlugin` and ends
  the run at a game-over screen. Uses Bevy states for menu/playing/game-over.
  Every gameplay event plays a one-shot sound via `SfxPlugin`; the files in
  `assets/sounds/` are generated placeholders (`scripts/gen-placeholder-sounds.py`),
  see `assets/sounds/README.md` and
  `docs/2026-07-03-audio-and-fruitninja-sounds.md`.
- `07_orbit` - "Orbit Runner": a surface-dodge game on a sphere. Ride a glowing
  marker around a planet steering with `DirectionalSphereOrbit` (A/D, arrow
  keys, or drag), sweep up wandering orbs and dodge wandering red hazards, both
  driven by `RandomSphereOrbit`. A `ChaseCamera` follows with `LerpSnap`
  smoothing; a hazard deals damage through `HealthPlugin` and zero health ends
  the run. The planet gets more crowded and faster as difficulty levels climb.
  Same shape as `06_fruitninja`: menu/playing/game-over states, `SfxPlugin`
  one-shots (`pickup`/`hurt`/`level_up`, sharing `menu_select`/`game_over`), and
  a wasm/trunk showcase build. Exercises the whole `transform/*` orbit family,
  `camera/chase` and `meth` under gameplay; grows out of `01_sphere`. See
  `docs/2026-07-03-orbit-runner.md`.
- `08_dropzone` - a lunar-lander game and the headline demo of
  `PDControllerPlugin`. A noise-displaced planet (the `02_planet` recipe) sits
  at the origin with radial gravity; you fly a lander down onto it. Space/Up
  thrusts along the ship's local up, W/S and A/D lean the target attitude, and
  the PD controller torques the avian3d rigid body toward that attitude (this
  is the crate's first real physics sim, not just the debug renderer). Touch
  down slow and upright to score; hit too hard or too tilted and the hull

  breaks apart via `mesh/explode`. The descent is also hazardous: rock monoliths
  ring the pad (static colliders forming a gap to thread), asteroids drift the
  corridor on `RandomSphereOrbit`, and a time-varying wind shoves the ship
  sideways; grazing a rock or asteroid drains a ship-integrity `Health` pool
  (`HealthPlugin`) scaled by impact speed, and zero integrity ends the run --
  terrain contact stays instant-crash. All hazards scale off one
  `HAZARD_DIFFICULTY` knob. Pulls in `camera/skybox` (a procedurally generated
  starfield, no asset file), `camera/post` bloom on the thruster flame,
  `camera/chase`, `transform/random_sphere_orbit`, `health`, `ui/status` gauges
  (altitude/speed/fuel/hull/wind) and `audio`. The planet's avian trimesh
  collider is built inline from `TriangleMeshBuilder::vertices_and_indices()`.
  Follows the `06_fruitninja` shape (states, sounds, wasm). See
  `docs/2026-07-03-dropzone-example.md`, `docs/2026-07-04-dropzone-tier-a-fun.md`
  and `docs/2026-07-04-dropzone-hazards.md`; the flight constants were
  play-tested and tuned in `tasks/20260703-213510`.
- `09_reactor` - "Reactor": a rules-as-machine incremental and the headline demo
  of `modding`. The whole simulation runs on the `03_modding` event bus, but the
  player builds the machine at runtime: a `ReactorWorld` (an `EventWorld`) holds
  ENERGY / HEAT / CREDITS, the engine `fire`s a `tick` every half second plus
  `click`/`sell` on the controls, and every rule that reacts is a JSON-authored
  `EventHandler` entity built through the `EventHandlerRegistry`. Built-in Manual
  Tap and Sell handlers ship with the reactor; every shop part you buy spawns
  another handler (fuel rods add energy AND heat, sinks/pumps/turbines shed heat,
  market uplinks sell energy for credits). Compose them into an escalating loop,
  but the grid heats up as you climb credit tiers, so HEAT hitting 100 is a
  meltdown (game over); score is total credits earned. The shop palette is a
  `HandlerSpec` list and buying is a `build_handler` call, so the gameplay and
  the modding data are the same thing. Also exercises `ui/status` (a compact
  telemetry HUD reading the `EventWorld`) and `SfxPlugin` one-shots (reuses
  `pickup`/`golden`/`alarm`/`level_up` plus shared `menu_select`/`game_over`,
  no new sound files). No 3D scene -- renders with a plain `Camera2d`. Follows
  the `06_fruitninja` shape (states, sounds, wasm). Grows out of `03_modding`.
  See `docs/2026-07-04-reactor-example.md`.

- `10_asteroids` - a top-down "asteroids" shooter and the crate's physics-
  fragments showcase, the counterpoint to `06_fruitninja`. You fly a ship around
  a bounded, zero-gravity arena and shoot drifting octahedron rocks; a hit
  inserts `ExplodeMesh` and the `on_fragments_spawned` observer respawns every
  sliced shard as a real `RigidBody::Dynamic` avian body that keeps drifting,
  bounces off the arena walls, and is a new smaller hazard -- unlike `06`, where
  fragments are hand-integrated throwaways. Rocks split large -> medium -> small
  (a `generation` cap keeps the field clearable); clear a wave to face a busier
  one. Bumping a rock costs a hull point via `HealthPlugin` (with i-frames);
  `camera/post` blooms the glowing bullets and thruster flame. Exercises a broad
  slice of avian3d not seen elsewhere (kinematic ship, sensor bullets,
  `CollisionLayers`, `Restitution`, `LockedAxes`, `CollisionStart` messages,
  `Gravity::ZERO`). Controls are unified keyboard + pointer (A/D rotate, W
  thrust, Space fire, or hold the mouse / a finger to fly toward it and
  auto-fire) so the wasm build is touch-playable. Follows the `06_fruitninja`
  shape (states, sounds, wasm). See `docs/2026-07-04-asteroids-example.md`.
- `11_overload` - "Overload": a dashboard-survival game and the headline demo of
  `ui/status` as a game surface. The whole game lives on the `status_bar`: four
  gauges (HEAT/PRES/FLUX/CHRG) climb and random-walk on their own, each a
  `status_bar_item` whose `color_fn` goes green -> amber -> red; press 1/2/3/4 to
  vent one back down, but each vent pushes a coupled neighbour up, so it is a
  juggling act. While any gauge sits red the reactor's `Health` drains
  (`HealthApplyDamage` -> `HealthZeroMarker` ends the run) and an alarm beeps;
  difficulty ramps the climb rates over time. No 3D scene -- renders with a plain
  `Camera2d`. Touch-playable like `08_dropzone`: an on-screen vent pad (a bottom
  strip of four buttons, revealed on first touch) lets a phone tap-vent, and the
  menu/meltdown screens take a tap; touch is an additive writer of the same
  `apply_vent` path, keyboard unchanged. Grows out of `04_status_item`, follows
  the `06_fruitninja` shape (states, sounds, wasm). See
  `docs/2026-07-04-overload-example.md` and
  `docs/2026-07-04-overload-touch-controls.md`.
- `12_bastion` - "Bastion": a defend-the-core tower defense and the headline demo
  of `camera/project` plus the two aim/track halves of the `transform` family
  that no other example showed. A glowing `Core` (a `Health` pool) sits at the
  arena center; enemies spawn on the border and converge inward from every
  bearing, and one that reaches the Core damages it (zero health ends the run).
  Kills earn credits, spent to place and upgrade towers as waves ramp.
  `camera/project`'s `pointer_on_plane` maps the tap/pointer to the ground point
  where a tower is placed (ghost + range ring) and `world_to_screen` anchors the
  "+N" credit popups; `transform/point_rotation` drives an orbit camera (a pivot
  at the Core the player yaws/pitches by dragging, pitch clamped in-game);
  `transform/smooth_look_rotation` drives each tower turret, rate-limited so a
  fast enemy can out-slew a cheap turret until upgraded. Tower/enemy stats are
  data-driven: loaded from `assets/bastion/catalog.json` at startup (native reads
  the file so stats/new types need no recompile, wasm uses a compiled-in copy),
  with the build key bindings and weighted enemy spawn iterating the catalog so a
  new tower/enemy is added purely in JSON (`tasks/20260704-220719`,
  `docs/2026-07-05-bastion-data-catalog.md`; the spike concluded the loader -- not
  a `SpecCatalog<T>` type -- is the only reusable nugget and should wait for a
  second user). Reuses `mesh/explode` (own `On<Insert,
  ExplodeFragments>` observer -- the crate has no fragment observer), `ui/popup`,
  `ui/status`, `camera/shake`, `feedback`, `scoring/streak`, `time/cooldown`,
  `helpers/temp`, `input/pointer` and `audio`. One pointer does double duty (drag
  = orbit, tap = place/select, disambiguated by a move threshold); Space is a
  keyboard/autopilot placement path. Follows the `06_fruitninja` shape (states,
  sounds, wasm). See `docs/2026-07-04-bastion-example.md`.
- `13_glide` - "Glide": a slide-merge (2048-style) number puzzle rendered
  entirely in Bevy UI, and the headline demo of `tween` plus `persist` +
  `scoring/high_score` (the first example for any of the three, and the gallery's
  first puzzle). Swipe or arrow-key input slides a 4x4 board; equal tiles merge
  into their sum, a new tile spawns each move, and a full board with no legal
  move ends the run. The best score is saved across launches via
  `PersistPlugin::<HighScore<u32>>`. The whole board is UI, so every animation
  drives a plain `Node`/`BackgroundColor` field from a `Tween` output (never
  `Transform` scale, which UI layout owns): a tile slide is a `Tween<Vec2>` into
  `Node { left, top }`, a spawn/merge pop a `Tween<f32>` into a `Node`'s
  size-percent, a merge flash a `Tween<Vec4>` into `BackgroundColor`, and the
  score readout rolls on another `Tween<f32>`. Each tile is a positioning wrapper
  (moved by the slide tween) around a face (sized/coloured by the pop and flash
  tweens), so the three animate independently and the pop grows from centre. The
  pure move logic (`resolve_line` / `apply_move` / `is_game_over`) is unit-tested
  off the ECS. The slide/pop/flash appliers are now the crate's `ui/animate`
  markers (`TweenNodeOffset`/`TweenNodeScale`/`TweenNodeBackground` + `node_flash`,
  harvested by `tasks/20260705-090557`); the rolling score readout stayed
  game-local (see `docs/2026-07-05-13glide-ui-juice-harvest.md`). Also reuses
  `ui/popup`, `ui/menu`, `input/pointer`, `input/state` and `audio`; renders with
  a plain `Camera2d`. Follows the `06_fruitninja` shape (states, sounds, wasm).
  Press Space during a run to hand control to a built-in auto-solver: a shallow
  expectimax (`best_move`/`score_grid`, pure and unit-tested off the ECS, with a
  headless test that plays a full game to the 2048 tile) that plays toward 2048
  on its own, paced at one move / 0.32s so each slide is watchable, driving the
  same `start_move` path a human move does. See `docs/2026-07-05-glide-example.md`
  and `docs/2026-07-05-glide-solver.md`.
- `14_breach` - "Breach": a grounded, Doom-like first-person arena shooter, and the
  gallery's first first-person game. It headlines three things no prior example
  showed: the first-person viewpoint as a real game (`camera/wasd` only ever
  appeared in the free-fly tech demos), the crate's first avian `SpatialQuery`
  raycast (the hitscan gun), and (now harvested) the first-person controller.
  Because `camera/wasd` is a free-fly spectator camera (no gravity/ground/collision/
  cursor-grab), the player uses the crate's `physics/doom_controller`: a
  `RigidBody::Dynamic` capsule with `LockedAxes::ROTATION_LOCKED` whose
  `DoomControllerOutput` velocity the game writes into `LinearVelocity`, so avian's
  solver does collide-and-slide against the static level for free (a kinematic body
  is NOT pushed back by statics -- that was the key call). The body stays
  axis-aligned; yaw lives in `DoomControllerState` and a `DoomEye` `Camera3d` child
  at eye height carries the view rotation, with the move intent rotated by the same
  yaw. Look is always-on from `AccumulatedMouseMotion` fed into
  `DoomControllerInput`, with the cursor grabbed via `input/cursor` over the 0.19
  `CursorOptions` component (a per-window component, not `window.cursor`) and a pitch
  clamp. Left-click (a `time/cooldown` gate) fires
  `SpatialQuery::cast_ray` masked to `[Enemy, World]` and excluding the player; the
  first enemy hit takes `HealthApplyDamage` + `feedback/flash` and on death bursts
  into `mesh/explode` physics gibs. Waves of octahedron enemies (`mesh/builder`)
  path toward you (straight-line, so the arena is open -- no interior cover to snag
  the AI) and melee via continuous proximity damage (spikes a `feedback/screen_flash`
  vignette + `camera/shake`); zero health ends the run. Kills chained inside a short
  window build a combo (`scoring/streak`) that multiplies the points each kill is
  worth, floats a "+N" and flashes a "COMBO xN +P" tally (`ui/popup`); the points
  score (not the raw kill count) is the persisted `HighScore`. Slain enemies have a
  chance to drop a glowing pickup (emissive-for-bloom sphere) that the player grabs by
  walking over it: an instant heal (`HealthPlugin`, capped at max) or a timed speed /
  fire-rate buff (the speed buff scales `DoomController.move_speed`, the fire-rate buff
  ticks the `Gun` cooldown faster). Reuses `camera/post`
  (bloom on tracers/enemies/pickups), `helpers/temp`, `ui/status` HUD + crosshair,
  `ui/menu`, `audio`, `persist`+`HighScore`, `ui/touchpad` (dual-stick touch),
  `input/state`. The pure logic (`wave_size`/`ring_positions`, the streak-scaled
  scoring, `apply_pickup`/`decay_buffs`/buff multipliers, and the controller's
  `doom_move_dir`/pitch clamp now in `physics/doom_controller`) is
  unit-tested off the ECS (chained kills multiply, the streak lapses, heal caps at max,
  buffs decay); the
  headless autopilot AIMS at the nearest enemy (an FPS gun can't be verified by
  fire-forward). Follows the `06_fruitninja` shape (states, sounds, wasm); touch is a
  compromise, desktop is primary. See `docs/2026-07-05-breach-example.md` and the
  harvest note `docs/2026-07-05-fps-controller-harvest.md` (`tasks/20260705-103238`).

## Workflow

- Work is tracked with the tatr CLI in `tasks/`; read `tasks/<id>/TASK.md`
  before picking something up, and check open tasks before planning new
  work.
- Feature branches per task, merged into `master` (the default branch)
  after review. Do not push without being asked.
- Document decisions in `docs/`, retrospectives in `docs/retros/`.

## Gotchas

- `meth` is the module's real name (math pun). Leave it.
- `README.md` is a three-line stub and slightly stale; this file is the
  real orientation document.
- The `EventKind` derive's default `Info` path does not resolve (stale
  module path, `tasks/20260703-095509`); always pass `#[event_info(...)]`
  explicitly, as `examples/03_modding.rs` does.
- `helpers/wasd` and `camera/wasd` are two halves of one feature: the
  camera math is input-agnostic on purpose, so games can swap the binding
  layer without touching camera behavior.
- Bevy 0.19 UI/light API when writing a new example: copy the idioms from an
  existing example rather than from memory. `TextFont.font_size` is
  `FontSize::Px(..)` (not a bare `f32`), `TextLayout` is built as a struct
  literal (`TextLayout { justify: Justify::Center, ..default() }`, no
  `new_with_justify`), and `AmbientLight` is a per-camera component, not a
  global resource. Rounded UI corners are a `Node` field, not a component:
  `Node { border_radius: BorderRadius::MAX, .. }` (whereas `BorderColor` *is* a
  component) - spawning `BorderRadius` in the bundle fails the `Bundle` bound.
  Grepping `06_fruitninja`/`08_dropzone` for `font_size:`, `TextLayout`,
  `AmbientLight` and `border_radius:` up front avoids a batch of compile errors
  (this has bitten three cycles now: `docs/retros/20260703-150200-bevy-019-migration.md`,
  `docs/retros/20260703-165432-dropzone-example.md` and
  `docs/retros/20260704-103517-dropzone-touch-controls.md`). The same "copy /
  verify, do not improvise the visual layer" rule extends past compile errors to
  things that render wrong silently (a background run cannot see the screen, so
  they are not panics): a `StandardMaterial` with an HDR `emissive` must NOT set
  `unlit: true` or it will not bloom -- `unlit` makes Bevy skip the lighting pass
  where emissive is applied (`bevy_pbr` `render/pbr.wgsl`); an entity that
  carries mesh children but no mesh of its own needs an explicit `Visibility` for
  the children to render (else `B0004`); and camera shake is an absolute offset
  from a fixed base (`translation = BASE + offset`, per `06_fruitninja`), never
  an accumulating `+=`, or the camera drifts. All three slipped past
  implementation and were caught in review in `10_asteroids`
  (`docs/retros/20260703-170744-asteroids-example.md`).
- Running examples: the examples open a window and are the de facto integration
  tests, so a new or changed example is not "done" until it has actually been
  run once - `cargo build` only proves it compiles, not that it boots. Even a
  headless/background session often has a display (`echo $DISPLAY`); if so, run
  `cargo run --example NN_name` under a `timeout` and confirm it reaches the
  render loop (a `bevy_render::view::window` swap-chain log line means startup
  finished). Not doing this shipped a startup hang in 08_dropzone
  (`docs/retros/20260703-165432-dropzone-example.md`). Booting only reaches the
  menu, though; to exercise a stateful example's actual gameplay
  (menu -> playing -> result) headlessly when no input-injection tool
  (`xdotool`) is around, do NOT hand-roll a throwaway autopilot any more (it was
  re-invented and deleted 7 times). Add the reusable `AutopilotPlugin`
  (`debug/harness`, behind `#[cfg(feature = "debug")]` like `InspectorDebugPlugin`):
  give it a `.hold(state, seconds)` timeline and an optional `.input(|world,
  elapsed| ...)` closure that pokes the game's input, then run
  `BCS_AUTOPILOT=1 cargo run --example NN_name --features debug` under `timeout`
  and confirm the log shows each `autopilot: -> State` transition and the final
  `autopilot: cycle complete, no panic` line. It is inert unless `BCS_AUTOPILOT`
  is set, so it stays in the example permanently -- no add/remove churn. It exits
  via `AppExit::Success`, not `std::process::exit` (the latter segfaults on wgpu
  teardown -- harmless, but do not mistake it for a game crash). See
  `docs/dev-harness.md`; 08_dropzone and 11_overload are wired up as the
  reference. The two harness plugins are mutually exclusive in a single
  run: `AutopilotPlugin` and `ScreenshotPlugin` both drive `NextState`, so
  running with both `BCS_AUTOPILOT` and `BCS_SHOT` set makes the screenshot never
  fire (its settle-frame count never accumulates). To capture a mid-gameplay
  frame, use `ScreenshotPlugin` alone, or drive with `AutopilotPlugin` and grab
  the window externally with `scrot`
  (`docs/retros/20260704-220736-bastion-example.md`). (History, for context: verified 07/08 gameplay via the old
  hand-rolled harness in `tasks/20260703-213510`, `tasks/20260704-103544`.)
- Seeing the screen (do not treat "a background run cannot see the screen" as a
  hard limit): when `$DISPLAY` is set, a background session CAN screenshot the
  running app and verify the visual layer. `scrot` and ImageMagick `import` grab
  the root window; `xdotool` (via `nix run nixpkgs#xdotool` when not on PATH)
  finds/moves the app window so you can crop it precisely (`magick IN -crop
  WxH+X+Y +repage OUT`), and `import`/`Read` the PNG to actually look. To capture
  a specific state or viewport, do NOT hand-roll the window-size + auto-advance
  harness (it was re-invented twice). Add the reusable `ScreenshotPlugin`
  (`debug/harness`, behind `#[cfg(feature = "debug")]`): `ScreenshotPlugin::new(
  TargetState).settle_frames(n).path("shot.png")`, then run
  `BCS_SHOT=390x844 cargo run --example NN_name --features debug` under `timeout`
  -- the `WxH` env value forces the window resolution, the plugin advances to the
  target state, waits `n` settled frames, writes the PNG and exits via
  `AppExit::Success`. It is inert unless `BCS_SHOT` is set, so it stays in the
  example permanently. This caught a real 09_reactor layout regression -- at phone width four
  of six shop buttons rendered below the fold, invisible to `cargo build`,
  clippy and the boot check but obvious in a screenshot
  (`docs/retros/20260704-143000-reactor-overload-mobile-touch.md`). For a
  responsive Bevy 0.19 grid that must hold N columns at any width, use percentage
  item widths, not fixed px + `flex_wrap` (flexbox wraps before it shrinks, so
  fixed-px cards collapse to one column on a narrow frame).
- Verifying builds: never judge a build/command by a piped `| tail`'s exit
  code -- the pipe reports `tail`'s status, so a failed `cargo build | tail`
  looks like it passed. Redirect to a file and check `$?` when pass/fail
  matters; use `| tail` only for interactive peeking.
- A bare `cargo build` compiles the library and bins but NOT the examples, so
  it is a false green for any change that touches example call sites (the
  examples are the integration tests). Use `cargo clippy --all-targets` (or
  `cargo build --examples`) as the real compile gate. Missing this hid eight
  example errors behind a "clean" build in `input/pointer`
  (`docs/retros/20260704-161508-input-pointer.md`).
- Tests that embed a data file (`include_str!`) go stale when the same task edits
  that file. If a `#[cfg(test)]` assertion checks the contents of a JSON/asset the
  task also changes (e.g. an `include_str!`ed catalog), assert the *final shipped*
  state and re-run `cargo test --examples` AFTER the data edit, not just after the
  code edit -- a green example-test run before the data was finalized is a false
  green for anything embedding it. Bit `12_bastion`'s data-driven catalog: a
  roster-count test passed at 2+2 then failed once the "prove it" step made the
  embedded catalog 3+3 (`docs/retros/20260704-220719-bastion-data-catalog.md`).
- Prelude name collisions with bevy: a public type re-exported through
  `crate::prelude` must not share a name with anything in `bevy::prelude`.
  A game-local `struct Foo` silently shadows a bevy-prelude `Foo`, so it never
  clashes in an example; the moment you harvest it into the crate prelude,
  `use bevy::prelude::*` + `use bevy_common_systems::prelude::*` make every
  reference ambiguous (E0659). This bit the harvested unified pointer, whose
  natural name `Pointer` collides with bevy's `bevy_picking` `Pointer` event --
  it had to become `UnifiedPointer`. Check new prelude names against
  `bevy::prelude` before committing to them
  (`docs/retros/20260704-161508-input-pointer.md`).
- Doctests that construct and configure an `App` are runtime tests, not just
  compile checks -- `cargo test --doc` actually runs them. `init_state` /
  `NextState` / any state transition panics at runtime without the state
  machinery, so a doctest built on `MinimalPlugins` alone compiles green then
  panics with "The `StateTransition` schedule is missing. Did you forget to add
  StatesPlugin or DefaultPlugins?". Give such doctests
  `(MinimalPlugins, bevy::state::app::StatesPlugin)` (or `DefaultPlugins`), the
  plugins the real app would have (`docs/retros/20260704-175425-leaf-helpers.md`).
- Web/wasm builds: `trunk` must run from the repo root (it fails with
  `Unable to find any Trunk configuration` from a subdir like `web/`), and
  `rand` on wasm needs the getrandom `wasm_js` backend. Both are handled in
  `web/scripts/build-games.sh` and `.cargo/config.toml`; see
  `docs/wasm-web-builds.md`. Verify web tooling through the real entry point
  (`npm run build`), not a hand-run of the underlying tool.
- Fresh worktrees have no `node_modules`. A sprout (or any new git worktree)
  starts without `web/node_modules` (git-ignored, not copied from the main
  checkout), so the first `npm run build` fails its webpack half with
  `webpack: command not found` (exit 127) even though the trunk half succeeds.
  Run `npm ci` in the worktree's `web/` before the first web build.
- GitHub Actions workflow changes: validate them locally with
  `nix run nixpkgs#actionlint -- .github/workflows/<file>.yml` (exit 0, no
  output means clean) instead of pushing and watching the run. The devshell has
  no `actionlint` or `python3-yaml` on PATH, but the `nix run` form works. The
  Pages deploy (`pages.yml`) also retries `actions/deploy-pages` once because
  that step intermittently returns a transient "Deployment failed, try again
  later." -- do not remove the retry as redundant
  (`docs/2026-07-04-pages-deploy-retry.md`).
- A state-machine example screenshot at *state entry* is not gameplay
  verification. `ScreenshotPlugin` snaps as soon as it reaches the target state,
  before any input has driven the game, so a `BCS_SHOT` grab of a
  menu/playing/game-over example shows the initial frame only (e.g. `13_glide`'s
  board with just its two seed tiles, never a rendered merge). That is exactly
  where a bug in logic-that-drives-rendering hides: `13_glide` shipped a blocker
  where every merged tile displayed the stale un-doubled number because the
  move->entity classification (`merges` list always empty) was wrong, and it
  passed a headless autopilot run (no panic) plus a state-entry screenshot because
  neither observed a merge, and the moves-to-entity mapping had zero test coverage
  (`docs/retros/20260705-101442-glide-example.md`). Rule: when a pure function
  returns both a result and a list describing side effects (a grid *and* the
  per-tile moves), test the *list*, not just the result -- a correct grid/score
  actively masks a mishandled moves list downstream. Make the rendering-driver
  logic pure and unit-test it (`13_glide`'s `classify_moves`); that is more
  reliable than any screenshot. `ScreenshotPlugin` (app framebuffer) and
  `AutopilotPlugin` (drives the states) are mutually exclusive, and `scrot` of the
  X root is unreliable in a headless session (it returned a stale WM framebuffer
  from an orphaned earlier run), so do not lean on a live gameplay grab to catch
  these -- lean on the test.
- `AutopilotPlugin.hold(state, seconds)` force-drives the `States` machine on a
  fixed timer, so it proves only what happens *up to* each forced transition, never a
  transition the *game itself* makes. It is structurally blind to a game-driven state
  change -- most importantly the **lose condition**. `14_breach` reported a clean
  `Menu->Playing->GameOver` autopilot cycle with kills, yet the enemy melee was
  effectively broken (a defenceless player survived 30s+) and the player-death path
  (`Health` zero -> `RunOver` -> `GameOver`) never actually fired: the observed
  GameOver was always the `.hold(Playing, N)` timer, and the "kills" only proved the
  offence side (`docs/retros/20260705-114236-breach-example.md`). Rule: verify any
  game-driven transition (lose/win/level-up) with a headless `App` unit test
  (`MinimalPlugins` + `bevy::state::app::StatesPlugin` + the relevant crate plugin)
  that drives the trigger and asserts the state/resource flips -- write it before
  trusting the autopilot. And to debug *balance* headlessly (not just pass/fail),
  extend the Playing `.hold`, neuter the autopilot's offence (stop it firing) and log
  the relevant per-frame numbers (HP, distances): measurement turns "the player
  mysteriously survives" into "enemies arrive at 7s and melt 100->0 in 3s". Related
  avian-gameplay traps seen here: a cooldown-gated distance melee is unreliable when
  both bodies are dynamic (collision knockback flings the attacker out of range each
  frame -- use continuous proximity damage and/or drop the collision between them),
  and straight-line enemy AI with no avoidance gets stuck on interior obstacles (keep
  the arena open, or give the AI navigation).
