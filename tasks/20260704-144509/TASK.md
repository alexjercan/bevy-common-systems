# camera/shake: port 07/08/10 onto the module, delete local copies

- STATUS: OPEN
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

- [ ] Port `07_orbit` onto `CameraShakePlugin`: add the plugin, put `CameraShake`
      on the camera, route hazard-hit trauma through `CameraShakeInput`, reset on
      new game, delete the local resource/system/consts and `MainCamera` if it
      becomes unused. Chase composition is automatic (Apply is ordered after
      `ChaseCameraSystems::Sync`).
- [ ] Port `08_dropzone` the same way (land/crash trauma).
- [ ] Port `10_asteroids`: this camera's base is written by `fit_camera` in
      `Update`, not by chase in `PostUpdate`. Confirm the Restore/Apply ordering
      still yields no drift with an Update-schedule base writer, or move
      `fit_camera` so it sits between Restore and Apply; add a note if the module
      needs an ordering hook for non-chase drivers.
- [ ] Verify: full check suite + boot each ported example to the render loop.
      Net line count should drop across the three files.

## Note

`10_asteroids` is the interesting one: the module orders `Apply` after
`ChaseCameraSystems::Sync` (a PostUpdate set), but `fit_camera` runs in `Update`.
Since Restore runs in PostUpdate too, an Update-schedule base writer runs before
Restore, so Restore would wrongly subtract the last offset from the freshly
fitted base. Porting 10 may require either running the shake's Restore earlier
(a pre-driver schedule) or documenting that non-chase base drivers must run
between the Restore and Apply sets. Decide during planning.
