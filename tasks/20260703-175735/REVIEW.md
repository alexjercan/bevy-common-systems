# Review: CI: run example unit tests (cargo test --examples)

- TASK: 20260703-175735
- BRANCH: ci/example-tests

## Round 1

- VERDICT: APPROVE

Diff is small and on-spec: a new "Example tests" CI step runs
`cargo test --examples`, AGENTS.md documents the command and the CI-suite
sentence, and the flaky `10_asteroids` assertion that the new coverage exposed
is corrected. Verified independently:

- All 58 example tests run and pass (`06`=19, `07`=14, `08`=7, `10`=9, `11`=9);
  `06_fruitninja` = 19 as the Goal expects.
- The "no `--features debug` variant" claim is correct: no example test is
  `#[cfg(feature = "debug")]` (the debug-gated lines in `examples/*` are all
  plugin wiring in setup, not `#[test]` fns), so a debug run would add zero
  example coverage. Skipping it is right, not a gap.
- The `10_asteroids` assertion rewrite is a genuine correction, not a
  weakening-to-green. The old `speed >= parent drift` is false by design
  (`spawn_asteroid` zeroes the burst's z on a planar arena, so an out-of-plane
  burst leaves a shard slower than the parent); the new
  `|vel - drift| <= SPLIT_SPEED` is the invariant that actually holds and still
  catches a dropped/oversized/doubled drift term. Confirmed stable over repeated
  runs; the RNG-driven slice geometry no longer flips the result.
- fmt / clippy `--all-targets` / ascii all clean in the worktree.

- [x] R1.1 (MINOR) examples/10_asteroids.rs:1690 - the new velocity assertion
  (`|vel - drift| <= SPLIT_SPEED`) is satisfied even when `velocity == drift`,
  i.e. it does not fail if the outward-burst term were dropped
  (`velocity = parent_vel` alone). The comment above it claims shards get "an
  outward burst", but nothing asserts the burst is actually applied -- so a
  regression that stops applying `world_dir * SPLIT_SPEED` would slip through.
  Per the crate's own test-honesty rule (a comment claiming behavior X must be
  backed by an assertion of X), strengthen the test to also prove the burst is
  live, e.g. assert that at least one shard's velocity differs from the parent
  drift by a nonzero margin (`shards.iter().any(|v| (v - drift).length() > eps)`).
  Not blocking: the headline behavior (shards respawn as gen-1 dynamic bodies
  with colliders) is asserted by the surrounding checks.
  - Response: Addressed. Iterate the shards by reference and, after the
    per-shard loop, assert `shards.iter().any(|v| (v - drift).length() > 1e-2)`
    so a dead burst term (velocity == parent drift for every shard) now fails
    the test. Only one shard is required to differ, since a single shard's burst
    can point nearly straight out of plane and coincidentally land near the
    parent drift. Stable over 8 back-to-back runs; fmt/clippy clean.

## Round 2

- VERDICT: APPROVE

R1.1 resolved and verified against the new diff: the burst-liveness assertion
is present at examples/10_asteroids.rs and would catch a dropped
`world_dir * SPLIT_SPEED` term. No new findings; the diff remains small and
on-spec. All 58 example tests pass, fmt / clippy `--all-targets` / ascii clean.
