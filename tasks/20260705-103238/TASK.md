# Follow-up: harvest FP character controller / camera-wasd upgrades from 14_breach

- STATUS: CLOSED
- PRIORITY: 40
- TAGS: spike,feature,harvest,fps

## Goal

After `14_breach` ships, evaluate harvesting its game-local first-person pieces into
the crate:

- a reusable **first-person character controller** (walk + gravity + ground check +
  collide-and-slide), the biggest gap -- the crate has no character controller;
- **`camera/wasd` upgrades**: optional always-on look (not just RMB-drag), a
  cursor-grab / pointer-lock helper, and a pitch clamp -- the small changes that turn
  the free-fly tech-demo camera into a game-ready one;
- optionally a **hitscan / `SpatialQuery` helper** if the raycast-and-damage pattern
  is reusable.

Decide which generalize cleanly (collide-and-slide is level-specific, so the split
matters) and what altitude each belongs at. Depends on the MVP (20260705-103236)
shipping first so there is a concrete reference.

## Decision (from evaluation)

Promote TWO pieces; keep the game-specific glue local.

- **`physics/doom_controller`** -- a Doom-style (simple, arena-shooter) first-person
  character controller. Named `DoomController` on purpose: it is the pragmatic
  stopgap, reserving the premium `FirstPersonController` / `FpsController` name for a
  more capable future controller (jump / crouch / air-control / slopes). Output-only,
  matching crate taste (`pd_controller`/`point_rotation`/`smooth_look` all expose an
  Output the game applies) -- so it takes NO avian dependency: it computes a desired
  horizontal velocity as `Output`, and the game writes that into `LinearVelocity`
  (leaving `.y` to gravity, letting avian's solver do collide-and-slide). It also owns
  the look: integrates a look delta into yaw/pitch (clamped) and writes a marked eye
  camera child's rotation.
- **`input/cursor`** -- `grab_cursor` / `release_cursor` free functions over the
  Bevy 0.19 `CursorOptions` window component. Small but non-obvious (0.19 moved cursor
  state to a per-window component); the game keeps the *policy* (when to grab, and the
  harness `headless()` guard).

NOT promoted: extending `camera/wasd` (it owns translation -- free-fly, can't do
collide-and-slide), extending `transform/point_rotation` (no pitch clamp, tumbling
basis), and a hitscan helper (the raycast is ~6 lines, below the bar). Recorded in
the harvest doc.

## Steps

- [x] **Create `src/physics/doom_controller.rs`** following the crate module shape
  (`pd_controller`/`ui/animate` as templates): module `//!` doc (Doom-style, why the
  name reserves the premium one, a runnable doctest), `pub mod prelude`, `Reflect`
  derives.
  - `DoomController` config: `move_speed: f32`, `look_sensitivity: f32`,
    `pitch_min: f32`, `pitch_max: f32`; `#[require(DoomControllerInput,
    DoomControllerState, DoomControllerOutput)]`.
  - `DoomControllerInput { look: Vec2, movement: Vec2 }` (look = raw delta this frame;
    movement = (strafe, forward) intent) -- written by game code.
  - `DoomControllerState { yaw: f32, pitch: f32 }` -- PUBLIC (game sets it to aim /
    face on spawn); the plugin integrates `input.look` into it and clamps pitch.
  - `DoomControllerOutput { velocity: Vec3 }` -- desired horizontal velocity (y = 0);
    the game applies it to `LinearVelocity`.
  - `DoomEye` marker -- put on the camera child; the plugin writes its
    `Transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0)`.
  - pure `doom_move_dir(yaw: f32, movement: Vec2) -> Vec3` (forward +Y intent = -Z at
    yaw 0, strafe +X, yaw-rotated, y-zeroed, normalized) -- from `14_breach::move_dir`.
  - `DoomControllerSystems::Drive` set (Update); `DoomControllerPlugin` registers types
    and adds `drive_doom_controller` (integrate look->state, clamp, set output.velocity)
    + `orient_doom_eye` (write the `DoomEye` child rotation from its parent's state, via
    `ChildOf`) in the Drive set. Init of Input/State/Output is via `#[require(...)]` on
    the config (cleaner than pd_controller's observer -- no double-insert).
- [x] **Unit-test the module** (`#[cfg(test)]`): `doom_move_dir` (forward=-Z at yaw 0,
  strafe=+X, 90deg yaw rotates forward to -X, zero input = zero), pitch clamping, and a
  headless `App` test (MinimalPlugins + `DoomControllerPlugin`) that feeds an `Input`,
  steps, and asserts the `State` integrated + clamped and `Output.velocity` is right.
- [x] **Wire `physics/mod.rs`**: `pub mod doom_controller;` + `pub use
  super::doom_controller::prelude::*;` in the prelude block (flows up via `lib.rs`'s
  `physics::prelude::*`).
- [x] **Create `src/input/cursor.rs`**: `grab_cursor(&mut CursorOptions)` (Locked +
  invisible) and `release_cursor(&mut CursorOptions)` (None + visible), module doc, a
  `pub mod prelude`. Wire `input/mod.rs`: `pub mod cursor;` + `cursor::prelude::*`.
- [x] **Rewire `examples/14_breach.rs`** onto the modules:
  - Replace the game-local `FirstPersonController` with `DoomController { move_speed:
    PLAYER_SPEED, look_sensitivity: LOOK_SENS, pitch_min/max: -/+PITCH_LIMIT }` on the
    player body; put `DoomEye` on the eye camera child (replacing `PlayerCamera` --
    update the gun's camera query to `With<DoomEye>`).
  - Delete `move_dir`, `clamp_pitch`, `player_look`, `player_move`; add `feed_look`
    (write `DoomControllerInput.look` from `AccumulatedMouseMotion` + touch), `feed_move`
    (write `DoomControllerInput.movement` from WASD + touch), and `apply_move_velocity`
    (write `LinearVelocity.x/.z` from `DoomControllerOutput.velocity`, preserving `.y`)
    ordered `.after(DoomControllerSystems::Drive)`.
  - Point `grab_cursor`/`release_cursor` at `input::cursor::{grab_cursor,release_cursor}`
    (keep the game's `headless()` guard wrapping them).
  - Update the autopilot input closure to set `DoomControllerState.yaw` (was
    `FirstPersonController.yaw`); move the `move_dir`/`clamp_pitch` unit tests to the
    module (drop them from the example).
- [x] **Verify**: `cargo clippy --all-targets` + `--features debug` clean; `cargo test`
  (module tests) + `cargo test --example 14_breach`; `scripts/check-ascii.sh`;
  `cargo fmt --check`. Re-run `BCS_AUTOPILOT=1 ... --features debug` on `14_breach` and
  confirm Menu->Playing->GameOver, "cycle complete, no panic", no runtime errors, and a
  positive kill count (the controller still aims/moves/shoots). `BCS_SHOT` grab to
  confirm it still renders.
- [x] **Docs**: add `tasks/20260705-103238/NOTES.md` (mirror
  `tasks/20260705-090557/NOTES.md`: what shipped, the `DoomController`
  naming rationale, output-only/no-avian-dep decision, the eye-child + axis-locked-body
  requirements, what stayed game-local, and the verdict per candidate incl. why
  `camera/wasd`/`point_rotation` were not extended). Update `AGENTS.md`: add
  `physics/doom_controller` + `input/cursor` to the module map, and update the
  `14_breach` example entry to say it now uses them.

## Notes

Spike: tasks/20260705-103116/SPIKE.md

### API facts (verified, so the implementer does not re-search)

- Harvest templates: `src/ui/animate.rs` (recent harvest shape: module doc + doctest,
  `pub mod prelude` exporting only public surface, marker components with `Reflect`,
  `*Systems` set, `*Plugin` with `register_type` + `add_systems`), and
  `src/physics/pd_controller.rs` (Config/`*Input`/`*Output` split, `#[require(..)]`,
  `add_observer(setup_*)` on `On<Add, Config>`, output-only avian pattern).
- `physics/mod.rs` re-exports children in its `pub mod prelude`; `lib.rs` already
  aggregates `physics::prelude::*` and `input::prelude::*`, so new submodules need no
  `lib.rs` change.
- `point_rotation` (`transform/`) accumulates two-axis deltas but has NO clamp and uses
  a tumbling basis; `smooth_look_rotation` clamps but is single-axis + smoothed. Neither
  fits an FP euler-look-with-pitch-clamp, so the module is new (do NOT extend them).
- `camera/wasd` owns translation (free-fly), so it cannot do avian collide-and-slide;
  do NOT extend it.
- `14_breach` FP pieces to move: `FirstPersonController` (~:326), `PlayerCamera` (~:332),
  `move_dir` (~:356), `clamp_pitch` (~:378), `player_look` (~:757), `player_move`
  (~:769), `grab_cursor`/`release_cursor`/`headless` (~:738-753), player+eye spawn
  (~:577-618), autopilot yaw write (~:156). Body is `LockedAxes::ROTATION_LOCKED`, look
  writes the eye child, move writes `LinearVelocity.x/.z` leaving `.y` to gravity.
- Bevy 0.19: `CursorOptions` is a per-window component (`Single<&mut CursorOptions,
  With<PrimaryWindow>>`); `CursorGrabMode::{Locked,None}`, `.visible`.

### Design decisions

- **Output-only, no avian dep in the module.** `DoomControllerOutput.velocity` is a
  plain `Vec3`; the game writes it to `LinearVelocity`. Keeps the module composable and
  avian-agnostic (matches `pd_controller`'s output-only taste) even though it lives in
  `physics/` (semantically a character controller).
- **Requires an axis-locked body + a `DoomEye` child.** Document this; the body must
  keep rotation locked (yaw lives in `State`, not on the body Transform), and the eye
  child carries the view rotation.
- **`DoomController` name is deliberate** -- reserves `FirstPersonController`/
  `FpsController` for a future, more capable controller (per user steer).
- **Lessons applied** (`13_glide`/`14_breach` retros): keep the rendering/behaviour-
  driving math pure and unit-tested; re-verify `14_breach` gameplay via its autopilot
  (kills + no errors), not just that it compiles -- the autopilot's forced `.hold`
  can't prove the game's own transitions, but it CAN confirm the controller still
  drives movement/aim/fire.

Harvest-after-proof: the working `14_breach` controller exists (`20260705-103236`),
so this promotes a proven shape.

### Work log (implementation)

- Created `src/physics/doom_controller.rs` (Config/Input/State/Output + `DoomEye`
  marker + `doom_move_dir` + `DoomControllerPlugin`, output-only, no avian dep) with
  6 tests (pure move-dir + an App-based drive/clamp test). Created `src/input/cursor.rs`
  (`grab_cursor`/`release_cursor`). Wired both preludes.
- Rewired `examples/14_breach.rs`: player body carries `DoomController`, eye child
  `DoomEye`; `feed_look`/`feed_move`/`apply_move_velocity` replace the game-local look/
  move systems; `capture_cursor`/`free_cursor` call the crate helper (keeping the
  `headless()` guard); autopilot sets `DoomControllerState.yaw`; move_dir/pitch tests
  moved into the module.
- Verified: `cargo clippy --all-targets` clean (plain + `--features debug`); `cargo
  test` = 102 lib + 54 doctests pass (incl. the module's tests + doctest); `check-ascii`
  + `cargo fmt --check` clean; the rewired `14_breach` autopilot runs Menu->Playing->
  GameOver "cycle complete, no panic" with 6 kills (same as before -- the controller
  still aims/moves/fires). Naming `DoomController` reserves the premium
  `FirstPersonController`/`FpsController` name (user steer).
