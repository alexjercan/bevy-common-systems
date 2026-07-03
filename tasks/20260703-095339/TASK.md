# Fix doctests and clippy warning so cargo test passes clean

- STATUS: OPEN
- PRIORITY: 50
- TAGS: bug

## Goal

Make the full `cargo test` (including doctests) and
`cargo clippy --all-targets --features debug` pass with no failures or
warnings, so the check suite documented in AGENTS.md can be tightened from
`cargo test --all-targets` to plain `cargo test`.

## Steps

- [ ] Fix or mark the 10 failing doctests. These are illustrative snippets
      in module docs that reference out-of-scope variables (`commands`,
      `input`, `player_position`, ...). Either wrap them into compiling
      examples (preferred where cheap: full `fn main` or hidden setup lines
      with `#`) or mark the fence ```ignore where compiling them adds no
      value. Failing blocks: src/camera/chase.rs (line 26),
      src/camera/post.rs (13), src/camera/skybox.rs (9 and 30 - line 9 is
      ASCII art wrongly parsed as a doctest; make it a ```text fence),
      src/camera/wasd.rs (17), src/health/mod.rs (11),
      src/helpers/despawn.rs (14), src/helpers/mod.rs (11),
      src/helpers/temp.rs (15), src/mesh/builder.rs (14).
- [ ] Fix the typo `heleprs` -> `helpers` in the src/helpers/mod.rs module
      doc while touching that block.
- [ ] Fix clippy::default_constructed_unit_structs in
      src/debug/inspector.rs:58 (`PhysicsDebugPlugin::default()` ->
      `PhysicsDebugPlugin`); only visible with `--features debug`.
- [ ] Run the full suite: `cargo fmt --check`, `cargo clippy --all-targets`,
      `cargo clippy --all-targets --features debug`, `cargo test`,
      `cargo test --features debug`.
- [ ] Update AGENTS.md: change the documented test command from
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
