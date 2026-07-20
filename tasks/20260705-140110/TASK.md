# 13_glide: auto-solver that plays toward 2048 at a human-visible pace

- STATUS: CLOSED
- PRIORITY: 70
- TAGS: feature,example,glide,solver,historical

## Goal

Add an auto-solver to `examples/13_glide.rs` (the 2048-style Glide puzzle) that
plays the game on its own, choosing moves that push toward the 2048 tile. The
solver must be toggleable at runtime and paced so a human can watch each move
land (one move every few tenths of a second, not one per frame). It reuses the
example's existing pure move logic (`apply_move`, `is_game_over`) and drives the
same `start_move` animation path a human move would, so slides/merges/popups all
animate normally.

## Design

- Pure decision function, kept off the ECS and unit-tested (per the retro rule
  that a function returning a decision must be tested directly):
  `best_move(grid: &Grid) -> Option<Direction>`. Score each of the four legal
  results with a board heuristic and pick the best; return `None` only when no
  move changes the board (game over).
- Heuristic `score_grid(grid) -> f32`: a weighted blend of empty-cell count,
  row/column monotonicity, adjacent-tile smoothness, and a max-tile / keep-the-
  biggest-in-a-corner bonus. Use a shallow expectimax (depth over the random
  tile spawn) so it reliably climbs to 2048; budget is ample at one move / 0.3s.
- Runtime: a `Solver { enabled, timer }` resource; a toggle key while Playing
  (Space) flips `enabled` and plays the select sfx. A `solver_step` system runs
  in the Playing gameplay chain: when `enabled`, not `anim.active`, and its
  interval has elapsed, it calls `best_move` and drives `start_move` exactly as
  `player_move` does. `SOLVER_INTERVAL` ~= 0.3s so moves are visible.
- HUD: an "AUTO" indicator that shows when the solver is on.
- Reset `Solver` on run start.

## Steps

- [x] Add `score_grid` + `best_move` (with shallow expectimax) as pure functions
      near the other move logic; document them.
- [x] Add a `Solver` resource, `SOLVER_INTERVAL`, the Space toggle, and the
      `solver_step` system wired into the Playing chain (before `tick_move_anim`,
      sharing the `anim.active` lock so it never races a move in flight).
- [x] Add the "AUTO" HUD indicator reflecting `Solver.enabled`; reset the solver
      in `start_run`.
- [x] Unit-test the pure logic: `best_move` returns a legal changing move on a
      solvable board, returns `None` on a locked board, prefers the obvious
      merge/corner in a hand-built case, and -- in a headless simulation loop
      driving `apply_move` + a fixed-seed spawn -- reaches at least a high tile
      (e.g. 512+) to prove it actually progresses toward 2048.
- [x] Update the autopilot input closure to toggle the solver ON (Space) rather
      than pressing random arrows, so a `BCS_AUTOPILOT` run visually exercises
      the real solver path.
- [x] Update the module `//!` doc / controls list to mention the Space auto-play
      toggle; note the solver in the AGENTS.md `13_glide` entry.
- [x] Verify: `cargo fmt --check`, `cargo clippy --all-targets`,
      `cargo clippy --all-targets --features debug`, `cargo test`,
      `cargo test --examples`, plain `cargo build --example 13_glide`,
      `./scripts/check-ascii.sh`, and a `timeout` boot/run of the example.
