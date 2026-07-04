# camera/shake: trauma camera shake module

- DATE: 2026-07-04
- TASK: tasks/20260704-134500
- SPIKE: docs/spikes/20260704-134035-game-juice-and-scaffolding-kit.md (Wave 1)

## What changed

Added `src/camera/shake.rs`, a `CameraShakePlugin` that owns the trauma camera
shake four example games (06, 07, 08, 10) hand-rolled, and refactored
`examples/06_fruitninja.rs` onto it (deleting its local `CameraShake` resource,
`apply_camera_shake` system, and `MainCamera` marker).

The module follows the crate's Config / Input / Output / private State split:

- `CameraShake` (config): `decay`, `max_offset: Vec3`, `max_kick: Vec3`,
  `exponent`.
- `CameraShakeInput` (game writes each frame): `add_trauma` (consumed and
  reset to zero after it is applied) and `reset` (snap trauma to zero).
- `CameraShakeOutput` (game reads): the `offset`/`kick` currently applied.
- `CameraShakeState` (private): trauma plus the last-applied offset/kick.

## Key decisions

### Rotation kick: support both, default to translation-only

The spike left open whether v1 shakes rotation (kick) or translation only.
Resolved by supporting both: `CameraShake` carries `max_offset` and `max_kick`,
with `max_kick` defaulting to `Vec3::ZERO`. Out of the box the shake is
translation-only (matching all four existing example copies), but a game can
opt into a rotational kick by setting `max_kick`. This closes the open question
without shutting the door on kick, and costs almost nothing (a `Quat`
multiply that is identity when the config is zero).

### Restore/Apply two-phase, driver-agnostic (the anti-drift design)

The bug this module exists to prevent (asteroids retro,
`docs/retros/20260703-170744-asteroids-example.md`) is the *accumulating*
shake: `transform.translation += offset` on a camera whose base is not rewritten
each frame piles offsets up and drifts the camera off-center. The four example
copies each dodged this differently -- 06 wrote `= CAMERA_BASE + offset`
(static base const), 07/08 wrote `+= offset` *after* the chase camera rewrote
the base (so the chase overwrite acts as the reset), 10 let `fit_camera` own
the base. There was no single primitive that worked in all cases.

The module unifies them with two systems ordered around any base-writing driver:

- `CameraShakeSystems::Restore` runs *before* the driver and subtracts the
  previous frame's offset (and inverts the previous kick), returning the
  transform to the driver's clean base.
- `CameraShakeSystems::Apply` runs *after* the driver and re-applies a fresh
  offset/kick.

The net is `driver_base + offset` every frame -- an absolute offset, never an
accumulator -- whether the base comes from a chase camera, a custom framing
system, or nothing at all (a static camera, which is just "the driver is a
no-op"). `Apply` is ordered `.after(ChaseCameraSystems::Sync)` and `Restore`
`.before` it, so composing with the chase camera is automatic; the ordering is
a no-op when the chase plugin is absent, so a static-camera game (like 06)
needs no extra wiring.

Why two systems instead of the simpler "store base once and write
`base + offset`": storing the base once cannot follow a chase camera whose base
moves every frame. Why not the single-system "undo my last offset, then apply":
that corrupts a driver that overwrites the transform (chase), because the
driver's fresh value is not "base + my last offset". Splitting Restore (before
drivers) from Apply (after drivers) is the only ordering that is correct for
both a moving driver and a static base.

## Testing

- Pure-math unit tests: decay clamps at 0, trauma-add clamps at 1,
  `amount = trauma^exponent`, zero trauma -> zero offset, offset scales with
  `max_offset`, zero-kick config -> identity rotation.
- ECS integration tests (minimal `App`, `Time` driven by hand):
  - the offset never exceeds the configured bound;
  - the camera re-centers exactly on base after repeated trauma kicks and
    decays -- the direct regression test for the drift bug;
  - `reset` snaps the camera back to base on the next frame.
- `examples/06_fruitninja` boots to the render loop with no panic.

## Follow-ups

07, 08 and 10 still carry their own shake copies; porting them onto the module
(and reaping the net line reduction) is left for a follow-up so this task stays
scoped to the module plus one proof-of-use refactor, as the task specified.
