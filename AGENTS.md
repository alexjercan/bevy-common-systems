# AGENTS.md

Orientation for agent sessions. Read first.

## What This Crate Is

`bevy_common_systems`: Bevy utilities so a new game does not rewrite cameras,
health, orbit motion, procedural meshes, status-bar UI, a modding event bus.
"Fully copy-pastable" -- each module lifts out on its own, and the crate also
works as a normal dependency.

Optimize for that: small, composable, game-agnostic blocks with obvious APIs.
No framework machinery.

## Agent workflow

- Tracker/epics: tatr tasks in `tasks/<id>/TASK.md`; a multi-task plan is an EPIC task with STORY children. See "Development flow".
- Examples/retention: runnable examples in `examples/NN_name.rs` are the integration tests; keep them all. See "Examples".
- Domain docs: `examples/README.md` (games + headless harness), `web/README.md` (showcase site + wasm builds). Reference only.
- Research/network: `/spike` -> `tasks/<id>/SPIKE.md`. No network needed for build/test.
- Checks/records: `tatr check` (+ `--ledger LESSONS.md`); records per task (see "Where records go"). See "Build, Verify, Run".

## Repository Layout

| Path | What |
| --- | --- |
| `src/` | library crate, one dir (or file) per concern |
| `bevy_common_systems_macros/` | proc-macro subcrate; `#[derive(EventKind)]` |
| `examples/` | runnable `NN_name.rs`; integration tests + quickstart. `README.md`: game list + headless harness |
| `tasks/` | tatr tracker, versioned with the code |
| `LESSONS.md` | lessons ledger; read before any task |
| `web/` | TypeScript + webpack showcase serving examples as wasm (trunk). `README.md`: site + wasm build |
| `flake.nix`, `rust-toolchain.toml`, `rustfmt.toml`, `.cargo/config.toml` | toolchain |

## Where records go (/plan, /spike, /work, /review, /compound, /flow)

Everything for one task in that task's folder; never a loose per-task `.md` elsewhere in the repo.

| File | Content |
| --- | --- |
| `tasks/<id>/TASK.md` | the task (tatr) |
| `tasks/<id>/SPIKE.md` | research that scoped it |
| `tasks/<id>/REVIEW.md` | review rounds + verdicts |
| `tasks/<id>/RETRO.md` | retrospective |
| `tasks/<id>/NOTES.md` | design/fix record: what changed, why, alternatives, difficulties |

Pruned task worth keeping: recreate the folder as a CLOSED archive-stub
`TASK.md` (a folder without `TASK.md` breaks `tatr ls`) plus the record.

## Module Map

- `audio/`
  - `SfxPlugin` - one-shot SFX. Trigger `PlaySfx` / `commands.play_sfx(handle)`; spawns a self-despawning `AudioPlayer`. `SfxMasterVolume` scales all. SFX only, no music/mixer.
  - `registry` - `SoundBank`: named handle registry keyed by a game `Copy` enum; owns loading (`sounds/<name>.wav`) + `get`. `all_loaded`/`sounds_loaded` drive loading screens.
- `camera/`
  - `chase` - `ChaseCameraPlugin`: third-person offset/smoothing/look-ahead. Game writes `ChaseCameraInput`.
  - `post` - `PostProcessingDefaultPlugin`: `PostProcessingCamera` gets `Tonemapping::TonyMcMapface` + `Bloom::NATURAL`.
  - `project` - `pointer_on_plane` (viewport pointer -> infinite plane) / `world_to_screen` (world -> pixel, `None` when off-screen or behind).
  - `shake` - `CameraShakePlugin`: trauma `0..1` decays to zero; jitter magnitude `trauma^exponent`. Absolute offset from a fixed base, never `+=`.
  - `skybox` - `SkyboxPlugin` + `SkyboxConfig`: stacked 6-face image -> cubemap -> Bevy `Skybox`.
  - `wasd` - `WASDCameraPlugin`: free-camera math from `WASDCameraInput`; bindings live in `helpers/wasd`.
