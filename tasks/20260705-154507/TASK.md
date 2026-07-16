# Bug discovered in dropzone game

- STATUS: CLOSED
- PRIORITY: 100
- TAGS: bug,08_dropzone,crash

When I try to go around the planet there is some weird rotation behaviour. The
camera does a weird rotation which also makes the spaceship turn around.
Explore this maybe using some unit tests and check why this might happen and
try to fix it.

## Root cause

Both the ship's PD attitude target (`set_attitude_target`) and the chase camera
frame (`drive_chase_camera`) built their orientation with
`Quat::from_rotation_arc(Vec3::Y, radial_up)`. The shortest-arc rotation from +Y
to the radial up fixes the yaw/twist *for you*, and that twist swings as the
radial up sweeps around the sphere (parallel-transport holonomy) -- unit-tested
at ~80 degrees of spurious twist by the time you round toward the far side, and
it goes singular at the -Y antipode. Since the same construction fed both the
PD target and the camera, circling the planet yawed the hull around and rolled
the camera with it, exactly as reported.

## Fix

New pure helper `surface_frame(up, forward_ref)` builds the orientation with
local +Y on `up` and local -Z on `forward_ref` projected into the tangent plane,
so the yaw is anchored to an explicit heading instead of an arbitrary arc. The
ship feeds its own current forward (target keeps the current heading, so the PD
controller no longer yaws it), and the camera feeds the hull's forward (radial
up keeps the view level on lean, heading follows the ship). Covered by four unit
tests: upright + tangent-heading invariant, degenerate forward-parallel-to-up,
and the circling regression (arc twists >1 rad while `surface_frame` stays at 0).

See `tasks/20260705-154507/NOTES.md`.

## Steps

- [x] Reproduce/understand the weird rotation and pin the root cause.
- [x] Add a pure, unit-testable helper that builds a stable surface frame.
- [x] Use it in both `set_attitude_target` and `drive_chase_camera`.
- [x] Unit-test the invariant and the circling regression.
- [x] Verify: fmt, clippy (both configs), tests, ascii, headless boot.

