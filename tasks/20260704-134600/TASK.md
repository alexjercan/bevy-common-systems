# feedback: hit-flash material flash module (Wave 1)

- STATUS: OPEN
- PRIORITY: 36
- TAGS: spike,feature,feedback

> Spike: docs/spikes/20260704-134035-game-juice-and-scaffolding-kit.md (read
> first). Wave 1 -- promote recurring example juice into the library.

## Goal

Add a `feedback` module (new top-level concern) for the hit/damage material
flash that three games (06, 07, 10) hand-roll: a `Flash { color, duration }`
component that briefly overrides an entity's material emissive / base color and
eases it back to the original.

The design problem to solve cleanly (spike open question) is restoring the
material without leaking handles when the base material is shared -- e.g. snap
the original values on `On<Add, Flash>` into a private `*State` and restore on
completion, or spawn a per-entity material clone. Follow the Config / State
convention and observers for setup. Prove it by refactoring one example
(10_asteroids) onto it. Pairs with the `tween` module for the decay curve.
