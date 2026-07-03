# Fruit ninja: thicker gradient blade trail

- STATUS: OPEN
- PRIORITY: 85
- TAGS: feature,example

## Goal

Make the blade trail look more like a blade: give it apparent width and a
color that reads as a bright core with a cooler edge, instead of a single 1px
line.

## Steps

- [ ] In `draw_blade_trail`, for each segment compute a perpendicular offset
      (perp of the segment direction on the play plane) and draw 2-3 parallel
      lines: a bright/white center plus fainter cyan lines offset by a small
      amount on each side, so the trail looks thicker.
- [ ] Keep the existing tail->head alpha ramp; optionally shift color from cyan
      at the tail to white at the head for a hotter leading edge.
- [ ] Scale the offset width slightly by the alpha `t` so the trail tapers to a
      point at the tail.
- [ ] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, and a real boot (seed the trail as in
      the blade-trail task's verification and confirm the multi-line draw runs
      without panic).

## Notes

- `draw_blade_trail` currently draws one `gizmos.line(a+lift, b+lift, color)`
  per segment with `color = srgba(0.7, 0.95, 1.0, t)` and `lift = Vec3::Z*0.5`.
- Perp on the play plane: for a segment dir `d` (xy), the perpendicular is
  `Vec3::new(-d.y, d.x, 0).normalize() * width`; guard against zero-length
  segments.
- Gizmos are 1px regardless of camera distance, so width must be faked with
  offset lines; keep the offset small (fractions of a world unit).
- Purely cosmetic; no state changes, no new resource.
- No new dependencies.
