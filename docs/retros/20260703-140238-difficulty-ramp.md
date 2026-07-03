# Retro: difficulty ramp over time

- TASK: 20260703-140238 (merged, 1 round, APPROVE)

## What went well
- Extracted the ramp into pure helpers first, so the curve is CI-tested
  (endpoints + midway) without needing to run the game - the recurring
  "test the pure core" lesson, applied by default now.
- Named start/floor/cap/ramp-duration consts keep the curve a one-line tune.

## Improve next time
- Nothing notable; small, well-scoped, self-tested change.
