# Review: Fruit ninja main menu and game states

- TASK: 20260703-121418
- BRANCH: feature/fruitninja-menu

## Round 1

- VERDICT: APPROVE

The state machine is correct and idiomatic for Bevy 0.18: `init_state`,
`OnEnter` spawners, `run_if(in_state(...))` gating, `NextState` transitions,
and `DespawnOnExit(state)` for cleanup. Verified on real GPU via a throwaway
state-cycling driver: Menu (0 fruit) -> Playing (fruit spawning) -> GameOver
(fruit despawned by `DespawnOnExit(Playing)`) -> Menu -> Playing (score reset
by `start_game`), no panic. Checks clean (`fmt`, `clippy --all-targets` both
feature configs, `check-ascii`). Persistent camera/light/FPS bar correctly
stay out of state scoping; HUD/menu/gameplay are scoped. `centered_screen` /
`screen_text` factor the shared UI cleanly. Only two NITs.

- [ ] R1.1 (NIT) examples/06_fruitninja.rs:286 - the menu text advertises
  "avoid the bombs", but bombs do not exist until task 20260703-121347. At the
  end of this task the instruction describes behavior that is not present yet.
  It becomes true the moment the bombs task lands (next in the queue), so this
  is acceptable to leave; noting it for honesty. If the bombs task slips,
  drop the "avoid the bombs" clause.
  - Response: Left intentionally. Bombs (20260703-121347) are the very next
    task in the flow queue and make this true; if that task were dropped, the
    clause should go.

- [ ] R1.2 (NIT) examples/06_fruitninja.rs:295-298,273 - `menu_click` sets
  `Playing` on `just_pressed`, but `slice_fruit` begins a swipe on `pressed`
  (held), not `just_pressed`. If the player holds the mouse button through the
  menu-start click into the first `Playing` frame, a fruit under the cursor
  could be sliced immediately. Very low impact (fruit spawn from the bottom,
  so nothing is usually under the cursor at start), but if it ever feels
  wrong, gate the start of a swipe on `just_pressed` or skip slicing on the
  first `Playing` frame.
  - Response: Left to discretion - impact is negligible (fruit spawn from the
    bottom edge, not under the cursor at start). Revisit only if it feels wrong
    in play.
