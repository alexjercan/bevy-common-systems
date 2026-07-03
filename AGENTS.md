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
- `physics/`
  - `pd_controller` - `PDControllerPlugin`: computes the PD torque needed
    to rotate an avian3d rigid body (the `PDControllerTarget` entity)
    toward the `PDControllerInput` rotation; game code applies the
    resulting `PDControllerOutput`.
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

## Conventions

The modules are deliberately uniform. Follow these patterns when adding or
changing code; consistency is the crate's main defense against bloat.

- One concern per module. Modules that add runtime behavior ship one
  plugin per concern, named `*Plugin`; pure utility modules (meth,
  mesh/builder) export plain types and functions instead. Systems that
  need ordering hooks get a public `SystemSet` enum named `*Systems`
  (a few use `*PluginSystems`).
- Config / Input / Output / State component split:
  - a public config component named after the feature (`WASDCamera`,
    `ChaseCamera`, `PDController`, `SphereOrbit`, ...);
  - a public `*Input` component that game code writes each frame;
  - a public `*Output` component (or direct `Transform` writes) that game
    code reads;
  - private `*State` components the plugin manages internally - keep them
    out of the prelude.
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
- `bevy_enhanced_input` 0.25 - input contexts/actions for the WASD
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
./scripts/check-ascii.sh                     # enforce the plain-ASCII rule
cargo run --example 01_sphere                # opens a window
cargo run --example 01_sphere --features debug   # same, with inspector UI
```

(The only warning surfaced by clippy is a future-incompat note from the
transitive `proc-macro-error2` dependency, not from this crate's code.)

CI (`.github/workflows/ci.yml`) runs this whole suite - fmt, both clippy
configs, both test configs, and `scripts/check-ascii.sh` - on every push
and pull request. Keep it green; run the commands above locally before
pushing.

Testing convention: pure math and geometry functions get `#[cfg(test)]`
unit tests next to the code (13 today, in `meth/sphere`, `mesh/builder`,
`physics/pd_controller` and `transform/point_rotation`); ECS behavior is
exercised by the examples, which are the de facto integration tests. If
you add a feature, test its pure logic in-module and wire the ECS side
into an existing example or a new numbered one (`examples/NN_name.rs`,
clap CLI header, `DefaultPlugins`, setup system), and make sure everything
compiles via `cargo clippy --all-targets`.

Examples:

- `01_sphere` - octahedron sphere from `TriangleMeshBuilder` + WASD camera.
- `02_planet` - the same mesh displaced with Fbm/Perlin noise: a planet.
- `03_modding` - the `modding` event bus end to end, including
  `#[derive(EventKind)]`; the event logic prints to the console.
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
- `08_dropzone` - a lunar-lander game and the headline demo of
  `PDControllerPlugin`. A noise-displaced planet (the `02_planet` recipe) sits
  at the origin with radial gravity; you fly a lander down onto it. Space/Up
  thrusts along the ship's local up, W/S and A/D lean the target attitude, and
  the PD controller torques the avian3d rigid body toward that attitude (this
  is the crate's first real physics sim, not just the debug renderer). Touch
  down slow and upright to score; hit too hard or too tilted and the hull
  breaks apart via `mesh/explode`. Pulls in `camera/skybox` (a procedurally
  generated starfield, no asset file), `camera/post` bloom on the thruster
  flame, `camera/chase`, `ui/status` gauges (altitude/speed/fuel) and `audio`.
  The planet's avian trimesh collider is built inline from
  `TriangleMeshBuilder::vertices_and_indices()`. Follows the `06_fruitninja`
  shape (states, sounds, wasm). See `docs/2026-07-03-dropzone-example.md`;
  the flight constants are tuned by reasoning and may need play-testing.

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
- Verifying builds: never judge a build/command by a piped `| tail`'s exit
  code -- the pipe reports `tail`'s status, so a failed `cargo build | tail`
  looks like it passed. Redirect to a file and check `$?` when pass/fail
  matters; use `| tail` only for interactive peeking.
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