- `debug/` (feature `debug` only)
  - `wireframe` - `WireframeDebugPlugin`: global wireframe, F11, starts enabled.
  - `inspector` - `InspectorDebugPlugin`: inspector-egui + avian gizmos + diagnostics, F11, starts enabled.
  - `harness` - env-gated headless tooling: `AutopilotPlugin` (`.hold(state, seconds)` timeline + optional input closure, exits via `AppExit`; `BCS_AUTOPILOT`) and `ScreenshotPlugin` (forces window size, advances to a state, settles N frames, writes PNG; `BCS_SHOT="WxH"`). Inert without the env var, so keep them in the example. Demoed by `08_dropzone`, `11_overload`.
- `feedback/` - short-lived "juice":
  - `flash` - `FlashPlugin`: `Flash` overrides a `StandardMaterial` emissive/base color and eases back. Clones the material per entity.
  - `screen_flash` - `ScreenFlashPlugin`: spikes a full-screen `ScreenFlash` alpha and decays it. Caller picks the tint.
- `health` - `HealthPlugin`: `Health`, `HealthApplyDamage` (entity event, propagates up), `HealthZeroMarker` at zero. Damage via `commands.trigger(...)`.
- `helpers/`
  - `despawn` - `DespawnEntityPlugin`: `DespawnEntity` despawns immediately.
  - `pointer` - `EnhancedInputPointerPlugin`: drives `UnifiedPointer` from a bevy_enhanced_input press action.
  - `temp` - `TempEntityPlugin`: `TempEntity(seconds)` auto-despawns.
  - `wasd` - `WASDCameraControllerPlugin`: binds WASD/mouse/space/shift, writes `WASDCameraInput`.
- `input/`
  - `pointer` - `UnifiedPointerPlugin` + `UnifiedPointer`: per-frame resource collapsing mouse/touch/cursor into position + down + just-pressed (touch wins). `PreUpdate`, no input-framework dep.
  - `cursor` - `grab_cursor` / `release_cursor` over the 0.19 per-window `CursorOptions`. Policy stays with the game.
  - `state` - `set_state_on_key`: factory system binding one key to a `States` transition.
- `integrity/` - `IntegrityPlugin`: destruction over a graph of connected health-bearing nodes.
  - `components` - `IntegrityRoot`, `ConnectedTo` (own neighbour list), `IntegrityLeafMarker` / `IntegrityDisabledMarker` / `IntegrityDestroyMarker`.
  - `blast` - `blast_damage()`: radial `BlastDamageConfig` sensor bundle, owns its collision events, linear-falloff damage.
  - `damage` (private) - the collision half: impacts (impulse from relative velocity + mass) and blast overlaps -> `HealthApplyDamage`. Owns the three damage constants and the avian dependency.
  - `plugin` - `IntegrityPlugin` wiring plus the cascade: disable at zero; destroy disabled leaves (or a disabled root); prune from neighbours to cascade. Game owns both seams: builds the graph, reacts to `On<Add, IntegrityDestroyMarker>`. Demoed by `15_integrity`.
- `material` - `glowing_material`: the emissive-that-blooms `StandardMaterial`; bakes in the "must NOT be `unlit`" footgun.
- `mesh/`
  - `builder` - `TriangleMeshBuilder`: octahedron spheres, subdivision, noise displacement, plane slicing, normals/UVs, `Mesh` conversion.
  - `slice` (private) - the triangle-vs-plane geometry kernel behind `builder`'s `slice()`. Pure math, total by construction (degenerate/parallel input stays finite).
  - `explode` - `ExplodeMeshPlugin`: `ExplodeMesh` slices an entity's mesh (and children) into `ExplodeFragments`.
- `meth/` - math helpers (pun is intentional, do not "fix" it)
  - `lerp` - `LerpSnap`: exponential lerp with snap-to-target for `f32`/`Vec3`.
  - `sphere` - spherical coordinate conversions + `slerp`.
