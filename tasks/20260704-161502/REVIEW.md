# Review: camera/project screen<->world projection helpers

- TASK: 20260704-161502
- BRANCH: feat/camera-project

## Round 1

- VERDICT: APPROVE

Clean, faithful harvest. The two helpers live in `camera/project` (the
spike-favored, camera-coupled home), are pure functions with a module doc,
per-item docs and runnable doctests, and are wired through the camera prelude
into `crate::prelude`. Both duplicated projection bodies were reproduced
byte-for-byte inside the new helpers, so the refactor is behavior-preserving:

- `pointer_on_plane` keeps 06's `(0,0,PLAY_Z)` origin and 10's `Vec3::ZERO`
  origin at the call sites; 10's verbatim copy is deleted, 06's local
  `pointer_on_play_plane` is reduced to a thin play-plane wrapper (kept because
  it is called twice and pins the game-specific plane, matching the crate's
  "leave the game-specific config in the game" line).
- `world_to_screen` folds `world_to_viewport(...).ok()` and replaces all three
  callers (06, 07, 08), exceeding the task's "at least one" bar.

Verified independently: `cargo fmt --check`, `cargo clippy --all-targets`
(both default and `--features debug`), `cargo test` (54 unit + 22 doctests),
`cargo test --examples` and `scripts/check-ascii.sh` all pass; 06_fruitninja
boots to the render loop. No remaining raw `viewport_to_world`/
`world_to_viewport` calls in `examples/`.

Signature note: the task sketched `pointer_on_plane(camera, gt, viewport_pos,
plane)`, but `InfinitePlane3d` carries only a normal, not a point, so the helper
correctly takes `plane_origin: Vec3` plus `plane: InfinitePlane3d` to preserve
both games' plane offsets. Reasonable, necessary deviation.

- [x] R1.1 (NIT) src/ui/popup.rs:13 - the module doc still tells users to
  "compute the viewport point yourself (`camera.world_to_viewport(...)`) and
  pass that in". Now that a blessed `world_to_screen` exists (which the spike
  named as the thing this task unblocks for `ui/popup`), that sentence could
  point at `camera::project::world_to_screen` instead. Take it or leave it;
  not required by the task Goal.
  - Response: Addressed. Repointed the popup module doc at
    `[camera::project::world_to_screen]` with an intra-doc link. fmt/doctests/
    ascii re-verified green.
