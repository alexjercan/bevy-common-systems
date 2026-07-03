# Retro: screen shake and bomb impact feedback

- TASK: 20260703-140237 (merged, 1 review round, APPROVE)

## What went well
- Applied the branch-first and reset-new-state-in-start_game lessons up front:
  trauma and the dying timer were reset in start_game in the same change, no
  review round needed to catch it.
- Made the camera-shake system always-on rather than Playing-only, so the
  camera self-settles across state changes with no extra OnExit reset - one
  fewer moving part.

## What went wrong
- Nothing blocking. The death beat means the player is "dead" but the state is
  still Playing for ~0.35s, so fruit can still be sliced/scored during the
  flash. Judged harmless and left it.

## Improve next time
- For time-delayed transitions (the beat), remember the intermediate window
  where old-state systems still run; gate anything that must not happen after
  "death" explicitly if it ever matters.
