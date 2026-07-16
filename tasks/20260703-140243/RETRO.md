# Retro: time-window combos

- TASK: 20260703-140243 (merged, 1 round, APPROVE)

## What went well
- Kept the anti-cheese intact while reworking combo lifetime by being precise
  about the two separate concerns: slicing (still gated on swipe speed) vs the
  combo counter's reset (moved to a timer). Separating them meant the rework
  did not reopen the hold-to-farm hole.
- Added timer reset to start_game in the same change (reset-new-state lesson).

## Improve next time
- Nothing notable; the earlier anti-cheese design made this rework clean.
