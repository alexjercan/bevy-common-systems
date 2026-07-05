# Review: breach -- game-over screen has no camera (UI invisible), add one

- TASK: 20260705-154058
- BRANCH: fix/breach-gameover-camera

## Round 1

- VERDICT: APPROVE

Diff is a 12-line, single-purpose fix that delivers the Goal exactly as the task
prescribes: `spawn_game_over` now spawns a `Name("Game Over Camera")` + `Camera2d` +
`DespawnOnExit(GameState::GameOver)`, byte-for-byte the menu's own 2D-camera pattern.

Verified:
- Correctness: during GameOver the Playing `Camera3d` is already despawned
  (`DespawnOnExit(Playing)`), so this is the only camera in that state -- no double-camera
  ambiguity, and it is torn down on exit GameOver so it never leaks into the next state.
- Spec: the required visual check was done -- a real windowed autopilot run grabbed with
  `xdotool`/`import` shows "YOU DIED", the score line, best line and dismiss hint all
  rendering (previously nothing drew). This is the verification the task explicitly asked
  for and that the autopilot alone cannot provide.
- Tests: `cargo fmt --check`, `cargo clippy --all-targets --features debug`,
  `scripts/check-ascii.sh`, and `cargo test --examples --features debug` (22 passed) all
  green. No tests weakened.
- The stale menu comment that claimed game-over shared its lack of a camera was corrected
  in the same diff -- good hygiene.

No BLOCKER/MAJOR/MINOR/NIT findings. Clean diff, short round.
