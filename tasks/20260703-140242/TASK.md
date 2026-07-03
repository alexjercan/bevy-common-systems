# Fruit ninja: high score across runs

- STATUS: OPEN
- PRIORITY: 80
- TAGS: feature,example

## Goal

Track the best score across runs in a session and surface it: show "Best: N"
on the menu and a "New best!" flourish on the game-over screen when the run
beats it. Adds replay pull.

## Steps

- [ ] Add a `HighScore(usize)` resource (`init_resource`), persisting for the
      whole app run (not state-scoped).
- [ ] On entering `GameOver` (or in `on_player_died` / the game-over spawn),
      update `HighScore` to `max(HighScore, Score)` and remember whether this
      run set a new best.
- [ ] `spawn_menu`: add a "Best: N" line reading `HighScore`.
- [ ] `spawn_game_over`: show the final score and the high score, and a "New
      best!" line when the run beat the previous best.
- [ ] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
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
