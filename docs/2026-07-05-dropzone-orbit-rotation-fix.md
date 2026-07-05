# 08_dropzone: weird rotation when flying around the planet

Task: `tasks/20260705-154507` (bug, 08_dropzone, crash).

## Symptom

Flying the lander laterally to go "around the planet" made the camera do a weird
rotation that also turned the spaceship around, as if some yaw was being applied
that nobody asked for.

## Diagnosis

The lander lives on a sphere with radial gravity, so both the ship's upright
attitude and the chase camera's frame are built relative to the local "up"
(`radial_up = position.normalize()`). Both did it the obvious way:

```rust
let upright = Quat::from_rotation_arc(Vec3::Y, radial_up);
```

`from_rotation_arc(a, b)` returns the *shortest* rotation taking `a` to `b`. That
is fine for the up axis, but it also silently commits to a yaw/twist about that
up axis -- the one that happens to fall out of the shortest arc. That twist is
not stable as `radial_up` moves:

- **Holonomy.** As `radial_up` sweeps around the sphere (you fly around), the
  shortest-arc frame twists about up. By the time you round toward the far side
  it is off by roughly 80 degrees (measured in the unit test), with no steering
  input at all.
- **Antipode singularity.** At the -Y antipode the arc is undefined and flips.
  Gameplay keeps the landing pad within ~0.5 rad of the +Y pole, but the player
  can fly anywhere, so the twist is reachable and grows without bound toward the
  far pole.

Because the *same* `from_rotation_arc` expression fed both the PD attitude target
(`set_attitude_target`) and the camera frame (`drive_chase_camera`), the twist
showed up in both: the PD controller yawed the hull to chase the swinging target,
and the camera rolled with it. Two symptoms, one cause.

## Fix

A pure helper that anchors the yaw to an explicit heading instead of an arbitrary
arc:

```rust
fn surface_frame(up: Vec3, forward_ref: Vec3) -> Quat {
    let up = up.normalize_or(Vec3::Y);
    let mut fwd = forward_ref - up * forward_ref.dot(up); // project into tangent plane
    if fwd.length_squared() < 1e-6 {
        fwd = up.any_orthonormal_vector();                // degenerate fallback
    }
    let fwd = fwd.normalize();
    let right = fwd.cross(up).normalize();                // local +X
    Quat::from_mat3(&Mat3::from_cols(right, up, -fwd))    // +Y = up, -Z = fwd
}
```

- **Ship** (`set_attitude_target`): `forward_ref` is the ship's *own* current
  forward, so the upright target keeps the current heading. The PD controller
  then only corrects tilt (up alignment + lean), never yaw -- the hull stops
  turning around.
- **Camera** (`drive_chase_camera`): `forward_ref` is the hull's forward too.
  Radial up keeps the view from rolling when you lean; the heading follows the
  ship, so circling the planet no longer spins the camera.

## Why `from_mat3` handedness matters

The first cut used `right = up.cross(fwd)`, which makes the basis left-handed
(det -1); `Quat::from_mat3` on an improper matrix returns garbage (the upright
test failed immediately with "up axis not radial"). The correct right vector for
columns `(right, up, -fwd)` to be a proper rotation is `right = fwd.cross(up)`.

## Tests

Four unit tests in `examples/08_dropzone.rs`, all off the ECS:

- `surface_frame_is_upright_with_tangent_heading` -- local +Y maps to the radial
  up, forward is a unit tangent aligned with the projected heading (several ups,
  including just shy of the antipode).
- `surface_frame_handles_forward_parallel_to_up` -- degenerate input stays finite
  and upright.
- `surface_frame_adds_no_twist_while_circling_the_planet` -- the regression:
  sweeping from the +Y pole toward the -Y antipode, `from_rotation_arc` twists
  more than a radian while `surface_frame` stays at zero twist.

## Verification

`cargo fmt --check`, `cargo clippy --all-targets` (+`--features debug`),
`cargo test`, `cargo test --examples`, `scripts/check-ascii.sh`, and a headless
`BCS_AUTOPILOT=1` boot (Menu -> Playing -> Result, no panic).

## Reflection

- The bug was invisible to every automated gate before this: it compiles, clippy
  is clean, and the autopilot force-drives states without ever flying laterally,
  so it never circled the planet. The fix is only trustworthy because the yaw
  logic was pulled into a pure function and unit-tested against the geometry --
  the same lesson the glide retro records (make rendering/attitude-driver logic
  pure and test it, do not lean on a screenshot or a state-entry autopilot).
- `from_rotation_arc(Y, up)` is a recurring footgun for "upright on a sphere":
  it is correct for the axis but commits to an unstable twist. When a full
  orientation is needed on a surface, always supply an explicit heading.
