# scoring: Score + Combo/Streak decay resource (Wave B)

- STATUS: CLOSED
- PRIORITY: 22
- TAGS: spike,feature,scoring

> Spike: tasks/20260704-161210/SPIKE.md (read first). Wave B -- sketch-then-commit; a thin-wrapper
> negative result is acceptable.

## Goal

Add a `scoring` module with a `Score` resource and a `Combo`/`Streak` primitive
that tracks a decaying hit-window and scales a multiplier. Strong signal: the
mechanic was literally copied between games -- `examples/07_orbit.rs:285`
`struct Streak { count, timer }` carries the comment "Modelled on
`06_fruitninja`'s `Combo`", and plain `Score` resources recur in 5 games
(`06`, `07:283`, `09`, `10:334`, `11`).

The module owns the decay/multiplier bookkeeping only, NOT the scoring rule
("what a point is worth" stays game-specific). SKETCH FIRST and confirm it beats
a raw `Timer + usize` before committing; if it does not, document the pattern
and drop the module (a negative result is fine, per the prior spike's `spawn`
caveat). Unit-test the decay-window / multiplier logic and prove by refactoring
06 and 07 onto it.
