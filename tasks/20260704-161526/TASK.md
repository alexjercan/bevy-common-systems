# progress: difficulty-ramp helper, sketch-then-decide (Wave B)

- STATUS: CLOSED
- PRIORITY: 18
- TAGS: spike,feature,meth

> Spike: docs/spikes/20260704-161210-input-and-projection-harvest.md (read first). Wave B, LOWEST priority -- do AFTER the juice-kit
> Wave 2 `tween` task (tasks/20260704-134630); may downgrade to a doc.

## Goal

All six games ramp difficulty over time, but via genuinely different idioms:
time-lerp (`examples/06_fruitninja.rs` `ramp_t`/`spawn_interval_for`), discrete
`Level(n)` (`07_orbit.rs`, `11_overload.rs`), `Wave(n)`
(`10_asteroids.rs`), and log-scaled tiers (`09_reactor.rs` `tier_for_score`).
The only truly shared core is "elapsed -> normalized t -> ease a value from start
to cap".

SKETCH FIRST against the Wave-2 `tween` `EaseFunction` support: if the ramp core
is essentially a one-line ease call, DOCUMENT the two idioms (a time-ramp helper
and a level-interval timer) rather than shipping a heavy `progress` module -- a
doc is an acceptable outcome. Only build a module if a `Level`-timer helper
proves substantial and reused. Each game's level *effects* stay game-specific
regardless.
