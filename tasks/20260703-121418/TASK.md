# Fruit ninja: main menu and game states

- STATUS: IN_PROGRESS
- PRIORITY: 90
- TAGS: feature,example

## Goal

Add a game-state machine to `examples/06_fruitninja.rs`: the app boots into a
main menu, starts the game on click, and shows a game-over screen when the run
ends, from which the player returns to the menu. Gameplay only runs while
playing.

## Steps

- [x] Define a `GameState` enum deriving `States` with `Menu` (default),
      `Playing`, `GameOver`; register with `app.init_state::<GameState>()` and
      enable state-scoped entity cleanup for it.
- [x] Gate all gameplay systems (`spawn_fruit`, `move_fruit`, `slice_fruit`,
      `move_fragments`, score-text update) with
      `.run_if(in_state(GameState::Playing))`.
- [x] Main menu: `OnEnter(Menu)` spawns a state-scoped UI (title
      "FRUIT NINJA", subtitle "Click to play"). A system in `Menu` starts the
      game (`GameState::Playing`) on left click.
- [x] `OnEnter(Playing)`: reset `Score` to 0 and reset the spawn timer so each
      run starts fresh. Make the in-game HUD (score text) state-scoped to
      `Playing` (spawn it in `OnEnter(Playing)` rather than `setup`, or mark it
      state-scoped) so it does not show in the menu.
- [x] Ensure fruit and fragments are cleaned up when leaving `Playing` (mark
      them state-scoped to `Playing`, or despawn in `OnExit(Playing)`), so a
      new run starts with an empty field.
- [x] Game over: add a temporary trigger for now -- pressing `Escape` while
      `Playing` transitions to `GameOver` (the bombs task will add the real
      trigger). `OnEnter(GameOver)` spawns a state-scoped UI showing the final
      score and "Click to return to menu"; a click returns to `GameState::Menu`.
- [x] Keep the persistent camera/light/status-bar (FPS) out of state scoping so
      they survive across states; only gameplay entities and menu/HUD UI are
      state-scoped.
- [x] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, and a real boot walking menu ->
      play -> Escape -> game over -> menu.

## Notes

- Depends on: 20260703-121414 (score UI) -- this task makes that HUD
  state-scoped.
- Bevy 0.18 states: `#[derive(States, Default, ...)]`, `init_state`,
  `OnEnter`/`OnExit(state)`, `in_state(state)`. State-scoped entities in 0.18
  are done with `enable_state_scoped_entities::<S>()` + a `StateScoped(state)`
  component (verify the exact type name/`DespawnOnExit` naming against the
  installed Bevy during work).
- Menu/gameover click uses `ButtonInput<MouseButton>::just_pressed`; reuse the
  existing cursor plumbing only if needed (menu just needs a click, not world
  position).
- Assumption (confirmed with user): losing is driven by health hitting zero
  (bombs task). Here `Escape` is a stand-in trigger so the state machine is
  fully exercisable before bombs exist; keep it as a "give up" shortcut after.
- No new dependencies.
