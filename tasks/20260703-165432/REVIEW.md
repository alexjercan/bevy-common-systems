# Review: 08_dropzone - land a ship on the noise planet with PD controller

- TASK: 20260703-165432
- BRANCH: feature/08-dropzone

## Round 1

- VERDICT: APPROVE

The example delivers the Goal: a lunar-lander game driving `PDControllerPlugin`
(the crate's first real avian simulation), with the noise planet + trimesh
collider, radial gravity, thrust, soft/upright landing scoring, crash-explode,
skybox, post bloom, chase camera, status gauges, states, sounds and wasm
registration. Full check suite passes (fmt, check-ascii, clippy default+debug,
`cargo test`, `cargo test --features debug`). An independent read verified the
PD wiring, the FixedUpdate ordering around `PDControllerSystems::Sync`, the
crash -> fragments -> despawn ordering (fragments are spawned a frame before the
hull despawns; no race), the collision-message lifecycle (no stale misfire),
and the skybox image-before-camera ordering. No reachable panics.

No BLOCKER or MAJOR findings. The two MINORs below are user-visible camera
framing issues and are being addressed on this branch as implementer
discretion; the NITs are left as-is.

- [x] R1.1 (MINOR) examples/08_dropzone.rs:274,366-388 - `drive_chase_camera` is
  Playing-gated, so on the Menu screen `ChaseCameraInput` stays at its default
  (`anchor_pos = ZERO`) and the chase math parks the camera inside the planet
  (radius 40); the whole menu renders from inside the planet. The initial camera
  `Transform` is also dead (the plugin overwrites it frame 1). Fix: run the
  camera driver in every state, falling back to the ship's spawn vantage when no
  ship exists.
  - Response: Fixed. `drive_chase_camera` now runs unconditionally in `Update`
    and points at the ship when present, else at `ship_start_pos()` (a shared
    helper), so the menu/result screens frame the planet from above.
- [x] R1.2 (MINOR) examples/08_dropzone.rs:808-811 - related one-time swoop:
  `ChaseCameraState.anchor_pos` starts at the origin (inside the planet), so the
  first Playing frame lerps the camera out through the terrain. Fix: same as
  R1.1 - once the driver runs during Menu the state settles at the spawn vantage
  before Playing begins, so the run opens with the camera already on the ship.
  - Response: Fixed by R1.1. The camera settles on the spawn vantage during the
    menu, and the ship spawns at exactly that vantage, so Playing opens with no
    swoop.
- [ ] R1.3 (NIT) examples/08_dropzone.rs:903 - crash fragments sliced from the
  child meshes (nose, thruster) spawn at the ship's centre rather than the
  child's world offset, because `ExplodeFragments` is inserted only on the root
  and the observer uses the root transform. Purely cosmetic (all debris bursts
  from one point); left as-is.
  - Response: Acknowledged; left as-is (cosmetic, acceptable for a demo).
- [ ] R1.4 (NIT) examples/08_dropzone.rs:771 - the altimeter measures height
  above the base radius, not the terrain surface, so it reads ~+4m while resting
  on a peak. Clamped to >= 0. Cosmetic; left as-is.
  - Response: Acknowledged; left as-is. A true surface altitude would need a
    raycast/mesh sample every frame, which is not worth it for the gauge.
