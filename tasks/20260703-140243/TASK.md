# Fruit ninja: time-window combos

- STATUS: OPEN
- PRIORITY: 78
- TAGS: feature,example

## Goal

Rework combos so they persist through slow moments and across separate swipes:
after each slice you have a short window to land the next hit and keep the
combo going, even if the blade slowed or you re-swiped. Holding still still
cannot farm, because slicing itself remains gated on swipe speed.

## Steps

- [ ] Extend `Combo` with a timer: `Combo { count: usize, timer: f32 }` (seconds
      remaining in the window). Add a `COMBO_WINDOW` const (e.g. 1.2s).
- [ ] On slicing a fruit, `advance_combo` still increments `count`; also refresh
      `timer = COMBO_WINDOW`.
- [ ] Add a `tick_combo` system (Update, `Playing`) that counts `timer` down by
      `dt`; when it reaches 0 with `count > 0`, reset the combo to 0 (this is
      the combo-end event the summary task will hook).
- [ ] Remove the combo reset that currently happens on swipe stall
      (`slice_objects` :663) and on button release (:634) -- the timer now owns
      combo lifetime. KEEP the swipe-speed gate on *slicing* itself
      (`swipe_is_active`) so holding still still slices nothing; only the combo
      reset moves to the timer. Keep the `start_game` combo reset.
- [ ] Update the module `//!` doc and the combo comment to describe the
      time-window behavior.
- [ ] Add/adjust unit tests: the escalation test still holds; add a small pure
      helper if useful (e.g. a `combo_expired(timer)` predicate) and test it.
- [ ] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, real boot no panic.

## Notes

- Current combo reset sites: `slice_objects` release branch (:634) and stall
  branch (:663), plus `start_game` (:489). `advance_combo` at :200.
- Rationale for keeping the swipe gate: the earlier anti-cheese
  (20260703 swipe-speed gate) prevents hold-to-farm by requiring motion to
  slice; this task only changes when the *combo counter* resets, so no cheese
  returns.
- This task is a prerequisite for the golden-fruit combo-time bonus
  (20260703-140244) and the combo end summary (20260703-140246); do it first.
- No new dependencies.
