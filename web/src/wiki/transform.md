# transform

The `transform` module is a family of small "driver" plugins that compute an
orientation or position for you and write it into an `Output` component. You own
the `Transform`: each frame you copy the resolved `Output` onto it (usually in a
system ordered `.after(...Systems::Sync)`). Every driver follows the same shape:
a config component you spawn, an optional `Input` you write each frame, and an
`Output` you read. They build on [meth](../meth/) (`spherical_to_cartesian`,
`LerpSnap`) for the underlying math.

## Sphere orbit

`SphereOrbit` moves an entity across the surface of a sphere by two angles,
`theta` (azimuth) and `phi` (elevation). You write the target angles into
`SphereOrbitInput`; the plugin smooths the state toward them (via `LerpSnap`,
controlled by `smoothing`) and writes the world position into
`SphereOrbitOutput`. Add `SphereOrbitPlugin`; the config's `initial_theta` /
`initial_phi` / `center` seed the starting `Input`, state and `Output`.

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

fn setup(mut app: &mut App) {
    app.add_plugins(SphereOrbitPlugin);
}

fn spawn_orbiter(mut commands: Commands) {
    commands.spawn((
        SphereOrbit {
            radius: 5.0,
            center: Vec3::ZERO,
            initial_theta: 0.0,
            initial_phi: 0.0,
            smoothing: 0.5,
        },
        Transform::default(),
    ));
}

// Steer it, then copy the resolved position after SphereOrbitSystems::Sync.
fn drive(mut q: Query<&mut SphereOrbitInput>) {
    for mut input in &mut q {
        input.theta += 0.01;
    }
}

fn apply(mut q: Query<(&SphereOrbitOutput, &mut Transform)>) {
    for (output, mut transform) in &mut q {
        transform.translation = **output;
    }
}
```

## Directional and random orbit

Two variants trade the angle `Input` for other ways of choosing the target
point on the sphere.

`DirectionalSphereOrbit` takes a direction vector instead of angles: write a
`DirectionalSphereOrbitInput(Vec3)` and the plugin maps it (via
`direction_to_spherical`) to the surface point that direction intersects,
smoothing there and writing `DirectionalSphereOrbitOutput`. This is exactly how
`examples/07_orbit` steers its player: it advances a surface frame, writes the
"up" direction into the input, then reads the output onto the transform.

```rust
fn spawn_runner(mut commands: Commands) {
    commands.spawn((
        DirectionalSphereOrbit {
            radius: 5.0,
            center: Vec3::ZERO,
            direction: Vec3::Z,
            smoothing: 0.0,
        },
        Transform::default(),
    ));
}

fn steer(mut q: Query<&mut DirectionalSphereOrbitInput>) {
    for mut input in &mut q {
        input.0 = Vec3::Y; // the orbit resolves the surface point in this direction
    }
}
```

`RandomSphereOrbit` needs no `Input` at all: add `SphereRandomOrbitPlugin` and it
picks fresh random target angles once the previous ones are reached, moving at
`angular_speed` radians per second and writing `RandomSphereOrbitOutput`. The
same example drives its wandering hazards and orbs this way.

```rust
fn spawn_hazard(mut commands: Commands) {
    commands.spawn((
        RandomSphereOrbit {
            radius: 5.0,
            angular_speed: 0.4,
            center: Vec3::ZERO,
            initial_theta: 0.0,
            initial_phi: 0.0,
        },
        Transform::default(),
    ));
}
```

## Point rotation

`PointRotation` accumulates a rotation from per-frame 2D deltas (typically mouse
motion), rotating around the entity's local axes so yaw and pitch stay intuitive
under any orientation. Add `PointRotationPlugin`, write the delta into
`PointRotationInput(Vec2)` (x = yaw, y = pitch), and read the accumulated
`PointRotationOutput(Quat)`. The config's `initial_rotation` seeds the output.

```rust
fn spawn_head(mut commands: Commands) {
    commands.spawn((PointRotation::default(), Transform::default()));
}

fn look(mut q: Query<&mut PointRotationInput>) {
    for mut input in &mut q {
        **input = Vec2::new(0.01, 0.0); // yaw a little this frame
    }
}

fn apply(mut q: Query<(&PointRotationOutput, &mut Transform)>) {
    for (output, mut transform) in &mut q {
        transform.rotation = **output;
    }
}
```

## Smooth look rotation

`SmoothLookRotation` eases a single angle around a fixed `axis` toward a target
at a capped angular `speed`, with optional `min` / `max` limits. Add
`SmoothLookRotationPlugin`, write the desired angle into
`SmoothLookRotationTarget(f32)`, and read `SmoothLookRotationOutput(f32)`, which
you can turn into a rotation with `Quat::from_axis_angle(axis, output)`. The
target and output are seeded from the config's `initial`.

```rust
fn spawn_turret(mut commands: Commands) {
    commands.spawn((
        SmoothLookRotation {
            axis: Vec3::Y,
            initial: 0.0,
            speed: std::f32::consts::PI, // 180 deg/s
            min: None,
            max: None,
        },
        Transform::default(),
    ));
}

fn aim(mut q: Query<&mut SmoothLookRotationTarget>) {
    for mut target in &mut q {
        **target = std::f32::consts::FRAC_PI_2;
    }
}

fn apply(mut q: Query<(&SmoothLookRotation, &SmoothLookRotationOutput, &mut Transform)>) {
    for (look, output, mut transform) in &mut q {
        transform.rotation = Quat::from_axis_angle(look.axis, **output);
    }
}
```
