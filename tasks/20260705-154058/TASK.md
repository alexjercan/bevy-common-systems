# breach -- game-over screen has no camera (UI invisible), add one

- STATUS: OPEN
- PRIORITY: 55
- TAGS: bug,breach,example,ui


## Goal

The game-over screen renders no UI: the only camera is the Playing `Camera3d`, a child
of the player entity, which despawns on exit Playing (`DespawnOnExit(Playing)`). So the
GameOver state has no camera and Bevy UI ("YOU DIED", score, restart hint) never draws.
This is the same latent bug the main-menu task fixed for the Menu state (the menu now
spawns its own `Camera2d`); do the same for GameOver.

Discovered during `tasks/20260705-151821`. Fix: spawn a `Camera2d` in
`OnEnter(GameState::GameOver)` (or in `spawn_game_over`) with
`DespawnOnExit(GameState::GameOver)`, and verify by an actual window grab (headless
framebuffer captures come back black; use a real run + xdotool/import like the menu
task did) that "YOU DIED" and the score render.

## Notes

- Spike/parent: `tasks/20260705-151821` (main menu + options), which found and fixed the
  identical Menu-state gap.
- The autopilot force-transitions to GameOver on a timer, so it will NOT reveal a
  non-rendering GameOver -- verify visually with a real run.
