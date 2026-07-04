# Review: 10_asteroids - slice-on-hit shooter with physics-body fragments

- TASK: 20260703-170744
- BRANCH: feature/10-asteroids

## Round 1

- VERDICT: REQUEST_CHANGES

Solid example: it delivers the Goal (a hit slices a rock and the shards respawn
as real avian bodies), the headless integration test proves that exact path, the
whole check suite is green, and the design (kinematic ship, sensor bullets,
collision layers, generation cap, aspect-fit camera) is well reasoned and
documented. Three real bugs in the visual layer keep it from an approve; none
touch the core physics-fragment mechanic.

- [x] R1.1 (MAJOR) examples/10_asteroids.rs:`apply_camera_shake` - the shake adds
  to the camera translation (`transform.translation.x += offset.x`) and, when
  trauma has decayed to zero, `return`s without ever restoring the base
  position. So the random per-frame offsets accumulate and the camera
  permanently drifts off-center over a run (and never re-centers between shakes).
  The arena is centered on the origin, so the fix is to write the offset
  absolutely from a zero base each frame and reset to zero when idle: compute
  `offset = Vec2::ZERO` when `amount <= 0.0`, then
  `transform.translation.x = offset.x; transform.translation.y = offset.y;`
  (mirrors `06_fruitninja`'s `translation = CAMERA_BASE + offset`). `fit_camera`
  already owns `z`, so only x/y should be touched.
  - Response: Fixed. `apply_camera_shake` now computes `offset` (zero when idle)
    and writes it absolutely to x/y, so shakes never accumulate and the camera
    re-centers.
- [x] R1.2 (MAJOR) examples/10_asteroids.rs:`spawn_ship` - the thruster flame is
  spawned as a direct child of the ship *sibling* to the model, but the ship
  body's rotation is never changed (only the model child is rotated to the
  heading in `control_ship`). So the flame stays pinned to ship-local -Y (screen
  down) regardless of which way the ship faces, instead of trailing behind the
  nose. Nest the flame under the `ShipModel` entity (so it inherits the model's
  heading rotation) rather than under the ship. Its local transform
  `(0, -1.1, 0)` rot PI is then correct relative to the cone.
  - Response: Fixed. The flame is now spawned via `children![..]` on the
    `ShipModel` entity, so it inherits the model's heading rotation.
- [x] R1.3 (MAJOR) examples/10_asteroids.rs:`setup` (bullet_material) and
  `spawn_ship` (flame_material) - both set `unlit: true` alongside an HDR
  `emissive`. In Bevy's PBR shader `unlit` takes `out.color = base_color` and
  skips `apply_pbr_lighting` entirely, which is where emissive is added
  (verified in `bevy_pbr` `render/pbr.wgsl:81-85`). So the emissive is discarded,
  the surfaces render at their LDR base color, and they do not bloom -- defeating
  the task's "`camera/post` bloom on the shots" requirement. Remove `unlit: true`
  from both materials so the HDR emissive is applied and picked up by bloom.
  - Response: Fixed. Removed `unlit: true` from both the bullet and flame
    materials (with a comment explaining why), so the HDR emissive is applied
    and blooms.

## Round 2

- VERDICT: APPROVE

All three round-1 findings verified fixed against the new diff (commit bf011a0):
`apply_camera_shake` now writes the offset absolutely from a zero base and
re-centers when idle (R1.1); the flame is nested under `ShipModel` via
`children![..]` so it inherits the heading rotation (R1.2); `unlit` is gone from
both emissive materials so the HDR emissive blooms (R1.3). No new issues
introduced. Full check suite (build, clippy both feature configs, fmt, test,
check-ascii) is green, the 9 in-file tests pass, and the example runs to the
render loop with no panic or B0004. Ships.
