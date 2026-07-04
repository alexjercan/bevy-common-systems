# camera/project: screen<->world projection helpers (Wave A)

- STATUS: CLOSED
- PRIORITY: 32
- TAGS: spike,feature,camera

> Spike: docs/spikes/20260704-161210-input-and-projection-harvest.md (read first). Wave A -- smallest, unblocks the others; start here.

## Goal

Add screen<->world projection helpers over a Bevy `Camera` + `GlobalTransform`,
harvested from four games. Two are wanted:

- `pointer_on_plane(camera, gt, viewport_pos, plane) -> Option<Vec3>`: the
  `viewport_to_world` + `InfinitePlane3d` intersect that is copy-pasted
  BYTE-FOR-BYTE in `examples/06_fruitninja.rs:1515` (`pointer_on_play_plane`,
  plane at `Vec3::Z`) and `examples/10_asteroids.rs:1533` (same, plane at
  `Vec3::ZERO`).
- `world_to_screen(camera, gt, world_pos) -> Option<Vec2>`: the popup-anchoring
  glue (guarding off-screen / behind-camera) in `06:1304`, `07_orbit.rs:1042`
  and `08_dropzone.rs:1863`.

Decide the home (`camera/project` favored, camera-coupled; vs `meth`). This also
unblocks the prior spike's open "popup rendering" question by giving `ui/popup`
a blessed `world_to_screen` to track a world entity. Prove it by refactoring the
two verbatim copies (06, 10) onto `pointer_on_plane` and at least one
`world_to_screen` caller onto the helper, deleting the local copies.
