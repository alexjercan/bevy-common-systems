# Retro: combo end summary

- TASK: 20260703-140246 (merged, 1 round, APPROVE)

## What went well
- Traced the reset ownership before adding the points accumulator and caught a
  leak: golden fruit sliced outside a combo would add to points that tick_combo
  never clears (it only resets on a counted-combo expiry). Guarded it. This is
  the "know who resets shared state" habit paying off during implementation.
- Unit-tested the tally accounting on the real advance_combo.

## Improve next time
- When adding an accumulator, immediately ask "every path that adds must have a
  path that resets" - the golden path added without a matching reset.
