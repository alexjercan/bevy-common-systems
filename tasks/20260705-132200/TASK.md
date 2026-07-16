# breach -- points + combos via scoring/streak + ui/popup

- STATUS: CLOSED
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

- Spike: tasks/20260705-132024/SPIKE.md
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

## Steps

- [x] **Add PopupPlugin + combo state.** Add `PopupPlugin` to the app. New consts
  `BASE_KILL_POINTS` and `COMBO_WINDOW`. Replace `Score(u32)` (kills-only) with
  `Score { points: u32, kills: u32 }`. New `Combo { streak: Streak, window_points: u32 }`
  resource (Default uses `Streak::new(COMBO_WINDOW)`) and a `KillFeed(Vec<u32>)` buffer.
  Register both; reset them in `start_run`.
- [x] **Score the kill by streak in `on_health_zero`.** On an enemy death: `n =
  combo.streak.hit()`, `gained = BASE_KILL_POINTS * n`, add to `score.points`, bump
  `score.kills`, add to `combo.window_points`, push `gained` into `KillFeed`. Keep the
  observer logic-only (no UI/window) so it stays headlessly testable.
- [x] **Popups + live combo readout (Playing systems).** `spawn_kill_popups` drains
  `KillFeed` and floats a "+N" via `popup()` near screen centre (window-size query,
  per-item jitter), scoped `DespawnOnExit(Playing)`. `tick_combo` ticks the streak; on
  lapse with final count >= 2 flash a "COMBO xN +P" banner popup and zero
  `window_points`. `update_combo_text` shows/hides a `ComboText` node ("COMBO xN",
  alpha from `remaining_frac`) while count >= 2.
- [x] **HUD + game over.** Status bar KILLS item -> SCORE (points); add the `ComboText`
  node. Game-over line -> "{points} pts -- {kills} kills over {waves} waves".
  `record_high_score` records `score.points`.
- [x] **Tests + verify.** Update `enemy_death_scores_one...` to assert kills==1 and
  points==BASE_KILL_POINTS; add a headless test that two kills inside the window score
  escalating points (streak multiplier) and that a lapse tick returns the final count.
  `init_resource::<Combo>()`/`KillFeed` in `death_app`. Run `cargo fmt`, `cargo clippy
  --all-targets`, `cargo test --examples`, ascii check, a `BCS_AUTOPILOT` headless run,
  and a real `cargo run` once. Update the `//!` header score line and the AGENTS module
  note for 14_breach.
