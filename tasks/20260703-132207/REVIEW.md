# Review: Fruit ninja blade trail

- TASK: 20260703-132207
- BRANCH: feature/fruitninja-blade-trail

## Round 1

- VERDICT: APPROVE

Clean, small addition. `BladeTrail` is a capped `VecDeque` filled from
`slice_objects` while LMB is held and cleared on release; `draw_blade_trail`
renders it with `Gizmos::line` and a tail->head alpha ramp, lifted toward the
camera. The `count < 2` guard makes the `(count - 1)` divisor safe. Verified on
real GPU (with the release-clear temporarily neutralized so a seeded trail
survives): `draw_blade_trail` drew line segments every frame, no panic, no
gizmo errors. Checks clean (`fmt`, `clippy --all-targets` both configs,
`check-ascii`). Two findings, neither blocking.

- [ ] R1.1 (MINOR) examples/06_fruitninja.rs:start_game / BladeTrail - a run
  ends via game-over, which can happen mid-swipe (slicing a bomb while holding
  LMB), leaving points in `BladeTrail`. `start_game` resets `Score`, the spawn
  timer and `CursorTrail.previous`, but not `BladeTrail`, so the next run can
  draw a stale trail for a frame before `slice_objects` clears it (order
  dependent with `draw_blade_trail`). Clear `blade.points` in `start_game` for
  a clean run start, consistent with the other per-run resets.
  - Response: Done - `start_game` now takes `ResMut<BladeTrail>` and clears
    `blade.points`.

- [ ] R1.2 (NIT) examples/06_fruitninja.rs - `BladeTrail.points` uses the inline
  `std::collections::VecDeque` path; `mesh/explode.rs` imports it with `use
  std::collections::VecDeque;`. A top-level `use` would match repo style.
  Optional.
  - Response: Done - added `use std::collections::VecDeque;` and simplified the
    field type.
