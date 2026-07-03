# Fix bevy 0.19 build: ResMut mutability + chain/deref/or_else in modding/events.rs

- STATUS: CLOSED
- PRIORITY: 90
- TAGS: bug,bevy-migration

## Goal

After the bevy 0.18 -> 0.19 bump, `src/modding/events.rs` no longer compiles.
Bevy 0.19 makes `ResMut<T>` require `T: Resource<Mutability = Mutable>`, so the
generic `EventWorld` resource used through `ResMut<W>` is rejected; that failure
cascades into `.chain()` resolving against the wrong trait (`Curve`) and into
`filter`/`action` being handed a `ResMut` instead of `&W`. Also a deprecation:
`SystemCondition::or` is deprecated in favor of `or_else`. Fix all four so the
event bus compiles and behaves identically.

## Steps

- [x] Constrain the world type to a mutable resource: change the trait bound
      `pub trait EventWorld: Resource + Send + Sync` (line ~21) to
      `pub trait EventWorld: Resource<Mutability = Mutable> + Send + Sync`
      with `use bevy::ecs::component::Mutable;`. This satisfies the `ResMut<W>`
      bound in `queue_system` (fixes E0271 at line ~230) and lets `.chain()`
      resolve against the scheduling trait again (fixes both E0599 at line ~198).
- [x] In `queue_system` (lines ~241, ~246), deref the `ResMut<W>` when calling
      the handler API: `handler.filter(&world, ..)` -> `handler.filter(&*world,
      ..)` and `action.action(&mut world, ..)` -> `action.action(&mut *world,
      ..)`, since `filter` takes `&W` and `action` takes `&mut W` (fixes the two
      E0308 at those lines).
- [x] Replace the deprecated run-condition combinator (line ~199):
      `not(is_queue_empty::<W>).or(resource_changed::<W>)` ->
      `not(is_queue_empty::<W>).or_else(resource_changed::<W>)` to clear the
      `deprecated SystemCondition::or` warning while preserving semantics.
- [x] Verify the module compiles clean: `cargo build --all-targets` (all
      E0271/E0599/E0308 in this file gone, no deprecation warning).
- [x] Full check suite: `cargo fmt --check`, `cargo clippy --all-targets`
      (+ `--features debug`), `cargo test`, `./scripts/check-ascii.sh`.
- [x] Boot the modding example end to end to confirm no behavior change:
      `cargo run --example 03_modding` (it prints handler output to the
      console); confirm events still fire and handlers still run.

## Notes

- Errors, all in `src/modding/events.rs`:
  - `E0271 <W as Component>::Mutability == Mutable` at line ~230 (`ResMut<W>`).
  - `E0599 the method chain ... trait bounds not satisfied` at line ~198 (x2)
    -- a knock-on of the ResMut failure; fixing the bound should resolve it.
  - `E0308 expected &W, found &ResMut<W>` at line ~241 (filter).
  - `E0308 expected &mut W, found &mut ResMut<W>` at line ~246 (action).
  - `warning: deprecated SystemCondition::or` at line ~199.
- Root cause is the bevy 0.19 split of `Component`/`Resource` mutability into an
  associated `Mutability` type; `ResMut` is only valid for `Mutable` resources.
  Confirm the canonical import path for `Mutable` (likely
  `bevy::ecs::component::Mutable`, re-exported somewhere) and let `cargo fmt`
  manage the import block.
- Do the trait-bound fix first; re-run `cargo build` before touching the deref
  sites, since the E0599 `.chain()` errors may already vanish.
- No new dependencies.

## Close-out

Added `Resource<Mutability = Mutable>` to the `EventWorld` bound (with
`use bevy::ecs::component::Mutable;`); confirmed the `.chain()` E0599 errors were
purely a knock-on and vanished once the bound compiled. Deref'd `world` at the
`filter`/`action` call sites (`&*world` / `&mut *world`). Swapped the deprecated
`.or(..)` run-condition for `.or_else(..)` (bevy 0.19 lazy-evaluates; semantics
unchanged for these two side-effect-free conditions). Full suite green;
`cargo run --example 03_modding` boots and the handler output prints as before.
