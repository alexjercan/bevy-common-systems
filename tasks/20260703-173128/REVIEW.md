# Review: Fruit ninja touchscreen support via bevy_enhanced_input

- TASK: 20260703-173128
- BRANCH: feature/fruitninja-touch

## Round 1

- VERDICT: APPROVE

Reviewed `git diff master...feature/fruitninja-touch` against TASK.md, re-ran
the full check suite in the worktree, and verified the enhanced_input event
semantics against the vendored crate source rather than trusting the summary.

### What holds up

- Goal delivered: the whole loop (menu -> slice -> game over -> menu) reads a
  single `Pointer` resource; the press is one `PointerPress` action bound to
  `MouseButton::Left` + `Binding::Custom(touch_id)`, staged from `Touches`.
  Mouse desktop play is preserved (on native `Touches` is empty, so the custom
  input is `Bool(false)` and position falls back to the cursor).
- The one high-risk assumption -- that `Start`/`Complete` fire on a plain bool
  binding with no explicit condition -- is correct. The state-transition table
  in `bevy_enhanced_input` `src/action/events.rs:21-28,79-85` shows
  `None -> Fired => START | FIRE` and `Fired -> None => COMPLETE`, and the
  crate's own docs use `bindings![MouseButton::Left]` the same way. So
  `just_pressed` (Start) and `pressed` (Start/Complete) are wired correctly, and
  the `Last` clear gives clean one-frame edge semantics.
- Resource ordering is safe: `TouchInputId` is inserted and the action entity
  spawned in `setup` (Startup); commands flush before the frame-1 PreUpdate, so
  `stage_pointer_input`'s `Res<TouchInputId>` is always present.
- Checks all green: `cargo build`, `cargo clippy --all-targets`
  (+`--features debug`), `cargo fmt --check`, `cargo test` (+`--features debug`),
  `./scripts/check-ascii.sh`; `cargo test --example 06_fruitninja` = 19 tests
  incl. 3 new `active_pointer_pos` cases; the tests assert behavior (touch
  priority, cursor fallback, none), not just execution. Full wasm showcase build
  via `web/scripts/build-games.sh` succeeded.
- No existing tests weakened or deleted; the mouse-only geometry/combo tests are
  intact.

### Findings

- [ ] R1.1 (MINOR) examples/06_fruitninja.rs (whole diff) - the ECS input
  wiring was verified by compile + source-reading + the crate's documented state
  machine, but NOT by an interactive playtest: this ran headless, so no actual
  mouse click / touch swipe was exercised at runtime. The Outcome is honest
  about this. Suggested action: before relying on it, do a quick manual pass --
  mouse on the native `cargo run --example 06_fruitninja`, and touch on the
  built web bundle -- confirming tap-to-start, hold-swipe-slice, and
  tap-to-return. Not a blocker: the wiring matches the crate's own example
  pattern and the verified transition table, and merging/playtesting is the
  user's call anyway.
- [ ] R1.2 (NIT) examples/06_fruitninja.rs:327 - `touches.iter().next()` picks
  an arbitrary active touch (touch storage is not ordered), so with two fingers
  down the aim point could jump between them. Harmless for this single-pointer
  game, and simplest-thing-that-works is fine for an example. If multi-touch
  robustness is ever wanted, track a primary `TouchId` (first-down wins until
  released) instead of `next()`. Leave as-is otherwise.

Both findings are non-blocking (MINOR/NIT), so the round is an APPROVE. The
implementer may address R1.2 or leave it at discretion; R1.1 is a
verification-follow-up for the user, not a code change.
