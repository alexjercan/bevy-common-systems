# Retro: 14_breach ground pickups

- TASK: 20260705-132238
- BRANCH: feat/breach-pickups (squash-merged to master as b250a20)
- REVIEW ROUNDS: 1 (one self-caught test fix before green)

Part of the breach fun-pass flow (spike
`tasks/20260705-132024/SPIKE.md`).

## What went well

- The combos task had already established the pattern for this exact shape: keep the
  death observer UI-free by pushing to a buffer resource (`KillFeed` there,
  `PickupDrops` here) and let a Playing system own the meshes/UI. Reusing that seam made
  the drop-on-kill path drop in cleanly and kept `on_health_zero` headlessly testable.
- Choosing existing integration points instead of new machinery: the speed buff just
  scales `DoomController.move_speed`, and the fire-rate buff ticks the existing
  `Gun` cooldown faster (`dt * scale`) rather than mutating a duration. Both are
  one-liners that compose with the crate controller and `time/cooldown` as-is.
- Extracting `apply_pickup` / `decay_buffs` / the buff multipliers as pure functions
  made the interesting logic testable off the ECS (heal caps at max, timers set/decay),
  exactly the lesson carried from the combos and lose-condition retros.

## What went wrong

- The first `tick_buffs` test drove a `MinimalPlugins` App and advanced `Time` by hand,
  but `TimePlugin` overwrites the manual delta with the (near-zero) real elapsed between
  updates, so the buff never decayed and the test failed (11/12). The `ui/popup` tests
  get away with hand-advancing `Time` because they assert coarse tween progress, not an
  exact dt. Fix: extract the pure `decay_buffs(&mut Buffs, dt)` and test that directly.
  Net: the flaky ECS-time test became a deterministic pure test -- strictly better, and
  it cost one extra compile cycle to learn.

## What to improve next time

- Do not unit-test frame-rate-dependent decay through a `MinimalPlugins` App and a
  manual `Time::advance_by`; `TimePlugin` clobbers the delta. Make the per-dt step a
  pure function and test that. Only reach for an App+Time test when asserting something
  coarse (a tween crossed a threshold), never an exact post-dt value.
- The pattern "observer buffers, Playing system spawns, effect is a pure fn" is now
  proven twice (combos, pickups). The enemy-archetypes task touches the same
  `spawn_wave` / observer; keep enemy stat selection a pure fn over a data table so the
  wave-composition is testable the same way.

## Action items

- None blocking. The `Sfx::Pickup` cue reuses the `golden` placeholder; the dedicated
  sound pass (`tasks/20260705-132244`) will give it its own sound.
