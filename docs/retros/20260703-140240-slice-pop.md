# Retro: slice pop flash

- TASK: 20260703-140240 (merged, 1 round, APPROVE)

## What went well
- Chose scale-only pop over a material flash after noticing the material swap
  would leak into fragment colors (on_fragments_spawned reuses the shell
  material). Picking the design that avoids the bug beats fixing it later.
- Kept bombs on the instant path so the pop did not tangle with the death beat
  from the shake task - respecting the earlier task's timing.

## Improve next time
- When a new visual reuses an existing shared value (the shell material), trace
  who else reads it before mutating it.
