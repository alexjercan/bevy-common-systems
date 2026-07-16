# Harvesting 14_breach's first-person controller into the crate

- TASK: tasks/20260705-103238 (spike / harvest-after-proof)
- SPIKE INPUT: tasks/20260705-103116/SPIKE.md
- REFERENCE: examples/14_breach.rs

`14_breach` is the crate's first first-person game. It built a game-local
first-person controller because `camera/wasd` is a free-fly spectator camera (owns
translation, no gravity/ground/collision/cursor-grab). This note records which of
those pieces generalized into the crate and which stayed local.

## What shipped: `physics/doom_controller`

A new avian-adjacent module: a **Doom-style** first-person character controller.

- `DoomController` (config): `move_speed`, `look_sensitivity`, `pitch_min/max`.
  `#[require(...)]` pulls in its Input/State/Output companions.
- `DoomControllerInput { look: Vec2, movement: Vec2 }` -- written by the game each
  frame (mouse/stick look delta; strafe/forward intent).
- `DoomControllerState { yaw, pitch }` -- accumulated orientation. Public, so a game
  can set it directly to face a direction on spawn or aim under a test/AI (14_breach's
  autopilot sets `yaw` to aim at the nearest enemy).
- `DoomControllerOutput { velocity: Vec3 }` -- the desired horizontal velocity.
- `DoomEye` -- a marker for the eye camera child; the plugin writes its rotation from
  the parent's yaw/pitch.
- `doom_move_dir(yaw, movement)` -- the pure move-basis helper (unit-tested).

The plugin integrates the look delta into yaw/pitch (clamping pitch), orients the
`DoomEye` child, and sets the velocity output -- in the `DoomControllerSystems::Drive`
set. `14_breach` now spawns `DoomController` + a `DoomEye` camera child, feeds
`Input`, and copies `Output.velocity` into its `LinearVelocity`; its `move_dir`,
`clamp_pitch`, `player_look`, `player_move` (and the `FirstPersonController`/
`PlayerCamera` components) are gone, their tests moved into the module.

### Why the name is `DoomController`, not `FirstPersonController`

Deliberate, at the user's steer. This is the pragmatic arena-shooter controller:
flat ground, no jump / crouch / air-control / slope handling. Claiming the premium
`FirstPersonController` / `FpsController` name now would box in a more capable
controller built later. Naming it `Doom` sets expectations (Quake/Doom-simple) and
reserves the good name.

### Why output-only, and no avian dependency

The crate's body drivers (`pd_controller`, the `transform` family) are all
output-only: they compute an `Output` the game applies, rather than writing physics
components themselves. `doom_controller` follows suit -- `Output.velocity` is a plain
`Vec3`, and the game writes it into `LinearVelocity` (leaving `.y` to gravity so
avian's solver does collide-and-slide). So the module takes **no avian dependency at
all**, staying composable and testable with `MinimalPlugins`. It lives in `physics/`
for discoverability (it is a character controller), not because it touches avian.

### The two requirements it documents

The design only works if (1) the body keeps its rotation locked
(`LockedAxes::ROTATION_LOCKED`) and stays axis-aligned -- yaw lives in the state, not
on the body Transform, so the physics solver never fights the camera -- and (2) the
view rotation lives on a `DoomEye` child at eye height. Both are stated in the module
doc; a body that rotates would double-apply yaw.

## What shipped: `input/cursor`

Two free functions -- `grab_cursor(&mut CursorOptions)` (lock + hide) and
`release_cursor(&mut CursorOptions)` (free + show) -- over Bevy 0.19's `CursorOptions`
window component. Small, but non-obvious: 0.19 moved cursor state off `Window` onto a
per-window component, which a future session would otherwise rediscover. The *policy*
(when to grab, and 14_breach's `headless()` guard that skips the grab under the test
harness) stays game-local; the crate provides only the primitive.

## What stayed game-local

- **The input wiring** (`AccumulatedMouseMotion`, `ButtonInput`, the touch sticks,
  the velocity write into `LinearVelocity`). Per the crate's Input-component contract
  (`camera/wasd`), the module consumes a plain `Input` component and the game owns
  where the numbers come from and where the output goes.
- **The gun (hitscan), waves, enemies, HUD, arena** -- game logic, not controller.

## Verdict per candidate

- `physics/doom_controller` -- **PROMOTED.** A clean, generalizable shape (standard FP
  look + planar move), output-only, reusing the crate's driver conventions. First user
  is 14_breach; the shape is well-understood enough to promote (unlike a bespoke
  format).
- `input/cursor` grab helper -- **PROMOTED.** Tiny, non-obvious, genuinely reusable.
- Extend `camera/wasd` with a grounded mode -- **DROPPED.** It owns translation
  (free-fly), so it cannot participate in avian collide-and-slide; bolting a physics
  body onto a transform-owning camera is a category error.
- Add a pitch clamp to `transform/point_rotation` -- **DROPPED.** `point_rotation`
  accumulates two-axis deltas on a tumbling local basis (no clean pitch to clamp) and
  `smooth_look_rotation` is single-axis + smoothed; neither is an FP euler-look. The
  controller owns its own clamp.
- A hitscan / `SpatialQuery` helper -- **DROPPED.** The raycast-and-damage is ~6 lines
  of game-specific logic (layer mask, what counts as a hit); below the bar.
