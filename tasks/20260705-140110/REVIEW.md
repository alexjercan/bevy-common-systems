# Review: 13_glide auto-solver

- TASK: 20260705-140110
- BRANCH: feature/glide-solver

## Round 1

- VERDICT: APPROVE

Scope reviewed: `git diff master...feature/glide-solver` -- the solver logic
(`score_grid`/`transpose`/`line_*`/`expectimax`/`chance_value`/`best_move`), the
ECS glue (`Solver` resource, `solver_step`, `update_auto_indicator`, HUD slot,
`start_run` reset), the autopilot rewrite, docs and tests.

Independently verified:
- `cargo fmt --check`, `cargo clippy --all-targets`,
  `cargo clippy --all-targets --features debug` all clean.
- `cargo test --example 13_glide` -- 15 passed, including
  `solver_climbs_toward_2048_over_a_full_game`, which drives a full deterministic
  game and asserts the solver reaches the `WIN_VALUE` (2048) tile. This is the
  key claim ("reaches 2048") and it is proven off the ECS, exactly as the
  crate's retro rules demand for decision-returning functions.
- Boot under `BCS_AUTOPILOT` reaches the render loop and completes
  Menu -> Playing -> GameOver -> "cycle complete, no panic"; a live window grab
  shows the green "AUTO" HUD tag and a persisted `Best: 7292` (a score only
  reachable by climbing to 1024/2048-class tiles), confirming the solver really
  drives moves.

Correctness spot-checks that passed: expectimax terminates (depth decrements at
every player node; the full-board `chance_value` -> `expectimax` same-depth hop
is followed by a decrementing player node or a `score_grid` leaf); the
`anim.active` lock plus chain order (`player_move` before `solver_step`) means a
same-frame human move wins and the solver never double-moves; the autopilot
toggle is idempotent (presses Space only while `Solver.enabled` is false).

Design is consistent with the example's existing structure: the solver produces a
`Direction` and feeds the same `apply_move`/`start_move` path a human move does,
so animation/scoring/sound/win-lose bookkeeping is shared. Kept game-local, which
matches the crate's "wait for a second user before harvesting" rule; the decision
note (`docs/2026-07-05-glide-solver.md`) records this.

Non-blocking observations (implementer's discretion):

- [x] R1.1 (NIT) examples/13_glide.rs:1172-1180 - the interval clock is not
  advanced while `anim.active`, so the real cadence between solver moves is
  `MOVE_DURATION + SOLVER_INTERVAL` (~0.43s), not the 0.32s the module doc /
  `SOLVER_INTERVAL` comment imply ("one move every third of a second"). Behaviour
  is fine and still comfortably watchable; only the stated number is slightly
  off. Either tweak the wording or decrement the timer during the animation if
  exact pacing matters.
  - Response: Fixed. `solver_step` now advances the timer every enabled frame
    and gates firing on `timer <= 0.0 || anim.active`; since the interval
    (0.32s) is longer than a slide (0.11s) the timer always expires after the
    move finishes, so the true cadence is SOLVER_INTERVAL. Comment added
    explaining it. Verified: tests + autopilot cycle still green.
- [x] R1.2 (NIT) examples/13_glide.rs:515-534 - `chance_value` samples the first
  6 empty cells in row-major order (`empties.iter().take(6)`), a fixed top-left
  subset rather than a random sample, so on sparse boards the spawn expectation
  is systematically biased. It demonstrably still reaches 2048, so this is only a
  play-strength nicety; a comment noting the bias (or sampling spread cells)
  would document the intent.
  - Response: Fixed. Now strides across the empties
    (`step_by(len.div_ceil(CHANCE_SAMPLE))`) so the sample spreads over the
    board, and averages by the actual count visited. `CHANCE_SAMPLE` named
    constant + comment added. Sim test still reaches 2048.
