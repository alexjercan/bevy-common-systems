# Fruit ninja: cursor play-plane indicator

- STATUS: OPEN
- PRIORITY: 60
- TAGS: feature,example

## Goal

Draw a small indicator where the cursor sits on the play plane so aiming reads
clearly even when the player is not actively swiping.

## Steps

- [ ] Add a `draw_cursor_indicator` system (Update, `Playing`) that computes the
      cursor world position with `cursor_on_play_plane` and draws a small gizmo
      circle there (`gizmos.circle` / `circle_2d` as appropriate), lifted
      slightly toward the camera like the blade trail.
- [ ] Skip drawing when the cursor is off-window (projection returns `None`).
- [ ] Optionally change the indicator color/size while the left button is held
      (swiping) vs idle, for a subtle active-state cue.
- [ ] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, real boot no panic.

## Notes

- `cursor_on_play_plane(window, camera, camera_transform)` returns
  `Option<Vec3>` on the z=`PLAY_Z` plane; reuse it.
- Gizmos are immediate-mode (like `draw_blade_trail`); no entity, no cleanup.
- Keep it subtle so it does not compete with the blade trail.
- No new dependencies.
