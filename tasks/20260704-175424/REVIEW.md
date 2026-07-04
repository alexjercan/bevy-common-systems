# Review: ui/menu screen builders + TitlePulse

- TASK: 20260704-175424
- BRANCH: feat/ui-menu

## Round 1

- VERDICT: APPROVE

Reviewed the diff (`git diff master...HEAD`): new `src/ui/menu.rs`, prelude
wiring in `src/ui/mod.rs`, and the 06/07 refactors. Ran the full suite in the
worktree: `cargo fmt --check`, `cargo clippy --all-targets` (clean),
`cargo clippy --all-targets --features debug` (clean), `cargo test` (47 pass),
`cargo test --examples` (10 pass), `check-ascii.sh`. Booted 06 and 07 to the
render loop (swap-chain line, no panic).

Correctness and design are sound. Notes below are informational.

- [x] R1.1 (NIT) src/ui/menu.rs:55 - `screen_text` now always attaches
  `TextLayout { justify: Center }`, which the old 06/07 helpers did not. This is
  the deliberate superset from 10's variant and only affects multi-line strings
  (the single-line titles/prompts are unchanged); it centers the multi-line
  controls line, a visual improvement, not a regression. Called out so the
  behaviour delta is on record.
  - Response: Acknowledged, intentional. The controls hint is the only
    multi-line string and it reads better centered.
- [x] R1.2 (NIT) src/ui/menu.rs:138 - `pulse_titles` runs every frame in all
  states, where the old `pulse_menu_title` was gated `in_state(Menu)`. Safe: the
  title carries `DespawnOnExit(Menu)`, so outside the menu the query is empty and
  the cost is negligible; driving the pulse off component presence rather than a
  state gate is actually cleaner and lets any screen reuse it.
  - Response: Intentional design - the component owns the behaviour, no state
    coupling.
- [x] R1.3 (NIT) TASK.md names `menu_screen`/`game_over_screen` builders and
  five games; this branch ships `centered_screen`/`screen_text`/`TitlePulse`
  only and refactors two games (06 baseline, 07 second proof). The
  opinionated whole-screen builders are a documented negative result (their
  content - title, controls string, colours, best-score format - varies per game,
  so a builder would be all-parameters; the `status_bar_item` analogy the task
  itself cites is pieces-not-framework). Refactoring 06+07 and folding 09/10/11
  into a migration follow-up matches this wave's established two-example pattern.
  - Response: Scope decided as above; migration of the remaining games tracked as
    a follow-up. Documented in the retro.
