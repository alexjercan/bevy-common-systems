# Fruit ninja: on-screen score UI

- STATUS: CLOSED
- PRIORITY: 100
- TAGS: feature,example

## Goal

Give `examples/06_fruitninja.rs` a proper on-screen score display using Bevy
UI (a large `Text` node), instead of only the small debug status-bar item.
The score updates live as fruit is sliced.

## Steps

- [x] Add a `ScoreText` marker component and spawn a `Text` UI node in `setup`
      (top-center or top-left), styled large, initialized to "Score: 0".
- [x] Add an `update_score_text` system (run in `Update`) that, when the
      `Score` resource changes (`Res<Score>` with an `is_changed()` guard),
      writes "Score: N" into the `ScoreText` node.
- [x] Keep the existing status-bar FPS item; drop the score status-bar item now
      that there is a real HUD element, to avoid two score displays (keep FPS).
- [x] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, and a real boot showing the score
      updating (throwaway auto-slice boot as in the 06 retro).

## Notes

- Relevant files: `examples/06_fruitninja.rs` (Score resource at ~line 99, the
  status bar wiring at ~line 186).
- Bevy 0.18 UI text: `Text::new(...)`, `Node { .. }`, `TextFont`, `TextColor`.
  Check the exact spawn shape against the installed Bevy version during work.
- This task is deliberately independent and ships first; the menu task will
  later make this HUD state-scoped so it only shows while playing.
- No new dependencies.

## Close-out

Shipped the score HUD (`ScoreText` + `update_score_text` + `score_label`),
dropped the status-bar score item (FPS kept), removed the now-unused `Arc`
import. Review: 1 round, APPROVE with one NIT (stale `Score` doc) fixed in the
same round. Verified via real-GPU auto-slice boot (HUD counted up, no panic).
