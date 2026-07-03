# Review: Fruit ninja on-screen score UI

- TASK: 20260703-121414
- BRANCH: feature/fruitninja-score-ui

## Round 1

- VERDICT: APPROVE

Small, focused diff. The score is now a large top-left `Text` HUD updated via
`update_score_text` (guarded by `Score::is_changed()`), the redundant
status-bar score item is gone (FPS kept), and the unused `Arc` import was
removed. Checks clean (`fmt`, `clippy --all-targets` both feature configs,
`check-ascii`); a real-GPU auto-slice boot confirmed the HUD text updating
0 -> 1 -> 2 -> 3 -> 4 with no panic. `score_label` is factored so the initial
text and the updated text cannot drift.

- [x] R1.1 (NIT) examples/06_fruitninja.rs:102 - `Score` doc still said "Shown
  in the status bar"; the score now lives in the HUD. Fixed in this round to
  "Shown in the score HUD".
