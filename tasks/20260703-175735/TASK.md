# CI: run example unit tests (cargo test --examples)

- STATUS: CLOSED
- PRIORITY: 60
- TAGS: ci,tests

## Goal

Make the CI test step actually run the `#[cfg(test)]` tests that live inside
`examples/`. Today `cargo test` builds the examples but does not run their unit
tests, so `examples/06_fruitninja.rs` alone has 19 in-file tests (geometry,
combo math, difficulty ramp, and the new `active_pointer_pos` cases) that never
execute in CI and can silently rot.

## Steps

- [x] Added an "Example tests" step running `cargo test --examples` to
      `.github/workflows/ci.yml` (after the two `cargo test` steps). No
      `--features debug` variant: no example test is `#[cfg(feature = "debug")]`,
      so it adds zero coverage (verified). 58 example tests now run
      (06=19, 07=14, 08=7, 10=9, 11=9); `06_fruitninja` = 19 as expected.
- [x] Updated AGENTS.md "Build, Verify, Run" (new `cargo test --examples` line)
      and the CI-suite sentence.

## Notes

- Surfaced by the retro for task 20260703-173128 (touchscreen support): the 3
  new `active_pointer_pos` tests pass under `cargo test --example
  06_fruitninja` but not under a plain `cargo test`. Pre-existing gap, not
  caused by that task.
- Enabling `--examples` immediately caught a rotted, flaky test in
  `10_asteroids` (`splitting_a_rock_spawns_smaller_physics_bodies`): it asserted
  every shard keeps at least the parent's drift speed. That is false by design:
  the outward burst is a 3D direction but `spawn_asteroid` zeroes z (the arena is
  planar / `LockedAxes`), so a shard whose burst points out of plane can end up
  slower than the parent. The assertion passed or failed depending on the random
  slice geometry. Replaced it with the invariant that always holds -- the planar
  velocity is the inherited parent drift plus a burst no larger than
  `SPLIT_SPEED` (`|vel - drift| <= SPLIT_SPEED`). This is exactly the silent rot
  the task set out to expose. Verified stable over 8 back-to-back runs.
