# Fruit ninja: cursor play-plane indicator

- STATUS: CLOSED
- PRIORITY: 60
- TAGS: feature,example

## Goal

Draw a small indicator where the cursor sits on the play plane so aiming reads
clearly even when the player is not actively swiping.

## Steps

- [x] Add a `draw_cursor_indicator` system (Update, `Playing`) that computes the
      cursor world position with `cursor_on_play_plane` and draws a small gizmo
      circle there (`gizmos.circle` / `circle_2d` as appropriate), lifted
      slightly toward the camera like the blade trail.
- [x] Skip drawing when the cursor is off-window (projection returns `None`).
- [x] Optionally change the indicator color/size while the left button is held
      (swiping) vs idle, for a subtle active-state cue.
- [x] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, real boot no panic.

## Notes

- `cursor_on_play_plane(window, camera, camera_transform)` returns
  `Option<Vec3>` on the z=`PLAY_Z` plane; reuse it.
- Gizmos are immediate-mode (like `draw_blade_trail`); no entity, no cleanup.
- Keep it subtle so it does not compete with the blade trail.
- No new dependencies.

## Close-out

draw_cursor_indicator draws a gizmo ring at the cursor's play-plane point
(lifted toward the camera), brighter/larger while LMB is held, skipped when the
cursor is off-window. Immediate-mode, no entity. Verified boot.
