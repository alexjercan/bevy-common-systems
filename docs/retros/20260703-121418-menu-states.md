# Retro: Fruit ninja main menu and game states

- TASK: 20260703-121418
- BRANCH: feature/fruitninja-menu (merged to master, deleted)
- REVIEW ROUNDS: 1 (APPROVE, 2 NITs left intentionally)

See TASK.md close-out for what shipped; this is process only.

## What went well

- Verifying the Bevy 0.18 state-scoped API against the installed source
  (`bevy_state-0.18.1/src/state_scoped.rs` -> `DespawnOnExit(state)`, and the
  bundled `bevy-0.18.1/examples/ecs/state_scoped.rs` for usage) before writing
  meant I used the right component name and learned it is auto-registered by
  `init_state` (no `enable_state_scoped_entities` call). No guessing, compiled
  first try. Same "repo/deps are the API reference" habit that paid off in the
  score-HUD task.
- The throwaway-boot verification pattern extended cleanly from "does it
  render" to "does the state machine cycle": a temporary auto-driver that
  advances Menu -> Playing -> GameOver -> Menu and logs `fruit_count` per state
  proved both the transitions AND the `DespawnOnExit(Playing)` cleanup AND the
  score reset in one 16s run. Logging a cross-cutting invariant (entity count
  per state) turned an un-clickable headless session into a real behavioral
  test.
- Splitting the goal so this task owns only states/menu kept the diff
  reviewable; the temporary `Escape` give-up made the state machine fully
  exercisable before bombs (the real lose trigger) existed, so the task did
  not depend on an unbuilt feature to be testable.

## What went wrong

- Nothing blocking. One honesty wrinkle: the menu text says "avoid the bombs"
  while bombs do not exist until the next task. It is correct-by-the-time-the-
  flow-finishes but wrong at this task's boundary. Root cause: I wrote the
  player-facing copy for the finished game, not the current increment. Called
  it out as a NIT rather than silently shipping copy that describes absent
  behavior.

## What to improve next time

- When one flow builds a feature across several tasks, keep player-facing copy
  honest to the current increment, or explicitly note in the review that it is
  forward-referencing the next task (which is what I did). A reviewer seeing
  "avoid the bombs" with no bombs in the code should be told it is deliberate,
  not a bug.

## Action items

- [x] States + menu shipped, reviewed to APPROVE, merged.
- [ ] Carried into next task (20260703-121347): bombs make the menu copy true
  and replace the `Escape` stand-in with the real health-driven lose trigger.
