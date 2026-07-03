# Fruit ninja: fruit visual variety (scale and spin)

- STATUS: OPEN
- PRIORITY: 90
- TAGS: feature,example

## Goal

Make the field less uniform: each fruit spawns at a random size and tumbles at
its own random spin rate, so launches feel varied rather than identical.

## Steps

- [ ] In `spawn_projectile`, pick a random scale in a modest range (e.g.
      0.8..1.3) and set it on the spawned `Transform` (`with_scale`). Scale the
      `Sliceable.radius` by the same factor so hit detection matches the visible
      size.
- [ ] Give each projectile its own spin: add per-entity spin rates (extend
      `Projectile` with `spin: Vec2` or add a `Spin` component) set randomly at
      spawn, and use them in `move_projectiles` instead of the fixed
      `rotate_local_x(dt*1.5)` / `rotate_local_y(dt*2.0)`.
- [ ] Bombs: keep them visually distinct -- either exclude bombs from the scale
      jitter or give them a tighter range so they stay unmistakable.
- [ ] Fragments: note the interaction -- `on_fragments_spawned` spawns fragments
      at the shell's world position at scale 1; if scaled fruit look wrong when
      they burst, apply the fruit's scale to the fragment `Transform` too.
- [ ] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, real boot with no panic.

## Notes

- `spawn_projectile` at :543 sets `Sliceable { radius: FRUIT_RADIUS }`,
  `Projectile { velocity }`, `Transform::from_xyz(...)`.
- `move_projectiles` applies the fixed tumble; `on_fragments_spawned` builds
  fragments from `fragment.mesh` at the origin translation.
- Slicing still requires the mesh centered at the origin (unchanged; scaling is
  uniform so the octahedron stays centered).
- No new dependencies.
