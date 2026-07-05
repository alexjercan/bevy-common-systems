# breach -- points + combos via scoring/streak + ui/popup

- STATUS: OPEN
- PRIORITY: 70
- TAGS: spike,breach,example,juice

## Goal

Turn `14_breach`'s flat kill count into a rewarding points-and-combos loop, and
be the crate's first FPS/3D user of `scoring/streak`.

Today `Score(u32)` just `+= 1` per kill. Instead: on each kill bump a
`scoring/streak` `Streak` (decaying combo counter, shared bookkeeping behind
fruitninja `Combo` / orbit `Streak`), score the kill by the current streak
(game-owned value rule -- e.g. base points * streak), and float a "+N" via
`ui/popup`. When the streak window lapses and the final count was >= 2, flash a
"COMBO xN" banner (see `06_fruitninja` for the exact pattern). Keep persisting the
best as `HighScore<u32>`. Reward fast, aggressive play so the game leans into the
swarm threat rather than passive kiting.

## Notes

- Spike: docs/spikes/20260705-132024-breach-fun-pass.md
- Reuse `scoring/streak` (do NOT re-implement decay; it owns count+decay only,
  the game owns the value rule) and `ui/popup`. Model the wiring on
  `06_fruitninja` (`Combo`, `tick_combo`, "+N"/"COMBO xN" popups).
- Pure logic (the streak-to-points value rule) gets an in-module `#[cfg(test)]`
  test. If the readout/banner is driven by a pure classifier, test that, not just
  the score number (glide moves-list lesson).
- Copy Bevy 0.19 UI idioms from an existing example (font_size/TextLayout/
  border_radius); do not improvise the visual layer.
- Verify: `cargo clippy --all-targets`, then `BCS_AUTOPILOT=1 ... --features debug`
  under timeout, then run the example once for real.
