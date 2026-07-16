# meth

The `meth` module is pure math utility: a smoothing trait (`LerpSnap`),
spherical-coordinate conversions, and a great-circle `slerp`. It has no plugin --
just functions and one trait you call directly. The [transform](../transform/)
orbit drivers are built on these, and [tween](../tween/) covers the complementary
"ease A to B over N seconds" case that `LerpSnap` does not.

## LerpSnap

`LerpSnap` is a trait (implemented for `f32` and `Vec3`) for frame-rate-independent
exponential smoothing toward a moving target. `lerp_and_snap(to, smoothness, dt)`
eases `self` toward `to`; `smoothness` is in `0.0..=1.0` (0 = instant, 1 = very
smooth) and the interpolation is scaled by `dt` so it behaves the same at any
frame rate. When the value gets within an epsilon of the target (and smoothing is
below 1) it snaps exactly, avoiding an endless asymptotic tail.

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

fn follow(time: Res<Time>, mut q: Query<(&mut Transform, &Target)>) {
    let dt = time.delta_secs();
    for (mut transform, target) in &mut q {
        transform.translation = transform.translation.lerp_and_snap(target.0, 0.5, dt);
    }
}

# #[derive(Component)]
# struct Target(Vec3);
```

The sphere orbit drivers use this to smooth their angles toward the input target.

## Spherical coordinates

Two conversions bridge angles and Cartesian directions, using the crate's
convention: `theta` is the azimuth around +Y measured from -Z, and `phi` is the
elevation from the horizontal plane.

`spherical_to_cartesian(radius, theta, phi) -> Vec3` places a point on a sphere.
At `theta = 0, phi = 0` it points down -Z; `phi = PI/2` points up +Y.

`direction_to_spherical(direction) -> (theta, phi)` is the inverse: it reads the
`(theta, phi)` angles from a direction vector (a zero vector yields `(0, 0)`).

```rust
// A point 5 units out, straight ahead (-Z).
let pos = spherical_to_cartesian(5.0, 0.0, 0.0);

// Recover the angles that face +X.
let (theta, phi) = direction_to_spherical(Vec3::X);
```

These are the math behind the orbit drivers: `SphereOrbit` uses
`spherical_to_cartesian` to turn its angles into a world position, and
`DirectionalSphereOrbit` uses `direction_to_spherical` to map a steering direction
onto the sphere.

## slerp

`slerp(a, b, t) -> Vec3` spherically interpolates between two vectors along the
great circle connecting them, returning a unit vector. Unlike a plain `lerp`
(which cuts a straight chord and shrinks toward the middle), `slerp` keeps a
constant radius and constant angular speed -- the right tool for blending
directions or moving along a sphere's surface.

```rust
// Halfway along the arc from +X to +Y is the 45-degree diagonal.
let mid = slerp(Vec3::X, Vec3::Y, 0.5);
// mid is roughly Vec3::new(0.707, 0.707, 0.0), on the unit circle.
```
