# radial gravity: RadialGravity component or documented recipe (Wave B)

- STATUS: CLOSED
- PRIORITY: 20
- TAGS: spike,feature,physics

> Spike: tasks/20260704-161210/SPIKE.md (read first). Wave B -- only 2 games; a documented recipe is an
> acceptable outcome.

## Goal

Promote the radial ("point") gravity idiom two games set up around
`Gravity(Vec3::ZERO)`: `gravity.0 = -position.normalize_or(Vec3::Y) * strength`,
in `examples/08_dropzone.rs:1636` (and recurring at `:1668,1763,2184`) and the
zero-g setup in `examples/10_asteroids.rs:170`. Options: a `RadialGravity {
strength }` component + system that writes each body's per-frame gravity toward
a center, OR -- since it is only 2 games and avian-coupled -- a documented recipe
next to `physics/pd_controller`.

DECIDE AT SKETCH TIME which of the two it should be; do not force a module if the
recipe is clearer. If a module, unit-test the direction math and prove by
refactoring 08 onto it.
