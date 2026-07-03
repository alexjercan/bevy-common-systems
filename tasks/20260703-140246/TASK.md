# Fruit ninja: combo end summary

- STATUS: CLOSED
- PRIORITY: 70
- TAGS: feature,example

## Goal

When a combo ends, show a brief legible tally like "3 HIT +6" so the reward for
a combo is clear, instead of only the per-fruit "+N" popups.

## Steps

- [x] Track the points earned during the current combo: extend `Combo` with a
      `points: usize` accumulator (added to alongside `count` on each slice) and
      reset with the combo.
- [x] On combo end (the timer-expiry reset added by the time-window task), if
      `count >= 2`, spawn a centered "COMBO x{count} +{points}" summary popup
      (reuse `spawn_floating_text`, larger font, distinct color), placed near
      screen center or above the HUD.
- [x] Make sure the summary is spawned from the combo-end code path (the
      `tick_combo` reset), reading `count`/`points` before zeroing them.
- [x] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, real boot no panic.

## Notes

- Depends on: 20260703-140243 (time-window combos) -- the "combo end" event is
  the timer expiry in `tick_combo`; this task hooks that reset.
- `spawn_floating_text(commands, viewport_pos, text, size, color)` exists;
  a screen-center position can be `Vec2::new(window.width()/2.0, ...)`.
- Keep it from overlapping the per-slice "+N" popups: place the summary higher /
  larger so it reads as a distinct tally.
- No new dependencies.

## Close-out

Combo gained a `points` tally (advance_combo accumulates count; golden folds in
its +5 only when a combo is active, to avoid a leak). tick_combo, on window
expiry with count>=2, spawns a centered "COMBO xN +M" summary before resetting
count+points. points reset in start_game. Unit test covers the 1+2+3=6 tally.
