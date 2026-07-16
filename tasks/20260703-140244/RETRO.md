# Retro: golden bonus fruit

- TASK: 20260703-140244 (merged, 1 round, APPROVE)

## What went well
- Used timer.max for the window extension so golden never shortens an active
  combo, only lengthens it - the correct monotonic behavior without a branch.
- Forced GOLDEN_CHANCE high in the throwaway probe to make a rare event
  reliably testable - representative-regime verification (popup retro lesson).

## Improve next time
- Nothing notable; built cleanly on the time-window combo groundwork.
