# Retro: breach -- game-over screen has no camera (UI invisible), add one

- TASK: 20260705-154058
- BRANCH: fix/breach-gameover-camera
- REVIEW ROUNDS: 1 (APPROVE, no findings)

See `tasks/20260705-154058/TASK.md` for the fix and `REVIEW.md` for the review.

## What went well

- The fix was a mechanical mirror of the menu-state camera the parent task
  (`tasks/20260705-151821`) had already added. Grepping for the menu's
  `Camera2d` + `DespawnOnExit` idiom gave the exact pattern to copy, so there
  was nothing to design -- just apply the same shape to `spawn_game_over`.
- The task spec named the verification method it wanted (real windowed grab, not
  a headless framebuffer capture) and named why (headless comes back black).
  Following that literally caught the one hazard: the first `xdotool search
  --name "Breach"` matched a *browser tab* titled "Breach", not the game. The
  game window's title is `14_breach`; searching `^14_breach$` fixed it. Reading
  the example for the actual `Window { title }` before searching would have
  skipped the wasted first grab.
- Temporarily bumping the autopilot's `.hold(GameState::GameOver, 0.8)` to 8.0s
  gave a comfortable window to grab the frame, then reverting it kept the change
  to the actual one-line fix. Clean way to make a timer-driven harness hold a
  transient state long enough to observe.

## What went wrong

- Nothing blocking. One wasted screenshot from matching the wrong X window by a
  too-loose title substring (a browser tab), corrected on the next grab.

## What to improve next time

- Before `xdotool search --name`, read the example's `Window { title: ... }` and
  match it anchored (`^title$`). A bare substring silently matches unrelated
  windows (browser tabs, editors) that happen to contain the word.

## Action items

- [x] No AGENTS.md change: the "every UI-rendering state needs its own camera"
  lesson is already recorded in the `14_breach` description (the menu-camera gap
  note). This GameOver fix is the companion case, now closed; a second gotcha
  entry would be redundant.
