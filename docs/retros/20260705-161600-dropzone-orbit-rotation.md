# Retro: 08_dropzone circling rotation bug

- TASK: 20260705-154507
- BRANCH: fix/dropzone-orbit-rotation
- REVIEW ROUNDS: 1 (APPROVE)

See `tasks/20260705-154507/TASK.md` and
`docs/2026-07-05-dropzone-orbit-rotation-fix.md` for what changed and why.
This retro is about how the working went.

## What went well

- Root cause fell out of reading the two *symptom* sites (camera + ship) side
  by side and noticing they shared one expression: `from_rotation_arc(Vec3::Y,
  radial_up)`. "The camera rotation makes the ship turn too" pointed straight at
  a shared cause rather than two bugs, and grepping for the rotation math found
  it in minutes.
- Pulling the yaw logic into a pure `surface_frame(up, forward_ref)` made the
  whole thing unit-testable off the ECS, which is the only reason the fix is
  trustworthy: every automated gate that existed before (build, clippy, the
  force-transition autopilot) was blind to this bug because none of them fly the
  ship laterally around the planet.
- The upright invariant test caught a real implementation bug immediately (see
  below) instead of it shipping.

## What went wrong

- Handedness slip: the first `surface_frame` used `right = up.cross(fwd)`, giving
  a left-handed basis (det -1); `Quat::from_mat3` on an improper matrix returns
  garbage and the "up axis is radial" assertion failed on the very first run.
  Root cause: hand-derived the cross-product sign instead of just letting the
  test tell me. Cheap because the invariant test was written first; the lesson is
  that basis handedness for `from_mat3` is worth a one-line numeric check, not a
  mental derivation.
- The regression test took three tries to actually observe the bug. First cut
  straddled the -Y antipode with a probe vector (`NEG_Z`) that happened to be
  parallel to the arc's rotation axis, so the measured swing was exactly 0 -- the
  test "passed" the wrong thing. Second cut circled at a realistic *small* polar
  angle (theta=0.4) where the arc twist is only ~2 degrees, far too weak to
  assert on. Root cause: I equated "circling the planet" with small-azimuth
  motion near the pole, but the user's "go around the planet" means large polar
  angle (out past the equator), where the holonomy twist grows to ~80 degrees.
  Only after *measuring* the twist across a real circumnavigation did the test
  demonstrate the bug. Lesson: a regression test must be shown to fail against
  the old code / observe the effect, by measuring the number, before trusting its
  threshold -- a green regression test can be green because it is looking in the
  wrong place.

## What to improve next time

- When a bug is "spurious rotation on a sphere", suspect `from_rotation_arc(Y,
  up)` first: it is correct for the axis but silently commits to an unstable
  twist about it. This is a reusable footgun worth an AGENTS.md gotcha.
- For any regression test whose job is to prove a specific bug, print/measure the
  quantity across the input range and confirm the *old* behavior actually trips
  the assertion before locking in the threshold. Do not assume the effect is
  large at the first parameters you pick.

## Action items

- [x] Proposed AGENTS.md gotcha: `from_rotation_arc(Y, up)` as the "upright on a
  sphere" footgun (unstable twist / antipode singularity), with `surface_frame`
  as the fix pattern.
- [ ] Possible future harvest (waiting for a second user, per crate convention):
  `surface_frame` is a candidate for `meth` or `camera` once another example
  needs a stable surface-relative frame -- `07_orbit` already does sphere work
  but drives its camera differently. Not a task yet; noted here.
