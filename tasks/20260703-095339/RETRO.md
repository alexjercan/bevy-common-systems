# Retro: Fix doctests and clippy warning so cargo test passes clean

- TASK: 20260703-095339
- BRANCH: fix/doctests-clippy (merged to master, deleted)
- REVIEW ROUNDS: 1 (APPROVE, no findings)

See TASK.md close-out for what changed; this is process only.

## What went well

- Running the failing command under both feature combinations
  (`cargo test --doc` and `--doc --features debug`) before editing
  surfaced the true scope up front: 12 blocks, not the 10 the task
  predicted. No "thought I was done" surprise at the end.
- Choosing compiling hidden-setup doctests over `ignore` turned dead
  documentation into live regression guards, and in doing so exposed two
  real doc bugs (`add_plugin` vs `add_plugins`, `HealthApplyDamage.target`
  vs `.entity`) that an `ignore` would have buried forever.
- Deferring the pre-existing non-ASCII chars in chase.rs to a separate
  task kept this diff single-purpose, which made review a short round.

## What went wrong

- One rework cycle inside the work phase: the mesh/builder doctest failed
  a second time with `build` not in scope, because `TriangleMeshBuilder`'s
  `build` comes from Bevy's `MeshBuilder` trait and the crate prelude does
  not re-export it. Root cause: assumed the crate `mesh::prelude` was
  sufficient without checking where `build` is actually defined. Cheap to
  fix (add `use bevy::prelude::*`), but it was a second test run that a
  30-second check of the method's home trait would have avoided.

## What to improve next time

- When writing a doctest that calls a method, confirm which trait/module
  provides it before choosing the imports - Bevy leans heavily on
  extension traits (`MeshBuilder`, etc.) that the crate prelude may not
  re-export.

## Action items

- [x] Follow-up filed: tasks/20260703-101712 (non-ASCII chars in
  chase.rs docs).
- [ ] Candidate AGENTS.md note if it recurs: "doctests calling Bevy
  extension-trait methods (e.g. MeshBuilder::build) need
  `use bevy::prelude::*`, not just the crate prelude." Holding off until a
  second occurrence to avoid over-documenting a one-off.
