# Retro: radial gravity - documented recipe (negative result)

- TASK: 20260704-161522
- BRANCH: feat/radial-gravity-recipe (squash-merged to master as 23f733c)
- REVIEW ROUNDS: 1 (APPROVE, no findings)

Second Wave B item, and the first cycle to land a deliberate *negative result*:
no new module, a documented recipe instead.

## What went well

- The sketch killed the module before a line of it was written. Reading the
  actual call sites (not just the task's summary) showed the evidence was
  weaker than it looked: `10_asteroids` doesn't do radial gravity at all -- it
  just sets `Gravity(Vec3::ZERO)` for a flat game -- so there is exactly ONE
  real user (`08_dropzone`), and that user fuses gravity with wind in a single
  `ConstantLinearAcceleration` write. A `RadialGravity` component owning that
  channel would have fought the wind term. One line of math + one real,
  fusion-constrained user is a recipe, not a module.
- Made the recipe carry its weight anyway: the direction math is a compiling,
  asserting doctest (so the task's "unit-test the direction math" is honoured
  even without a module), the avian wiring snippet is `ignore` (references
  game-specific types), and the change gave the previously doc-less `physics`
  module its module-level doc. A negative result still improved the crate.
- Verified every doc claim against `08`'s real code before committing (line
  numbers for the ZERO-gravity resource, the fused acceleration write, the
  world- vs local-space channels, the worked-example function name). The review
  re-checked them and found them accurate -- no "documentation drift" born on
  day one.

## What went wrong

- Nothing. The task explicitly pre-authorised the recipe outcome, so the only
  risk was talking myself into a module out of a sense that "a task should
  produce code"; reading the call sites first is what avoided that.

## What to improve next time

- On any "component OR documented recipe" task, count the *genuine* users from
  the code before deciding, and check whether the idiom is entangled with
  game-specific state at its one real site. Here the count (1, not 2) and the
  gravity+wind fusion were the whole decision; the task's own phrasing ("only 2
  games") over-counted. Trust the grep over the brief.
- A negative result is still a deliverable: leave the crate better (module doc,
  a testable snippet, a pointer to the worked example) rather than just writing
  "we decided not to".

## Action items

- [x] Radial gravity documented as a recipe in `src/physics/mod.rs`; no module
  shipped (deliberate negative result).
- [ ] Last Wave B item: `progress` difficulty-ramp helper (tatr 20260704-161526),
  also sketch-then-commit and, per the spike, the likeliest of the three to also
  land as a doc rather than a module. After it, Wave B is done and the
  higher-priority dev-harness spike (tatr 20260704-175421+) is next by priority.
