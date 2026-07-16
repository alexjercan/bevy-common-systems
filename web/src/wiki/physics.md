# physics

The `physics` module is a thin set of helpers built on avian3d. Like the
[transform](../transform/) family they are output-only: a plugin computes a
torque or velocity into an `Output` component and your game applies it to the
avian body, so the crate never owns the physics wiring. The module also documents
a radial ("point") gravity recipe -- avian's `Gravity` is a single uniform field,
so pulling bodies toward a point is a one-liner you apply through a
`ConstantLinearAcceleration`. See `examples/08_dropzone` for the worked version.

## PD attitude controller

`PDController` torques a rigid body toward a target rotation like a critically
damped spring: give it a `frequency` (Hz), a `damping_ratio` and a `max_torque`
clamp. Add `PDControllerPlugin`. It reads the desired attitude from
`PDControllerInput(Quat)` and the body to steer from `PDControllerTarget(Entity)`,
then writes the torque to apply into `PDControllerOutput(Vec3)`. It runs in
`FixedUpdate` under `PDControllerSystems::Sync`; write the input before that set
and apply the output after it (via avian's `Forces::apply_torque`). The controller
scales its raw PD term by the body's world-space inertia tensor, so it despins
off-principal spins correctly.

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
use avian3d::prelude::*;

fn setup(app: &mut App) {
    app.add_plugins(PhysicsPlugins::default())
        .add_plugins(PDControllerPlugin);
}

fn spawn_ship(mut commands: Commands) {
    let body = commands.spawn((RigidBody::Dynamic, Transform::default())).id();
    commands.spawn((
        ChildOf(body),
        PDController { frequency: 4.0, damping_ratio: 4.0, max_torque: 40.0 },
        PDControllerTarget(body),
        Transform::default(),
    ));
}

// Write the desired attitude BEFORE PDControllerSystems::Sync ...
fn command(mut q: Query<&mut PDControllerInput>) {
    for mut input in &mut q {
        **input = Quat::IDENTITY; // hold upright
    }
}

// ... and apply the torque AFTER it.
fn apply(
    mut bodies: Query<Forces>,
    q: Query<(&PDControllerOutput, &PDControllerTarget)>,
) {
    for (output, target) in &q {
        if let Ok(mut forces) = bodies.get_mut(**target) {
            forces.apply_torque(**output);
        }
    }
}
```

## Doom character controller

`DoomController` is a pragmatic, arena-shooter first-person mover (mouse-look
with a pitch clamp plus planar WASD-style movement) -- flat ground, no jump,
crouch, or air control. Add `DoomControllerPlugin`. Put the config on the physics
body and spawn a `DoomEye`-marked camera child that carries the view. Each frame
write `DoomControllerInput { look, movement }` (look is a raw delta, movement is a
`(strafe, forward)` intent) before `DoomControllerSystems::Drive`, and copy
`DoomControllerOutput::velocity` into the body's velocity after it, keeping the
vertical component for gravity. Lock the body's rotation
(`LockedAxes::ROTATION_LOCKED`) so only the eye rotates. This is the controller
harvested from `examples/14_breach`.

```rust
fn spawn_player(mut commands: Commands) {
    commands
        .spawn((
            DoomController::default(),
            Transform::default(),
            // your body: RigidBody::Dynamic, a capsule Collider,
            // LockedAxes::ROTATION_LOCKED.
        ))
        .with_children(|parent| {
            parent.spawn((Camera3d::default(), DoomEye, Transform::from_xyz(0.0, 0.6, 0.0)));
        });
}

fn input(mut q: Query<&mut DoomControllerInput>) {
    let mouse_delta = Vec2::new(4.0, 0.0); // from your mouse-motion reader
    for mut input in &mut q {
        input.look = mouse_delta;
        input.movement = Vec2::new(0.0, 1.0); // walk forward
    }
}

fn apply(mut q: Query<(&DoomControllerOutput, &mut LinearVelocity)>) {
    for (output, mut vel) in &mut q {
        vel.0.x = output.velocity.x;
        vel.0.z = output.velocity.z; // keep vel.0.y for gravity
    }
}
```

The free function `doom_move_dir(yaw, movement)` gives the world-space,
ground-plane move direction for a yaw, useful for driving other movers.

## Rigid bodies

Two small avian helpers.

`rigid_body_point_velocity(linear_velocity, angular_velocity, center_of_mass,
point)` returns the world velocity of a point rigidly attached to a moving body
(`v = v_linear + omega x (p - com)`). The canonical use is muzzle velocity: a shot
inherits the full motion of its muzzle, including the swing from the body's spin,
not just the linear velocity. All arguments must be in the same (world) frame;
avian's `ComputedCenterOfMass` is body-local, so transform it first with
`body_transform.transform_point(*center_of_mass)`.

```rust
// A muzzle offset from a spinning ship's centre inherits the swing.
let muzzle = rigid_body_point_velocity(linear, angular, com_world, muzzle_world);
```

`destructible_body(health, density)` returns a `Bundle` bundling a `Health` pool,
avian's `ColliderDensity`, and inherited `Visibility` -- the shared makeup of a
destructible physics object. Pair it with a `Collider` on an entity parented to a
`RigidBody`, and drive the destruction with the
[integrity](../integrity/) pipeline.

```rust
fn spawn_asteroid(mut commands: Commands) {
    commands.spawn((
        destructible_body(100.0, 1.0),
        Collider::sphere(2.0),
        RigidBody::Dynamic,
    ));
}
```
