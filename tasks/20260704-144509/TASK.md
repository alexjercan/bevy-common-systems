# camera/shake: port 07/08/10 onto the module, delete local copies

- STATUS: CLOSED
- PRIORITY: 38
- TAGS: feature,camera,cleanup


> Follow-up from tasks/20260704-134500 (camera/shake module). See
> docs/2026-07-04-camera-shake-module.md and
> docs/retros/20260704-134500-camera-shake-module.md.

## Goal

The `camera/shake` module (CameraShakePlugin) now owns trauma camera shake, and
`06_fruitninja` was refactored onto it as the proof-of-use. Three more example
games still hand-roll their own shake and should be ported so the crate stops
duplicating it:

- `examples/07_orbit.rs` -- chase camera; local `CameraShake` resource +
  `apply_camera_shake` ordered `.after(ChaseCameraSystems::Sync)`.
- `examples/08_dropzone.rs` -- chase camera; same shape, `+= offset` after Sync.
- `examples/10_asteroids.rs` -- static/fit camera; `apply_camera_shake` after
  `fit_camera`, writes offset absolutely on x/y.

## Steps

- [x] Port `07_orbit` onto `CameraShakePlugin`: add the plugin, put `CameraShake`
      on the camera, route hazard-hit trauma through `CameraShakeInput`, reset on
      new game, delete the local resource/system/consts and `MainCamera` if it
      becomes unused. Chase composition is automatic (Apply is ordered after
      `ChaseCameraSystems::Sync`).
- [x] Port `08_dropzone` the same way (land/crash trauma).
- [x] Port `10_asteroids`: this camera's base is written by `fit_camera` in
      `Update`, not by chase in `PostUpdate`. Confirm the Restore/Apply ordering
      still yields no drift with an Update-schedule base writer, or move
      `fit_camera` so it sits between Restore and Apply; add a note if the module
      needs an ordering hook for non-chase drivers.
- [x] Verify: full check suite + boot each ported example to the render loop.
      Net line count should drop across the three files.

## Note

`10_asteroids` is the interesting one: the module orders `Apply` after
`ChaseCameraSystems::Sync` (a PostUpdate set), but `fit_camera` runs in `Update`.
Since Restore runs in PostUpdate too, an Update-schedule base writer runs before
Restore, so Restore would wrongly subtract the last offset from the freshly
fitted base. Porting 10 may require either running the shake's Restore earlier
(a pre-driver schedule) or documenting that non-chase base drivers must run
between the Restore and Apply sets. Decide during planning.

## Resolution

Chose the "driver runs between the sets" option (no module change): moved
`fit_camera` from `Update` to `PostUpdate` ordered
`.after(CameraShakeSystems::Restore).before(CameraShakeSystems::Apply)`. This is
the same slot the chase camera's `Sync` set already occupies, so 10 now composes
exactly like 07/08 -- the base is (re)written between the two shake phases. The
module's `composes_with_a_moving_base_driver` test models this exact case, so 10's
correctness is covered by a library test, not just a visual boot.

07 and 08 are chase cameras, so composition was automatic (Apply is already
ordered after `ChaseCameraSystems::Sync`). All three keep their `MainCamera`
marker (used by other queries: chase input, popup/input projection, fit_camera).

Net line reduction: 07 -58, 08 -26, 10 -5 (fit_camera reschedule offsets most of
the deletion), ~90 lines removed across the three examples. All three build,
clippy and boot clean; full test suite green.
