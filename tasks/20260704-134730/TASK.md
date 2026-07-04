# spawn + time/cooldown: timed spawner and cooldown/i-frames (Wave 2)

- STATUS: OPEN
- PRIORITY: 24
- TAGS: spike,feature

> Spike: docs/spikes/20260704-134035-game-juice-and-scaffolding-kit.md (read
> first). Wave 2 -- only if each earns its keep over a raw Timer.

## Goal

Two small timing helpers harvested from the games:

1. `spawn` -- a `Spawner { interval, jitter }` component that emits a spawn
   signal on a cadence (06 and 08 hand-roll interval spawning; wave spawners in
   07/10 are adjacent). The module can only own the *timing*, not the
   game-specific entity, so first sketch it and confirm it is more than a thin
   `Timer` wrapper; if it is not, document the `Timer` pattern instead and drop
   the module (a negative result is fine).
2. `time/cooldown` -- a `Cooldown` component for weapon fire and invulnerability
   windows (10_asteroids hand-rolls i-frames). Small; could live beside
   `helpers/temp`.

Same gate applies: only add code that beats just using `Timer`. Prove whatever
lands by refactoring one example onto it.
