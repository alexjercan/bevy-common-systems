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
  - `registry` - `EventHandlerRegistry<W>`: maps event / filter / action name
    strings to registered constructors so `EventHandler`s can be authored in
    JSON (`HandlerSpec`) and built at runtime, the data-driven counterpart to
    the Rust `EventHandler` builder. Demoed by `examples/03_modding`.
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
  (`xdotool`) is around, add a TEMPORARY env-gated autopilot system that drives
  the state machine and controls the ship, run it under `timeout`, confirm the
  log shows the cycle completing with no panic, then remove the harness before
  commit. Used to verify 07/08 gameplay twice now
  (`tasks/20260703-213510`, `tasks/20260704-103544`). A hard `std::process::exit`
  in that harness segfaults on wgpu teardown -- harmless, but do not mistake it
  for a game crash.
- Seeing the screen (do not treat "a background run cannot see the screen" as a
  hard limit): when `$DISPLAY` is set, a background session CAN screenshot the
  running app and verify the visual layer. `scrot` and ImageMagick `import` grab
  the root window; `xdotool` (via `nix run nixpkgs#xdotool` when not on PATH)
  finds/moves the app window so you can crop it precisely (`magick IN -crop
  WxH+X+Y +repage OUT`), and `import`/`Read` the PNG to actually look. To capture
  a specific state or viewport, add a TEMPORARY env-gated harness (same pattern
  as the gameplay autopilot): force a window size with
  `Window { resolution: WindowResolution::new(w, h), resizable: false, .. }` and
  auto-advance the state machine, screenshot, then remove the harness before
  commit. This caught a real 09_reactor layout regression -- at phone width four
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
