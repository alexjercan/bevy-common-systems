# Retro: Fruit ninja bombs, health and lose condition

- TASK: 20260703-121347
- BRANCH: feature/fruitninja-bombs (merged to master, deleted)
- REVIEW ROUNDS: 1 (APPROVE, R1.1 naming fixed in-round, R1.2 accepted)

See TASK.md close-out for what shipped; this is process only.

## What went well

- Asking the user up front (via the two design-fork questions: bomb = instant
  loss vs damage, and what else drains health) turned a genuinely ambiguous
  request into a crisp spec before any code. That is exactly the "ask at the
  cheapest moment" the flow prescribes; it meant the health task did not get
  built twice.
- Reusing the crate's `HealthPlugin` end to end (Health -> HealthApplyDamage ->
  HealthZeroMarker -> observer) is the example doing its actual job:
  demonstrating the crate. The `on_damage` guards made repeated lethal triggers
  in one swipe idempotent for free, so no extra "already dead" bookkeeping.
- The shared `Sliceable`/`Projectile` + `Bomb` marker refactor kept the slice
  loop a single code path for fruit and bombs (branch only on `Has<Bomb>`),
  instead of duplicating the swipe-segment logic. Refactoring the model before
  adding the feature paid off.
- The throwaway-boot verification kept scaling: an auto-slicer plus a
  state-change logger proved the full Playing -> sliced-bomb -> GameOver chain
  on real hardware, which no compile check or unit test could show.

## What went wrong

- Process slip: I started task 3 without creating the feature branch and
  committed the work directly on master. Root cause: the previous task ended
  with a `git checkout master` after merging, and I set the task IN_PROGRESS
  and dove into code without the `git checkout -b` step that the earlier tasks
  had. Caught it at merge time and restructured the history (branch at the new
  commits, reset master, `merge --no-ff`) so the trail matches the other two
  tasks. No work lost, but the isolation the flow wants was momentarily absent.

## What to improve next time

- Make "create the feature branch" the very first action when a task moves to
  IN_PROGRESS, before touching any code - ideally in the same step that flips
  the status. The tell was flipping STATUS to IN_PROGRESS while still on
  master; that pairing should trigger the branch creation every time.

## Action items

- [x] Bombs + health + lose shipped, reviewed to APPROVE, merged.
- [ ] Process note (not a code task): when the flow loop starts a task, branch
  first. Considered adding this to the flow skill, but it already says "sprout
  worktree + branch"; the miss was mine, not the skill's, so no doc change.
