# 13_glide auto-solver (2026-07-05)

## What changed

`examples/13_glide.rs` gained a built-in auto-solver: press Space during a run
and an AI plays the board toward the 2048 tile on its own, one move every
`SOLVER_INTERVAL` (0.32s) so each slide is easy to follow. Press Space again to
take back over. A HUD "AUTO" tag (a new centre slot in the score row) lights up
while it is driving.

The feature is entirely additive and reuses the example's existing pure move
logic. New pieces:

- Pure decision logic (off the ECS, unit-tested):
  - `score_grid(grid) -> f32` -- a static board heuristic blending empty-cell
    count, row/column monotonicity, neighbour smoothness and a
    largest-tile-in-a-corner bonus, all in log2 space.
  - `expectimax` / `chance_value` -- a shallow expectimax that averages over the
    random tile the game spawns after each move.
  - `best_move(grid) -> Option<Direction>` (and `best_move_with_depth`) -- picks
    the highest-value legal direction; returns `None` only when the board is
    locked (same condition as game over).
- ECS glue:
  - a `Solver { enabled, timer }` resource, reset in `start_run`;
  - `solver_step` -- toggles on Space, and while enabled plays one `best_move`
    every interval, sharing the `anim.active` lock with `player_move` and
    driving the same `start_move` path so slides/merges/popups animate normally;
  - `update_auto_indicator` -- reflects `Solver.enabled` into the HUD tag.

## Why these decisions

- **Reuse `apply_move` / `start_move`, do not fork a headless move path.** The
  solver produces a `Direction` and feeds it through the exact code a human
  swipe uses. This keeps the animation, scoring, sound and win/lose bookkeeping
  identical whether a human or the solver moved, and means the solver cannot
  drift out of sync with the game rules.

- **Expectimax over the random spawn, not plain greedy or minimax.** 2048's only
  adversary is the random tile placement, so a chance node (expectimax) models
  it correctly; there is no minimising opponent to justify minimax. A depth-3
  search reliably reaches 2048 while staying far inside the per-move time
  budget (one move per 0.32s vs a sub-10ms search).

- **Heuristic choice.** The empty/monotonicity/smoothness/corner blend is the
  well-worn effective 2048 heuristic: keep the board open, keep values ordered
  so the biggest tile stays cornered, and keep neighbours mergeable. Scoring in
  log2 space stops a lone 1024 from being drowned out by a scatter of 2s.

- **Pace it, do not run flat out.** The whole point of the request is a *visible*
  solver, so `solver_step` gates on a timer (`SOLVER_INTERVAL`), not per-frame.
  0.32s comfortably clears the 0.11s slide + pop so moves never overlap.

- **Share the `anim.active` lock and run after `player_move`.** A same-frame
  human move wins (it sets `anim.active` first in the chain); the solver simply
  waits. No new locking scheme was needed.

- **Toggle via `Solver.enabled`, read back in the autopilot.** The headless
  `AutopilotPlugin` closure now enables the solver once (presses Space only
  while `Solver.enabled` is still false, releases once true) instead of pressing
  random arrows, so a `BCS_AUTOPILOT` run exercises the real `best_move` path.

## Testing

Per the crate's retro rule that a function returning a *decision* must be tested
directly (and the specific 13_glide lesson that a state-entry screenshot /
forced-state autopilot does not verify game-driven logic), the solver is proven
by pure unit tests, not by the harness:

- `best_move_never_returns_a_stuck_direction` -- on any board with a legal move,
  the returned direction actually changes the board.
- `best_move_is_none_on_a_locked_board` -- `None` exactly when game over.
- `best_move_takes_the_free_merge` -- given a single matching pair, the chosen
  move merges it (result max tile is 4).
- `solver_climbs_toward_2048_over_a_full_game` -- drives a whole game against a
  deterministic LCG spawner and asserts the solver reaches the 2048 goal tile.
  It runs at depth 2 (not the shipped depth 3) purely so the several-hundred-
  move game stays fast in CI; depth 3 in-game only climbs higher.

## Difficulties / notes

- **Search cost in the simulation test.** A full game is hundreds of moves, and
  a depth-3 search evaluates ~110k boards per move, so a depth-3 full-game test
  would be multi-second. Parametrising the search depth (`best_move_with_depth`)
  let the test run depth 2 (a few thousand board evals per move) while the game
  ships depth 3. Depth 2 still reaches 2048 with the test seed, so the cheaper
  test is not a weaker claim.

- **Autopilot toggle must fire exactly once.** The old closure did
  `keys.reset_all()` + `press` every frame; doing that with a *toggle* key would
  flip the solver on and off every frame. Reading `Solver.enabled` back and only
  pressing while still off makes the press idempotent.

## Possible follow-ups

- The solver is game-local by design (it is tightly coupled to this example's
  `Grid` / `Direction` / `apply_move`). If a second grid-puzzle example ever
  appears, the expectimax skeleton could be considered for harvest, but there is
  no second user today, so it stays in the example (matching the crate's
  "wait for a second user" harvest rule).