- `modding/`
  - `events` - serde-friendly event bus: `EventWorld`, `EventKind`, `EventHandler` entities (filters + actions), `GameEventsPlugin<W>`, `Commands::fire`. Payloads are `serde_json::Value`; filters/actions are Rust trait objects.
  - `registry` - `EventHandlerRegistry<W>`: name -> constructor, so handlers are authored in JSON (`HandlerSpec`) and built at runtime.
- `persist/` - `PersistPlugin<T>`: load a serializable `Resource` at startup, save on change. Native JSON under `dirs::data_dir()/bevy_common_systems/<key>.json` (or `$BCS_PERSIST_DIR`); wasm `localStorage`. `backend` hides the platform behind `load`/`save`.
- `physics/`
  - `pd_controller` - `PDControllerPlugin`: PD torque toward `PDControllerInput` rotation; game applies `PDControllerOutput`.
  - `doom_controller` - `DoomControllerPlugin`: arena-shooter FPS controller. Config + `DoomControllerInput` (look delta, move intent) + `DoomControllerState` (yaw/pitch, settable) + `DoomControllerOutput` (planar velocity the game writes into `LinearVelocity`, leaving `.y` to gravity). Orients a `DoomEye` camera child. Output-only -- no avian dep. Requires `LockedAxes::ROTATION_LOCKED` + a `DoomEye` child. Name reserves `FirstPersonController` for a richer future one.
  - `rigid_body` - `rigid_body_point_velocity` (`v = v_lin + omega x (p - com)`) and `destructible_body(health, density)` (pair with `integrity`).
