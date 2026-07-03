# Retro: high score across runs

- TASK: 20260703-140242 (merged, 1 round, APPROVE)

## What went well
- Used `.chain()` to force record-before-display ordering on GameOver enter,
  a clean fix for the same read-then-write ordering class that bit the swipe
  systems earlier - applied deliberately this time.
- Explicitly noted the reset lesson's inverse: HighScore should NOT be reset in
  start_game (it is session-persistent), so I left it out on purpose.

## Improve next time
- Nothing notable; small, clear feature.
