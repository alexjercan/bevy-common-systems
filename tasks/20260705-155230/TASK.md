# fix tween despawn-race crash: advance_tween inserts on despawned entity

- STATUS: CLOSED
- PRIORITY: 100
- TAGS: bug,crash,tween,historical

## Goal

Fix the P100 crash (`tasks/20260705-151725`): breach crashes mid-combat with
`Entity despawned: ... interacting with a despawned entity`. The backtrace points at
`bevy_common_systems::tween::advance_tween::<f32>` queuing an
`insert::<TweenFinished>` command that applies to an already-despawned entity.

Root cause: on completion, `advance_tween` (src/tween/mod.rs) runs
`commands.entity(entity).insert(TweenFinished)` (Keep), `.remove().insert()` (Remove),
or `.despawn()` (Despawn). If another system despawns that entity between the query
iteration and the command-buffer flush -- e.g. a `feedback/flash` tween on an enemy the
kill chain despawns, or a `ui/popup` fade whose entity is despawned by `DespawnOnExit` /
another path -- `commands.entity(entity)` panics. This is a crate bug affecting every
example that tweens short-lived entities.

Done = `advance_tween` no longer panics when a tween entity is gone at apply time; a
headless regression test reproduces the race and asserts no panic; a sustained-combat
breach headless run shows no `Entity despawned` error.

## Notes

- Fix in the crate (`src/tween/mod.rs`), not at the call sites. Bevy 0.19 has
  `try_insert` / `try_remove` / `try_despawn` (no-ops if the entity is gone); the crate
  already uses `try_remove`/`try_insert` for this reason (`camera/chase`,
  `feedback/flash`). Use them in all three completion branches.
- `advance_tween` is generic over `TweenValue` (f32/Vec2/Vec3/Vec4), so one edit covers
  all four registered systems.
- Existing tween tests (Keep/Remove/Despawn) must still pass; add the race test beside
  them.
- The call-site orderings that worked around this (breach flash-before-damage, glide
  tween ordering) become belt-and-suspenders -- leave them, do not churn.

## Steps

- [x] **Make completion despawn-safe.** In `advance_tween`, replace `insert` ->
  `try_insert`, `remove::<Tween<T>>().insert(..)` -> `try_remove::<Tween<T>>().try_insert(..)`,
  and `despawn()` -> `try_despawn()`. Keep the `completed` guard.
- [x] **Regression test.** Headless `App` (MinimalPlugins + TweenPlugin, or just the
  system) with a zero/short-duration `Tween<f32>` (Keep) on an entity, plus a despawner
  system `.chain()`ed BEFORE `advance_tween::<f32>` so its despawn command applies first;
  `app.update()` must NOT panic and the entity is gone. Do the same shape for a Remove
  tween. (Pre-fix this panics; post-fix it is a clean no-op.)
- [x] **Verify.** `cargo fmt`, `cargo clippy --all-targets`, `cargo test` (crate unit +
  doctests) and `cargo test --example 14_breach`, `scripts/check-ascii.sh`. Then a breach
  headless run with an extended Playing hold (more kills) and grep for `Entity despawned`
  / `panic` -- expect none. Note the fix in the tween module doc if warranted.
