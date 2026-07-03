# Fruit ninja: blade trail along the swipe

- STATUS: CLOSED
- PRIORITY: 100
- TAGS: feature,example

## Goal

Draw a bright "blade" trail following the cursor while the player swipes, so a
slice is visible on screen. The trail fades from head (newest, opaque) to tail
(oldest, transparent) and clears when the button is released.

## Steps

- [x] Add a `BladeTrail` resource holding a capped `VecDeque<Vec3>` of recent
      cursor world positions on the play plane (cap ~16 points). `init_resource`
      it in `main`.
- [x] In `slice_objects` (where `current` cursor world pos is already computed),
      push `current` onto `BladeTrail` and pop the front when over the cap. In
      the not-pressed branch (where `trail.previous` is cleared), also clear
      `BladeTrail` so a new swipe starts a fresh trail.
- [x] Add a `draw_blade_trail` system (run in `GameState::Playing`) that takes
      `Gizmos` and the `BladeTrail`, and draws connected segments with
      `gizmos.line(a, b, color)`, ramping alpha (and optionally color) from tail
      to head so the trail looks like a fading blade. Use a bright color
      (near-white / cyan).
- [x] Register `draw_blade_trail` in the `Playing` Update set.
- [x] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, and a real boot: swiping shows a
      fading trail that clears on release (throwaway auto-swipe if needed).

## Notes

- `Gizmos` is available: `GizmoPlugin` is in `DefaultPlugins`
  (bevy_internal default_plugins.rs). `Gizmos::line(start, end, color)` is the
  per-segment call; there is also `linestrip` / `linestrip_gradient` if a
  single call is preferred, but per-segment lines give the easiest alpha ramp.
- The play plane is z = `PLAY_Z` (0); the cursor world pos comes from
  `cursor_on_play_plane`. Drawing the line slightly toward the camera (small +z)
  can help it sit in front of fruit, but on-plane is fine to start.
- Gizmos are immediate-mode: draw every frame from the stored points, do not
  spawn entities. Avoids asset/entity churn and needs no cleanup beyond
  clearing the deque on release.
- The debug inspector / avian also use gizmos; no conflict expected.
- No new dependencies.

## Close-out

Added the blade trail: `BladeTrail` (capped `VecDeque<Vec3>`) filled from
`slice_objects` while LMB is held, cleared on release and in `start_game`;
`draw_blade_trail` renders it with `Gizmos::line` and a tail->head alpha ramp,
lifted toward the camera. Review: 1 round APPROVE; R1.1 (reset BladeTrail per
run) and R1.2 (VecDeque import) both fixed in-round. Verified on real GPU: the
gizmo draw path runs every frame with no panic.
