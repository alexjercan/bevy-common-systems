# Review: Fix doctests and clippy warning so cargo test passes clean

- TASK: 20260703-095339
- BRANCH: fix/doctests-clippy

## Round 1

- VERDICT: APPROVE

Verified independently against master:

- Suite: `cargo fmt --check` clean; `cargo test` 13 unit + 11 doctests
  pass; `cargo test --features debug` 13 unit + 14 doctests pass;
  `cargo clippy --all-targets --features debug` has no crate warnings (only
  the pre-existing `proc-macro-error2` transitive future-incompat note).
- Goal delivered: plain `cargo test` now passes, which was the point.
- Doctests are genuinely compiled, not hidden with `ignore`: the snippets
  use `# use ...` / `# fn demo(...)` setup so they are real regression
  guards. `src/debug/mod.rs` correctly uses `rust,no_run` because the
  example calls `.run()` (needs a window); it still compiles. The ASCII
  skybox layout is fenced as `text`. Good calls all round.
- Beyond the plan, two real bugs were fixed and are worth calling out as
  correct: `src/camera/post.rs:20` `add_plugin` -> `add_plugins` (the
  singular does not exist in Bevy 0.18, so the old doc could never have
  compiled), and `src/health/mod.rs:19` `target:` -> `entity` (the
  `HealthApplyDamage` field is named `entity`; the old doc named a
  non-existent field). Both were latent doc bugs the doctest work exposed.
- `src/debug/inspector.rs:58` clippy fix (`PhysicsDebugPlugin::default()`
  -> `PhysicsDebugPlugin`) is correct and behavior-preserving.
- AGENTS.md updated honestly: documented suite is now `cargo test` /
  `cargo test --features debug`, the stale known-issue block is gone, and
  the remaining transitive-dep warning is noted rather than hidden.
- No tests weakened or deleted. Diff introduces no non-ASCII characters.
- Pre-existing non-ASCII in `src/camera/chase.rs` (not in this diff) was
  correctly deferred to a new task (20260703-101712) rather than smuggled
  into this branch.

TASK.md checkboxes and close-out match what the diff actually does. No
findings. Clean diff, short round.
