# Retro: 14_breach points + combos

- TASK: 20260705-132200
- BRANCH: feat/breach-combos (squash-merged to master as c7fcff8)
- REVIEW ROUNDS: 1 (APPROVE)

Part of the breach fun-pass flow (spike
`tasks/20260705-132024/SPIKE.md`). This is about how the work went.

## What went well

- The spike had already picked the building blocks (`scoring/streak` for the decay,
  `ui/popup` for the "+N"), so the task was pure wiring with no design risk. Reading
  `06_fruitninja`'s `Combo`/`tick_combo` first meant the streak-hit / lapse-tally
  pattern transferred almost verbatim; the crate module owning only count+decay (game
  owns the value rule) is exactly the seam that made it reusable in 3D.
- Kept the death observer UI-free by routing kills through a `KillFeed(Vec<u32>)`
  buffer that a Playing system drains into popups. This preserved the existing
  "on_health_zero is headlessly testable" property the code comment prizes, and let
  the streak-scaling be unit-tested off the ECS (`chained_kills_multiply_by_the_streak`)
  rather than trusted to the autopilot -- the exact lesson from the breach lose-condition
  retro applied pre-emptively.
- Verified the whole path headlessly the honest way: the persisted HighScore jumped to
  280 (combo-scaled points), which only happens if kill -> streak.hit -> points really
  fires. A pass/fail autopilot transition alone would not have proven scoring.

## What went wrong

- Nothing broke, but two near-misses were caught by checking rather than luck:
  - `PopupPlugin` was not yet in breach; before adding it I confirmed it self-adds
    `TweenPlugin` (the fade/despawn rides a `Tween<f32>`), so popups do not leak. Had
    I assumed it was standalone, popups could have hung on screen forever.
  - Changing `Score(u32)` -> `Score { points, kills }` is the kind of edit that leaves
    stale `.0` references; a grep confirmed none remained instead of leaning on the
    compiler alone (the example half of the build is the easy thing to miss).
- Master moved twice during the single task (a README expansion and new tasks landed),
  so the flow merge-master-into-branch step was not a no-op -- a reminder that even a
  short task can race the default branch in this repo.

## What to improve next time

- When wiring an existing crate plugin into an example for the first time, check its
  `build()` for self-added dependency plugins (like `PopupPlugin` -> `TweenPlugin`)
  up front, so ordering/lifecycle assumptions are grounded, not guessed.
- The remaining fun-pass tasks (pickups, enemy archetypes) touch the same
  `on_health_zero` observer and `spawn_wave`; keep the observer logic-only and push any
  new per-kill side effects through buffers/resources so the headless tests stay valid.

## Action items

- None blocking. Carry the "check plugin build() for self-added deps" habit into the
  juice-pass task, which adds more `ui/popup` / `camera/*` surface.
