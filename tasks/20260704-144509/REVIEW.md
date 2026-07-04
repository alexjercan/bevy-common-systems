# Review: port 07/08/10 onto camera/shake

- TASK: 20260704-144509
- BRANCH: feat/camera-shake-ports

## Round 1

- VERDICT: APPROVE

Independent review verified all six dimensions with no findings:

- Behavior preservation: every trauma add/reset site in 07/08/10 maps to an
  `input.add_trauma +=` / `input.reset = true` with byte-identical constants
  (HAZARD_TRAUMA, LAND/CRASH_TRAUMA, SHAKE_SPLIT/HIT); the old `.min(1.0)`
  clamps correctly drop because the plugin clamps; `on_player_died`'s absolute
  `trauma = 1.0` -> `add_trauma += 1.0` is equivalent under clamping. The
  module's exponent-2.0 default matches the old `trauma * trauma`, and its
  linear decay matches the old formula.
- `max_offset` axes match per file: 07/08 `Vec3::splat` (old full 3D offset),
  10 `Vec3::new(x, x, 0.0)` (old x/y-only, z owned by fit_camera).
- 10_asteroids fit_camera reschedule correct: moved from Update to PostUpdate
  ordered `.after(Restore).before(Apply)`, removed from Update, z still owned by
  fit_camera, shake only x/y, no drift -- exactly the case the module's
  `composes_with_a_moving_base_driver` test covers.
- No borrow conflicts (the ported systems query the camera's CameraShakeInput
  and gameplay entities' components, never the camera Transform twice).
- MainCamera retained in all three (used by chase input / projection /
  fit_camera).
- No dead code, orphaned consts, or leftover references; clippy --all-targets
  clean; all three examples build and boot.
