# Review: Bug discovered in dropzone game (circling rotation)

- TASK: 20260705-154507
- BRANCH: fix/dropzone-orbit-rotation

## Round 1

- VERDICT: APPROVE

Diff reviewed against `master`: `examples/08_dropzone.rs` (helper + two call
sites + 3 unit tests), `tasks/.../TASK.md`, and a `docs/` note. Full check suite
(fmt, clippy `--all-targets` with and without `debug`, `cargo test`,
`cargo test --examples`, `check-ascii.sh`, headless `BCS_AUTOPILOT=1` boot) was
green on this exact code state during the work phase; the commit added only
TASK.md/docs on top, which do not affect the build.

The root cause is correctly identified and correctly fixed. `from_rotation_arc(
Vec3::Y, up)` commits to an unstable twist about the up axis; feeding it into
both the PD attitude target and the chase camera made the hull yaw and the camera
roll when circling. Replacing it with `surface_frame(up, forward_ref)`, which
anchors the yaw to an explicit heading, is the right shape and matches the crate's
"pure, testable helper" convention. The regression test is genuine: it fails if
anyone reverts to the arc (arc twists >1 rad while `surface_frame` holds 0).

No BLOCKER/MAJOR findings. Notes below are non-blocking.

- [x] R1.1 (NIT) examples/08_dropzone.rs:1580 - the ship's PD attitude target now
  keeps the ship's *own* current heading, so there is zero yaw restoring torque
  (yaw is a free integrator). This is intentional and harmless for a lander
  (thrust is along local up, so yaw is cosmetic), and is strictly better than the
  old spurious yaw. Called out only so a future reader does not mistake the absent
  yaw authority for a bug; a one-line comment to that effect on `set_attitude_target`
  would preempt the question. Optional.
  - Response: Addressed - added the free-yaw note to `set_attitude_target`'s doc
    comment.
- [ ] R1.2 (NIT) examples/08_dropzone.rs:1587 - if the hull ever pitches to
  near-vertical (forward nearly parallel to radial up, e.g. mid-tumble), the
  `surface_frame` fallback (`any_orthonormal_vector`) makes the target heading jump
  for that frame. Unreachable in normal flight (MAX_LEAN is 0.45 rad) and the
  fallback only exists to avoid NaN, so this is acceptable; noting it for the
  record. Optional.
  - Response:
