# Fruit ninja: high score across runs

- STATUS: CLOSED
- PRIORITY: 80
- TAGS: feature,example

## Goal

Track the best score across runs in a session and surface it: show "Best: N"
on the menu and a "New best!" flourish on the game-over screen when the run
beats it. Adds replay pull.

## Steps

- [x] Add a `HighScore(usize)` resource (`init_resource`), persisting for the
      whole app run (not state-scoped).
- [x] On entering `GameOver` (or in `on_player_died` / the game-over spawn),
      update `HighScore` to `max(HighScore, Score)` and remember whether this
      run set a new best.
- [x] `spawn_menu`: add a "Best: N" line reading `HighScore`.
- [x] `spawn_game_over`: show the final score and the high score, and a "New
      best!" line when the run beat the previous best.
- [x] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, real boot with no panic.

## Notes

- `spawn_menu` at :451, `spawn_game_over` at :522 (reads `Score`),
  `on_player_died` at :356.
- Default to in-memory (session-only) high score. Optional follow-up, no new
  deps: persist to a file with `std::fs` (e.g. a dotfile) and load it at
  startup -- only do this if the user asks; note it in the close-out either way.
- Compute the "new best" flag before overwriting `HighScore`, so the game-over
  screen can show it.
- No new dependencies.

## Close-out

Added session `HighScore` + `NewBest` resources. `record_high_score` (chained
before `spawn_game_over` on GameOver enter) flags a new best and updates the
best. Menu shows "Best: N"; game-over shows "New best!" or "Best: N". High
score is in-memory (session only); file persistence via std::fs left as an
optional follow-up (no new deps), not done.
