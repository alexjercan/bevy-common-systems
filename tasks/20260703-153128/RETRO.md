# Retro: enlarge game embed + center game page title

- TASK: 20260703-153128 (committed on web-showcase, 1 round, APPROVE)

## What went well
- Recognized this was pure gallery CSS - the game fits its canvas to the parent,
  so a bigger frame just renders bigger with no (multi-minute) wasm rebuild.
- Re-checked the FOV math before widening: aspect 0.8 still covers the fruit
  spawn range, so a bigger frame did not reintroduce edge clipping.

## Improve next time
- Trivial CSS tweak; nothing notable.
