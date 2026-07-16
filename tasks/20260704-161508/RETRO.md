# Retro: input/pointer unified pointer resource

- TASK: 20260704-161508
- BRANCH: feat/input-pointer (squash-merged to master as 259a9d5)
- REVIEW ROUNDS: 1 (APPROVE, 2 NITs addressed)

Wave A item 2 of the input-and-projection harvest
(`tasks/20260704-161210/SPIKE.md`), following the
`camera/project` cycle. A mostly clean harvest with one genuinely new gotcha
worth banking.

## What went well

- Resolved the spike's open question up front (raw `Touches` in core, no
  `bevy_enhanced_input` dependency, bridge deferred to `helpers/`) instead of
  discovering it mid-implementation. That decision cleanly determined which
  games could be refactored: asteroids/dropzone read raw `Touches`, fruitninja
  routes through enhanced-input.
- Read all three "copies" before committing to targets, which caught that
  08's `TouchControl` is a virtual-stick controller (a `ui/touchpad` concern),
  NOT a unified pointer -- so the honest refactor set was "asteroids fully +
  06's shared helper", not "all three". Naming the real shape of each copy
  avoided a forced, wrong refactor.
- The one behaviour-sensitive change (asteroids' `Single<&Window>` ->
  `Query<&Window, With<PrimaryWindow>>`, `or_else` -> `or`) was verified
  equivalent line-by-line in review rather than assumed.

## What went wrong

- Declared the library "builds clean" off a bare `cargo build`, which compiles
  the lib but NOT the examples. The `Pointer` name collision (below) was
  invisible until `cargo clippy --all-targets` compiled the examples and threw
  eight E0107/E0659 errors. Root cause: `cargo build` alone is a false green
  for any change that touches example call sites. Caught before commit, but a
  cycle later than it should have been.
- The resource was first named `Pointer`, which collides with bevy's prelude
  `Pointer` (the `bevy_picking` pointer event). The old examples never hit this
  because their *local* `struct Pointer` shadowed both globs; the moment the
  type moved into the crate prelude, `use bevy::prelude::*` + `use
  bevy_common_systems::prelude::*` made every reference ambiguous (E0659).
  Renamed to `UnifiedPointer`. Root cause: harvesting a game-local type into
  the shared prelude changes its name-resolution context -- a local name that
  never clashed can clash once it is prelude-exported.

## What to improve next time

- When harvesting a game-local type into the crate prelude, check its name
  against `bevy::prelude` first. A local `struct Foo` silently shadows a
  bevy-prelude `Foo`; a prelude-exported `Foo` collides with it. This will
  recur for the next harvested types.
- Never trust a bare `cargo build` for a change that touches examples. Run
  `cargo clippy --all-targets` (or `cargo build --examples`) as the real
  compile gate -- the examples are the integration tests and a plain build
  skips them entirely.

## Action items

- [x] tatr 20260704-173937 seeded: `helpers/` enhanced-input bridge to
  `UnifiedPointer`, to retire 06's last local `struct Pointer` (review NIT R1.2).
- [x] Proposed AGENTS.md gotcha: prelude-exported types must not collide with
  `bevy::prelude` names (the `Pointer` -> `UnifiedPointer` rename).
- [ ] Wave A continues: `ui/touchpad` reveal-on-first-touch + hit-test
  primitives (tatr 20260704-161513) -- and 08's `TouchControl` is its headline
  evidence, now that this task confirmed it is a stick, not a pointer.
