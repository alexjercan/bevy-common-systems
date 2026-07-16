# Retro: progress difficulty-ramp - documented recipe (negative result)

- TASK: 20260704-161526
- BRANCH: feat/progress-recipe (squash-merged to master as 10fa61c)
- REVIEW ROUNDS: 1 (APPROVE, no findings)

Last Wave B item, and the second consecutive documented-recipe (negative
result). That back-to-back pattern is itself the signal worth recording: Wave B
was correctly scoped by the spike as "sketch-then-commit, may downgrade to a
doc", and two of its three items did exactly that.

## What went well

- Sketched the substantial candidate (a `LevelTimer` state machine with a
  level-up edge, by analogy to the shipped `Streak`) before dismissing it,
  rather than reflex-documenting. That is what made the "doc" verdict honest:
  07 and 11 do implement a level-up edge, but *divergently* (07 recomputes
  `level_for(elapsed)` and compares; 11 uses a `next_level_at` accumulator), so
  it is two independent one-liners, not a copied primitive like `Streak` was.
  Weighing it against `Streak` -- same shape, weaker signal -- gave a principled
  reason to split the two decisions.
- Kept the recipe testable: both idioms (continuous ramp, level-interval timer +
  edge) ship as compiling, asserting doctests in `meth`, and every claim about
  the games was checked against the source before committing. A negative result
  still left the module better documented.

## What went wrong

- A real process slip: after `sprout rm` on the previous task, I wrote the first
  draft of this recipe into the *removed* worktree's path (a stale leftover dir),
  not a fresh sprout. Caught it immediately via `sprout ls` / `git worktree
  list` (the branch was gone, so it was an orphaned dir, not a tracked
  worktree), `rm -rf`'d the leftover, `git worktree prune`d, and started the
  correct `feat/progress-recipe` sprout. No repo damage -- the stray write was in
  an untracked, unregistered directory -- but it cost a detour.
  - Root cause: treated "start the next task" as "keep editing" instead of
    "sprout first". The flow's per-task opening move is *always* sprout a fresh
    worktree; I skipped straight to Write.

## What to improve next time

- Make "sprout a fresh worktree" the unskippable first action of every task,
  before any Read/Write. After a `sprout rm`, the previous worktree path is dead
  -- never edit a path under `.cache/sprouts/<removed-feature>/`. A one-line
  `sprout ls` check before the first edit of a task would have caught it before
  the write, not after.
- For a family of "component OR recipe" tasks, decide each on its own evidence
  (copied vs re-derived, substantial vs one-liner). Two docs in a row is a fine
  outcome when the evidence says so -- do not overcorrect into building a thin
  module just to "balance" the negative results.

## Action items

- [x] Difficulty ramp documented as a recipe in `src/meth/mod.rs`; no module
  shipped. Wave B of the input/projection harvest is complete: `scoring::Streak`
  shipped, radial gravity + progress landed as recipes.
- [ ] Next by priority is the dev-harness spike (tatr 20260704-175421, priority
  80), which a parallel session already has a `dev-harness` worktree open on --
  coordinate before starting to avoid colliding on it.