- `scoring/` - no `Score` type on purpose (a running score is the game's number):
  - `streak` - `Streak`: hit/combo count that decays on a time window.
  - `high_score` - `HighScore<T>`: best value + per-run "new best" edge; `PartialOrd + Copy`, serde (edge not serialized), composes with `PersistPlugin`.
- `time/`
  - `cooldown` - `Cooldown`: countdown for fire gates / i-frames. Plain value, `tick`ed each frame; a fresh one is READY (unlike a fresh `Once` `Timer`).
- `transform/` - motion drivers; each writes an Output your systems apply:
  `sphere_orbit` (theta/phi input), `directional_sphere_orbit` (toward a direction),
  `random_sphere_orbit` (wander), `point_rotation` (accumulate mouse deltas),
  `smooth_look_rotation` (toward a target angle, speed + optional limits).
- `tween` - `TweenPlugin` + `Tween<T>`: fixed start -> end over a fixed duration, shaped by `EaseFunction`. An output component + completion marker, NOT a keyframe timeline. Counterpart to `meth/lerp`'s open-ended smoothing.
- `ui/`
  - `status` - `StatusBarPlugin` + `status_bar()` / `status_bar_item()`: corner metrics overlay driven by `value_fn`/`color_fn`. Closures run in an exclusive system every frame (`&World`, blocks parallelism) -- keep them cheap.
  - `animate` - `UiAnimatePlugin` + markers copying a `Tween` into plain UI fields: `TweenNodeOffset` (`Vec2` -> `Node.left/top` px), `TweenNodeScale` (`f32` -> width/height percent), `TweenNodeBackground` (`Vec4` -> `BackgroundColor`); `color_to_vec4`/`vec4_to_color`, `node_flash()`.
  - `menu` - `MenuPlugin` + `centered_screen()` / `screen_text()` + `TitlePulse`. Pieces, not a menu framework; game owns content and states.
  - `popup` - `PopupPlugin`: screen-space `Popup` label rises, fades, despawns. Anchor world events with `camera/project::world_to_screen`.
  - `touchpad` - `TouchpadPlugin`: reveal-on-first-touch (`TouchSeen`, `RevealOnTouch`/`HideOnTouch`, no `wasm32` sniffing) + hit-test primitives `button_grid_at` / `stick_deflection`. Primitives, not a pad widget.
  - `health_display` - `HealthDisplayPlugin` + `health_display()`: one-line "Health: N%" tracking a target entity.
  - `objectives` - `ObjectivesPlugin` + `objectives_panel()`: list rebuilt from `GameObjectives` (opaque id + message per line).

## Conventions

Modules are deliberately uniform; consistency is the main defense against bloat.

- One concern per module. Runtime behavior ships one `*Plugin`; pure utility modules export plain types/functions. Ordering hooks get a public `SystemSet` named `*Systems` (a few `*PluginSystems`).
  - `.before(SetX)`/`.after(SetX)` orders only against `SetX`'s *current members*. An empty set (sibling plugin not added) silently drops the edge. Pin real dependencies directly (`configure_sets(schedule, B.after(A))` or `chain()`), never via a third set that might be empty. Bit `camera/shake` (`tasks/20260704-134500/RETRO.md`).
- Config / Input / Output / State split:
  - config component named after the feature (`WASDCamera`, `ChaseCamera`, `PDController`, `SphereOrbit`, ...);
  - public `*Input` the game writes each frame;
  - public `*Output` (or direct `Transform` writes) the game reads;
  - private `*State` the plugin manages -- keep out of the prelude.
  - Ordering contract: any `*Systems` set that reads `*Input` or writes `*Output` MUST document it on the set's doc -- write Input `.before(TheSet)`, read Output `.after(TheSet)`. Undocumented = every consumer races it; cost `14_breach` a one-frame input lag the autopilot could not catch (`tasks/20260705-132542/RETRO.md`).
- Preludes: most files define `pub mod prelude`, parents aggregate children, `crate::prelude` aggregates everything. Users import `bevy_common_systems::prelude::*`.
- Reactive setup via observers (`add_observer` on `On<Add, X>` / `On<Insert, X>`), not startup systems. `#[require(...)]` where a component needs companions.
- Derive `Reflect` on components (+ `Deref`/`DerefMut` for newtypes).
- Module-level `//!` doc with a usage snippet; doc comments on public items.
- Logging: `debug!("XPlugin: build")` in `Plugin::build`, `trace!` in systems/observers.
- rustfmt owns imports (`imports_granularity = "Crate"`, `group_imports = "StdExternalCrate"`). Clippy allows `type_complexity` and `too_many_arguments` crate-wide.
- Plain ASCII everywhere (code, comments, docs, commits): `-`, `--`, `...`, `->`.
- Own-line (non-doc) comments only guard a value, explain a non-obvious setting, or record a hazard, and open with `NOTE:` / `FIXME:` / `BUG:` / `TODO:` on the block's first line. Restatement and task narration go to `tasks/<id>/NOTES.md`. Enforced by `./scripts/check-comment-tags.sh <path>...`. Exempt: rustdoc, and end-of-line comments (`let x = 1; // 1 neighbor`), which label a value in place and read worse tagged -- they still must earn their keep.
  - In a `#[cfg(test)]` module the same split applies: what the TEST proves, or why it exists, is a `///` doc comment on the test fn (it outlives any line in the body, and `cargo test -- --list` shows it); what guards a VALUE inside the body stays a tagged `NOTE:` block. The `///` form is exempt from the checker, so use it for test intent only -- never to move an untagged body comment out of its reach (`tasks/20260731-172224/NOTES.md`).

### Promoted ledger lessons (folded 2026-07-20, task 20260720-220050)

Recurred x3+; conventions now.

- Reset shared state in the same commit as the new accumulator/timer/per-run resource. Grep the reset / `start_run` path; extract a helper past three resets.
- An `*Input` writer runs in EVERY state, or zero the Input on state exit. A state-gated writer leaves a stale value the plugin keeps integrating.
- Read a plugin's `build()` for self-added deps before wiring it in (e.g. `PopupPlugin` adds `TweenPlugin`; adding both panics).
- Harvest by reading evidence: survey precedent + its homes, reproduce bodies byte-for-byte, refactor call sites as the test, delete now-dead markers from the consumer.
- Split verifiable from manual: "assets load / logic runs" (headless) vs "audible/visible / transition fires" (play-test). State which real paths a shortcut harness does NOT exercise.

## Features and Dependencies

Features (default none): `debug` compiles the `debug` module and enables
`avian3d/diagnostic_ui`, `bevy/track_location`, `bevy-inspector-egui`.
`dev` is an alias for `debug`.

| Dep | Why |
| --- | --- |
| `bevy` 0.19 | engine; tracks current APIs (observers, `EntityEvent`, required components) |
| `avian3d` 0.7 | 3D physics: PD controller, debug diagnostics, examples |
| `bevy_enhanced_input` 0.26 | input contexts/actions for `helpers/wasd` |
| `bevy_asset_loader` 0.27 | loading states; backs `SoundBank` + example loading screens |
| `noise` 0.9 | noise for `TriangleMeshBuilder` |
| `rand` 0.9 | orbits, mesh explosion |
| `serde` / `serde_json` | `modding` payloads, `persist` resources |
| `bevy-inspector-egui` 0.37 (optional) | `debug` inspector |
| `clap` 4.5 (dev) | every example is a small CLI |

## Environment and Toolchain

- Rust nightly, pinned by `rust-toolchain.toml` (rustfmt + clippy). Edition 2021. MIT.
- `nix develop`: nightly toolchain, wasm tooling (trunk, wasm-pack, wasm32 target), Linux libs Bevy needs (vulkan, wayland/x11, alsa, udev). On NixOS graphical examples generally only run in this shell (`LD_LIBRARY_PATH`).
- `.cargo/config.toml` sets `--cfg=web_sys_unstable_apis` for wasm.

## Build, Verify, Run

Verified 2026-07-03. CI (`.github/workflows/ci.yml`) runs this whole suite on
every push and PR -- keep it green, run it locally first.

```
cargo build                                  # library + macros subcrate
cargo fmt --check                            # formatting
cargo clippy --all-targets                   # lints; keep clean
cargo clippy --all-targets --features debug
cargo test                                   # unit tests + doctests
cargo test --features debug
cargo test --examples                        # the #[cfg(test)] tests in examples/
./scripts/check-ascii.sh                     # plain-ASCII rule
cargo run --example 01_sphere                # opens a window
cargo run --example 01_sphere --features debug
```

Only expected warning: a future-incompat note from transitive `proc-macro-error2`.

Testing convention:

- Pure math/geometry gets `#[cfg(test)]` next to the code (`meth/sphere`, `mesh/builder`, `physics/pd_controller`, `transform/point_rotation`).
- ECS behavior is exercised by the examples (the de facto integration tests). New feature -> unit-test the pure logic, wire the ECS side into an existing or new `examples/NN_name.rs` (clap CLI header, `DefaultPlugins`, setup system), compile via `cargo clippy --all-targets`.
- Back every claim a test comment or TASK.md note makes with an assertion in the same edit. Reaching a `pub(super)` field is cheaper than the review round that catches the gap (`tasks/20260704-165400/RETRO.md`, `tasks/20260703-165439/RETRO.md`).

## Examples

Every game example follows the `06_fruitninja` shape: menu/playing/game-over
states, `SfxPlugin` one-shots, wasm/trunk showcase build.

| Example | What it is | Headlines |
| --- | --- | --- |
| `01_sphere` | octahedron sphere + WASD camera | `mesh/builder`, `camera/wasd` |
| `02_planet` | `01` displaced with Fbm/Perlin noise | noise displacement |
| `03_modding` | event bus end to end, `#[derive(EventKind)]`, JSON handlers via `EventHandlerRegistry` | `modding` |
| `04_status_item` | status bar with FPS + shell-command items | `ui/status` |
| `05_explode` | Left Mouse Button slices a mesh into flying `ExplodeFragments` | `mesh/explode` |
| `06_fruitninja` | swipe-slice fruit for score; combos, "+N" popups, blade trail, lethal bombs | `SfxPlugin`, states shape. `tasks/20260703-152544/NOTES.md` |
| `07_orbit` | "Orbit Runner": ride a planet surface, sweep orbs, dodge hazards, difficulty ramps | whole `transform/*` orbit family, `camera/chase`, `meth`. `tasks/20260703-165427/NOTES.md` |
| `08_dropzone` | lunar lander onto a noise planet with radial gravity; thrust + lean, hazards (monoliths, asteroids, wind) drain hull, crash explodes | `PDControllerPlugin`, `camera/skybox`+`post`+`chase`, `ui/status`. `tasks/20260703-165432`, `20260704-103544`, `20260704-103553` NOTES; tuned in `20260703-213510` |
| `09_reactor` | rules-as-machine incremental: shop parts spawn JSON `EventHandler`s, HEAT 100 = meltdown. `Camera2d` | `modding` as gameplay. `tasks/20260704-170738/NOTES.md` |
| `10_asteroids` | top-down shooter; each sliced shard respawns as a real dynamic avian body and splits large -> medium -> small | physics fragments, broad avian slice (sensors, `CollisionLayers`, `Gravity::ZERO`). `tasks/20260703-170744/NOTES.md` |
| `11_overload` | dashboard survival: four coupled gauges, 1/2/3/4 vents, red gauges drain `Health`. `Camera2d`, touch vent pad | `ui/status` as a game surface. `tasks/20260704-165400`, `20260704-130314` NOTES |
| `12_bastion` | tower defense; tap places towers, waves ramp, stats loaded from `assets/bastion/catalog.json` (wasm uses a compiled-in copy) | `camera/project`, `point_rotation` orbit cam, `smooth_look_rotation` turrets. `tasks/20260704-220736/NOTES.md` |
| `13_glide` | 2048-style slide-merge, entirely Bevy UI; every animation drives `Node`/`BackgroundColor` from a `Tween` (never `Transform` scale). Space hands off to a unit-tested expectimax solver | `tween`, `persist` + `HighScore`, `ui/animate` source. `tasks/20260705-090624`, `20260705-143000` NOTES |
| `14_breach` | Doom-like FPS arena: hitscan `SpatialQuery` gun, three enemy archetypes, combos, pickups/buffs, navigable menu with persisted look sensitivity | `physics/doom_controller`, `ui/touchpad`, `feedback/*`. `tasks/20260705-103236`, `20260705-103238`, `20260705-132200` NOTES |
| `15_integrity` | grid of connected blocks in zero-g; click detonates a blast, disabled patches cascade apart from the edges in | `integrity`, `ui/health_display`, `ui/objectives`. `tasks/20260708-112713/NOTES.md` |

## Development flow

- /flow drives it: /plan -> tatr tasks, /work implements each in a sprout worktree, /review runs out-of-context round-1 reviews until APPROVE, /compound writes the retro. Repeat until done.
- Done criteria carry machine-checkable proofs in `test:` / `cmd:` / `manual:` notation; a task closes only when its proofs pass.
- `LESSONS.md` is the ledger: read before any task. /compound appends; /lessons folds loose scratch in.
- `tatr check` (+ `--ledger LESSONS.md`) gates task artifacts and the ledger. Keep clean.
- Feature branch per task, merged into `master` after review. Do not push unless asked.

## Gotchas

- `meth` is the real module name (math pun). Leave it.
- `README.md` is a stale three-line stub; this file is the orientation doc.
- `EventKind` derive: the default `Info` is `()` and the default name is the lowercased struct name, so an attribute-less `#[derive(EventKind)]` compiles; pass `#[event_info(...)]` when the event carries a payload, as `03_modding` does. The old "default path does not resolve" warning (`tasks/20260703-095509`) described a defect fixed since, and `attribute_less_derive_defaults_to_no_payload` in `modding::events` now guards the default (`tasks/20260731-172232`).
- `helpers/wasd` + `camera/wasd` are two halves of one feature; the camera math is input-agnostic on purpose.
- Bevy 0.19 UI/light API: copy idioms from an existing example, never from memory. `TextFont.font_size` is `FontSize::Px(..)`; `TextLayout` is a struct literal (no `new_with_justify`); `AmbientLight` is per-camera, not a resource; rounded corners are `Node { border_radius: BorderRadius::MAX, .. }` (a field -- spawning `BorderRadius` fails the `Bundle` bound) while `BorderColor` IS a component. Grep `06`/`08` for `font_size:`, `TextLayout`, `AmbientLight`, `border_radius:` first. Bit three cycles (`tasks/20260703-150200`, `20260703-165432`, `20260704-103517` RETROs).
- Same "copy, do not improvise the visual layer" rule for things that render wrong *silently* (no panic, and a background run cannot see the screen): an HDR `emissive` material must NOT be `unlit` (skips the lighting pass where emissive applies); an entity with mesh children but no mesh of its own needs explicit `Visibility` (else `B0004`); camera shake is `translation = BASE + offset`, never `+=`. All three shipped and were caught in `10_asteroids` review (`tasks/20260703-170744/RETRO.md`).
- Running examples: a new/changed example is not done until it has been RUN once -- `cargo build` proves compile, not boot. Check `echo $DISPLAY`; run under `timeout` and confirm a `bevy_render::view::window` swap-chain line. Skipping this shipped a startup hang in `08_dropzone` (`tasks/20260703-165432/RETRO.md`).
- Booting only reaches the menu. For gameplay headlessly, do NOT hand-roll an autopilot (re-invented and deleted 7 times). Use `AutopilotPlugin` behind `#[cfg(feature = "debug")]`: `.hold(state, seconds)` + optional `.input(|world, elapsed| ...)`, then `BCS_AUTOPILOT=1 cargo run --example NN --features debug` under `timeout`; confirm each `autopilot: -> State` line and `autopilot: cycle complete, no panic`. Exits via `AppExit::Success` (a raw `std::process::exit` segfaults on wgpu teardown -- harmless, not a crash). See `examples/README.md`; `08_dropzone`/`11_overload` are the reference.
- Seeing the screen is possible: with `$DISPLAY` set, `scrot` / ImageMagick `import` grab the root window, `xdotool` (`nix run nixpkgs#xdotool`) finds/moves the window for a precise crop (`magick IN -crop WxH+X+Y +repage OUT`), then Read the PNG. For a specific state/viewport use `ScreenshotPlugin::new(TargetState).settle_frames(n).path("shot.png")` with `BCS_SHOT=390x844` -- do not hand-roll it (re-invented twice). Caught a real `09_reactor` phone-width regression: four of six shop buttons below the fold, invisible to build/clippy/boot check (`tasks/20260704-143000/RETRO.md`).
- Responsive Bevy 0.19 grid holding N columns at any width: percentage item widths, not fixed px + `flex_wrap` (flexbox wraps before it shrinks).
- `AutopilotPlugin` and `ScreenshotPlugin` are mutually exclusive in one run -- both drive `NextState`, so the screenshot never settles. For a mid-gameplay frame use `ScreenshotPlugin` alone, or autopilot + external `scrot` (`tasks/20260704-220736/RETRO.md`).
- A screenshot at *state entry* is not gameplay verification: it snaps before any input. `13_glide` shipped a blocker (every merged tile showed the stale un-doubled number; the `merges` list was always empty) that passed both an autopilot run and a state-entry shot (`tasks/20260705-101442/RETRO.md`). Rule: when a pure function returns a result AND a list describing side effects, test the *list* -- a correct grid actively masks a mishandled moves list. Make rendering-driver logic pure and unit-test it (`classify_moves`); more reliable than any screenshot. `scrot` of the X root is unreliable headless (returns stale WM framebuffers).
- `AutopilotPlugin.hold` force-drives states on a timer, so it is structurally blind to any transition the *game* makes -- above all the lose condition. `14_breach` reported a clean `Menu->Playing->GameOver` cycle while enemy melee was broken and the death path never fired (`tasks/20260705-114236/RETRO.md`). Rule: verify game-driven transitions (lose/win/level-up) with a headless `App` test (`MinimalPlugins` + `bevy::state::app::StatesPlugin` + the plugin) that drives the trigger and asserts the flip -- before trusting the autopilot. For balance, extend the `.hold`, neuter the autopilot's offence and log per-frame numbers (HP, distances).
- Related avian traps: a cooldown-gated distance melee is unreliable when both bodies are dynamic (knockback flings the attacker out of range) -- use continuous proximity damage and/or drop the collision; straight-line AI with no avoidance snags on interior obstacles -- keep the arena open or add navigation.
- Spurious rotation on a sphere is almost always `Quat::from_rotation_arc(Vec3::Y, up)`: correct for the up axis, but it commits to whatever yaw/twist falls out of the shortest arc. That twist swings as `up` sweeps (parallel-transport holonomy, ~80 degrees toward the far side) and goes singular at the `-Y` antipode. It yawed the hull and rolled the camera in `08_dropzone` (`tasks/20260705-154507/NOTES.md`). Fix: build the frame from `up` plus a `forward_ref` projected into the tangent plane (`surface_frame`), never a bare shortest-arc. Two traps in the same fix: `Quat::from_mat3` on an improper (det -1) basis returns garbage -- get cross-product handedness right; and a regression test must be shown to actually observe the bug (the first probe was parallel to the rotation axis and measured a swing of exactly 0).
- Verifying builds: never judge by a piped `| tail` exit code -- the pipe reports `tail`'s status. Redirect to a file and check `$?` when pass/fail matters.
- Bare `cargo build` does NOT compile examples -- a false green for anything touching example call sites. Use `cargo clippy --all-targets` (or `cargo build --examples`). Hid eight errors in `input/pointer` (`tasks/20260704-161508/RETRO.md`).
- `include_str!`ed data goes stale when the same task edits the file. Assert the *final shipped* state and re-run `cargo test --examples` AFTER the data edit. Bit `12_bastion`'s catalog: passed at 2+2, failed once it became 3+3 (`tasks/20260704-220719/RETRO.md`).
- Prelude names must not collide with `bevy::prelude`. A game-local `struct Foo` silently shadows in an example, then makes every reference ambiguous (E0659) once harvested. Cost the unified pointer its natural name (`Pointer` -> `UnifiedPointer`) (`tasks/20260704-161508/RETRO.md`).
- Doctests that configure an `App` actually RUN (`cargo test --doc`). `init_state`/`NextState` panic on `MinimalPlugins` alone ("The `StateTransition` schedule is missing"). Use `(MinimalPlugins, bevy::state::app::StatesPlugin)` or `DefaultPlugins` (`tasks/20260704-175425/RETRO.md`).
- Web/wasm: `trunk` must run from the repo root (fails with `Unable to find any Trunk configuration` from `web/`), and `rand` needs the getrandom `wasm_js` backend. Both handled in `web/scripts/build-games.sh` + `.cargo/config.toml`; see `web/README.md`. Verify through `npm run build`, not a hand-run of the underlying tool.
- Fresh worktrees have no `web/node_modules` (git-ignored, not copied), so the first `npm run build` fails its webpack half with exit 127 while trunk succeeds. Run `npm ci` in the worktree's `web/` first.
- Validate workflow changes with `nix run nixpkgs#actionlint -- .github/workflows/<file>.yml` (exit 0, no output = clean) instead of pushing. The devshell has no `actionlint` on PATH; the `nix run` form works. `pages.yml` retries `actions/deploy-pages` once on a transient "Deployment failed, try again later." -- do not remove it as redundant (`tasks/20260704-101608/NOTES.md`).
