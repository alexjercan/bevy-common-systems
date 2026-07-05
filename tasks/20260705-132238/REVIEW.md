# Review: breach -- ground pickups

- VERDICT: APPROVE
- ROUNDS: 1 (one self-caught test-harness fix before green)

## Verified

- `cargo fmt --check`, `cargo clippy --all-targets` (clean), `cargo test --example
  14_breach` (12 pass), `scripts/check-ascii.sh` (clean).
- Headless `BCS_AUTOPILOT=1 --features debug`: full cycle, no panic, no despawn-race
  errors -- the spawn -> animate -> collect -> despawn pickup path is safe under the
  swarm.
- Real windowed run reached the render loop.

## Findings / checks

- No correctness issues in the shipped code.
- One test-harness bug caught and fixed during the task, not shipped: the first
  `tick_buffs` test drove a `MinimalPlugins` App and advanced `Time` by hand, but
  `TimePlugin` overwrites the manual delta with (near-zero) real elapsed, so the buff
  did not decay and the test failed. Fixed by extracting the pure `decay_buffs(&mut
  Buffs, dt)` and unit-testing that deterministically -- the same "make the logic pure,
  test that" rule the combos task followed. (Lesson: do not lean on `MinimalPlugins`
  Time for deterministic dt; test the pure function.)
- Retro-guard checks: death observer stays UI/Assets-free (rolls the drop, buffers
  `PickupDrops`); pickup effect is the pure `apply_pickup`, tested (heal caps at max,
  each buff sets its timer); emissive pickup material does NOT set `unlit`, so it blooms;
  pickup is a single-mesh entity, so no B0004.
- Component access audited: `collect_pickups`' `Single<&mut Health>` and
  `Query<&Transform, With<Pickup>>` are disjoint (player has no `Pickup`); the
  pickup-tuple systems touch disjoint components (`DoomController` vs `Gun` vs `Buffs`),
  so no scheduler conflict.

## Nits (non-blocking)

- `Sfx::Pickup` reuses the `golden` placeholder; the dedicated sound pass (task
  20260705-132244) will replace it.
- `apply_speed_buff` writes `DoomController.move_speed` unordered vs the controller's
  Drive read, so a speed change can land one frame late. Imperceptible; not worth a
  pinned edge.
