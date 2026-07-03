# Fix doctests and clippy warning so cargo test passes clean

- STATUS: CLOSED
- PRIORITY: 50
- TAGS: bug

## Goal

Make the full `cargo test` (including doctests) and
`cargo clippy --all-targets --features debug` pass with no failures or
warnings, so the check suite documented in AGENTS.md can be tightened from
`cargo test --all-targets` to plain `cargo test`.

## Steps

- [x] Fix or mark the failing doctests. These are illustrative snippets
      in module docs that reference out-of-scope variables (`commands`,
      `input`, `player_position`, ...). Either wrap them into compiling
      examples (preferred where cheap: full `fn main` or hidden setup lines
      with `#`) or mark the fence ```ignore where compiling them adds no
      value. CORRECTED during work: 12 failing blocks, not 10 - running
      doctests with --features debug surfaces 2 more. Full list:
      src/camera/chase.rs (line 26), src/camera/post.rs (13),
      src/camera/skybox.rs (9 and 30 - line 9 is ASCII art wrongly parsed
      as a doctest; make it a ```text fence), src/camera/wasd.rs (17),
      src/health/mod.rs (11), src/helpers/despawn.rs (14),
      src/helpers/mod.rs (11), src/helpers/temp.rs (15),
      src/mesh/builder.rs (14), src/debug/mod.rs (10),
      src/debug/wireframe.rs (7). Passing and untouched: src/camera/mod.rs
      (11 and 22), src/debug/mod.rs (29).
- [x] Fix the typo `heleprs` -> `helpers` in the src/helpers/mod.rs module
      doc while touching that block.
- [x] Fix clippy::default_constructed_unit_structs in
      src/debug/inspector.rs:58 (`PhysicsDebugPlugin::default()` ->
      `PhysicsDebugPlugin`); only visible with `--features debug`.
- [x] Run the full suite: `cargo fmt --check`, `cargo clippy --all-targets`,
      `cargo clippy --all-targets --features debug`, `cargo test`,
      `cargo test --features debug`.
- [x] Update AGENTS.md: change the documented test command from
      `cargo test --all-targets` to `cargo test` and drop the known-issues
      note about failing doctests (keep the note style consistent).

## Notes

- Discovered during 20260703-094842 while verifying commands for AGENTS.md.
- `cargo test` currently: 2 doctests pass, 10 fail. `cargo test
  --all-targets` passes (13 unit tests in meth/sphere, mesh/builder,
  physics/pd_controller, transform/point_rotation; doctests skipped).
- Do not weaken existing passing doctests; module docs in meth/, debug/,
  camera/mod.rs currently pass or are not doctests.
- Depends on: 20260703-094842 (AGENTS.md must exist before updating it).

## Close-out

What changed and why:
- Turned the 12 illustrative module-doc snippets into real, compiling
  doctests by wrapping them in hidden setup (`# use ...`, `# fn demo(...)
  { ... # }`) instead of marking them `ignore`. Compiling doctests are
  worth more than ignored ones: they now fail CI if the documented API
  drifts, which is exactly what a copy-pastable-crate's docs need.
- src/camera/skybox.rs: the 6-face ASCII layout was being parsed as Rust;
  fenced it as ```text.
- src/debug/mod.rs: the plugin example needs a real window to `run()`, so
  it is `no_run` (compiled, not executed) with `use bevy::prelude::*`.
- src/mesh/builder.rs: `TriangleMeshBuilder::build` comes from Bevy's
  `MeshBuilder` trait, so the doctest needs `use bevy::prelude::*` in
  scope (not just the crate mesh prelude).
- Fixed two real doc bugs found in passing: the `heleprs` typo in
  helpers/mod.rs and `HealthApplyDamage { target: ... }` which used a
  field name that does not exist (the field is `entity`).
- Fixed clippy::default_constructed_unit_structs in debug/inspector.rs.
- Updated AGENTS.md: documented suite is now plain `cargo test` /
  `cargo test --features debug`, and the known-issue block is gone,
  replaced by a note that the only remaining clippy warning is a
  transitive-dependency future-incompat note.

Difficulties:
- The task estimated 10 failing doctests; the real count was 12, because
  two live in the `debug` module which only compiles under
  `--features debug`. Caught immediately by running
  `cargo test --doc --features debug` at the start rather than trusting
  the estimate.
- The mesh/builder doctest failed on a second pass with a trait-not-in-
  scope error (`build` is a `MeshBuilder` method). The crate prelude does
  not re-export that Bevy trait; adding `use bevy::prelude::*` fixed it.
  Lesson: doctests that call methods from Bevy traits need the bevy
  prelude, not only the crate prelude.

Self-reflection:
- Running the failing command under every feature combination first
  (`--doc` with and without `--features debug`) surfaced the true scope
  before any edits, which avoided a "thought I was done" surprise.
- Preferring compiling hidden-setup doctests over `ignore` cost a little
  more effort per block but turned dead documentation into a live
  regression guard, consistent with the crate's copy-pastable goal.
