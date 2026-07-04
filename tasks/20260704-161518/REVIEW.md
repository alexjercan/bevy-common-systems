# Review: scoring/streak decaying combo counter

- TASK: 20260704-161518
- BRANCH: feat/scoring

## Round 1

- VERDICT: APPROVE

A well-judged sketch-then-commit. The task asked to confirm the abstraction
beats a raw `Timer + usize` before committing, and to accept a thin-wrapper
negative result. The implementer split the difference correctly: shipped
`scoring::Streak` (the count-and-decay state machine with the ended-edge that
07 literally copied from 06) and dropped the proposed `Score` resource (a bare
counter with no logic) with that reasoning documented in the module and commit.
That is exactly the discipline the task wanted -- own the bookkeeping, not the
scoring rule.

`Streak` is clean: private `count/timer/window` behind accessors, `hit()`
returns the new count for the multiplier, `tick(dt) -> Option<final_count>`
gives the tally edge, `extend_to`/`reset` and `remaining_frac` (which also
dedups 06's HUD fade). Module docs, prelude, `Reflect`/`register_type`, and the
`scoring/mod.rs` + `scoring/streak.rs` layout all match the crate conventions.
Name checked against `bevy::prelude` (no `Streak` there); clean
`clippy --all-targets` with both preludes confirms no collision.

Behaviour-equivalence verified against the deleted code:
- 07: `advance_streak` -> `hit()`, `tick_streak` -> `tick()` (old reset-on-`<=0`
  == new end-on-not-`>0`), two reset sites -> `reset()`. Identical.
- 06: `Combo` now embeds `Streak` + keeps its own `points`; `advance_combo`
  delegates to `hit()` then tallies; `tick_combo` fires the tally on
  `tick()`'s `Some(final_count)`. The golden-fruit `extend_to(GOLDEN)` guards
  `count > 0`, whereas the old `timer.max(GOLDEN)` was unconditional -- but the
  old write was dead when `count == 0` (tick bails on `count == 0`, the next
  `hit` overwrites the timer, the HUD only reads at `count >= 2`), so the guard
  is observably identical and just drops a dead write. HUD fade
  `timer / COMBO_WINDOW` -> `remaining_frac()`: identical.

Tests: the pure decay behaviour moved into the module (7 unit + 3 doctests, all
meaningful -- window refresh, ended-edge with final count, extend-never-shortens,
zero-window, inactive no-op). The two 06 tests that only exercised that decay
were dropped (relocated, not weakened); 06 keeps its points-tally tests and 07
keeps its value-rule test, both now driving counts through `hit()`.

Verified independently in the worktree: `cargo fmt --check`,
`cargo clippy --all-targets` (default and `--features debug`), `cargo test`
(67 unit + 32 doctests), `cargo test --examples` (06 now 14, 07 13) and
`scripts/check-ascii.sh` all pass; 06 and 07 both boot to the render loop.

No findings.
