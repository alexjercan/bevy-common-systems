# Review: leaf input/status/material helpers

- TASK: 20260704-175425
- BRANCH: feat/input-leaf-helpers

## Round 1

- VERDICT: APPROVE

Reviewed `git diff master...HEAD`: four new helpers (`AnyStartPress` +
`any_start_pressed` in input/pointer, `set_state_on_key` in a new input/state,
`status_bar_with_fps` in ui/status, `glowing_material` in a new material module)
and the 07/10 refactors. Ran the full suite in the worktree: `cargo fmt --check`,
`cargo clippy --all-targets` (clean), `--features debug` (clean), `cargo test`
(93 unit + 51 doctests), `cargo test --examples`, `check-ascii.sh`. Booted 07
and 10 to the render loop (swap-chain line, no panic).

The helpers match the spike's evidence and the crate conventions (prelude
exports, `//!` docs with runnable snippets, `debug!`/`trace!` untouched). Notes:

- [x] R1.1 (NIT) src/input/pointer.rs:135 - `AnyStartPress::just_pressed`
  includes `Enter`, which 10's old check (`pointer.just_pressed || Space`) did
  not. This is a deliberate superset lifted from 07's `advance_pressed`; it can
  only make a screen easier to dismiss, never harder, so it is a safe unification.
  - Response: Intentional - one "advance" definition for all games, the union of
    what they individually accepted.
- [x] R1.2 (NIT) src/material.rs - `glowing_material` is a pure function whose
  entire value is the "never unlit" guarantee, so per the repo's "back a doc
  claim with an assertion" rule it earns a unit test. Added
  `sets_base_and_emissive_and_stays_lit` asserting `!mat.unlit`.
  - Response: Added during self-review before finalizing.
- [x] R1.3 (NIT) TASK.md asks to migrate all six games onto `UnifiedPointer`
  and the new helpers; this branch proves them on 07 (drops local
  `advance_pressed`/`giveup_on_escape`, adopts all four) and 10 (drops its
  pointer+keys advance checks and `giveup_on_escape`), matching the wave's
  two-example proof pattern. Remaining games (06/08/09/11/12) fold into the
  migration follow-up.
  - Response: Scope as above; documented in the retro and the follow-up task.
