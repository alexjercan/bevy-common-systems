# material

The `material` module is a thin set of `StandardMaterial` helpers. Right now it
exists to solve one specific footgun: building the emissive material that glowing
objects (bullets, thruster flames, pickups, reactor cores) need so they actually
bloom.

## glowing_material

`glowing_material(base_color, emissive)` returns a `StandardMaterial` with the
given `base_color` and `emissive` glow, and -- crucially -- leaves `unlit` at its
`false` default. An emissive `StandardMaterial` marked `unlit: true` skips the
lighting pass where emissive is applied, so it never glows; this helper bakes in the
correct choice so you cannot trip over it.

Use an HDR `emissive` (channel values above `1.0`) so the material streaks under
bloom.

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

fn spawn_bullet(mut materials: ResMut<Assets<StandardMaterial>>) {
    // HDR emissive (values > 1.0) streaks under bloom.
    let handle = materials.add(glowing_material(
        Color::srgb(0.1, 0.3, 0.5),
        LinearRgba::rgb(1.0, 5.0, 8.0),
    ));
    let _ = handle;
}
```

If you need extra `StandardMaterial` fields, spread the helper's output, as the
example games do:

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

fn tuned(materials: &mut Assets<StandardMaterial>) {
    let handle = materials.add(StandardMaterial {
        perceptual_roughness: 0.5,
        ..glowing_material(
            Color::srgb(0.3, 0.85, 0.4),
            LinearRgba::rgb(0.2, 2.5, 0.6),
        )
    });
    let _ = handle;
}
```

## Making it bloom

Emissive only reads as a glow once a bloom post-processing pass is on the camera.
The easiest way is `PostProcessingDefaultPlugin` from [camera](../camera/), which
adds `Bloom::NATURAL` and tonemapping to any camera tagged `PostProcessingCamera`.
Without it, HDR emissive values still render but do not streak.

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

fn build(app: &mut App) {
    app.add_plugins(PostProcessingDefaultPlugin);
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera3d::default(), PostProcessingCamera));
}
```

The higher the emissive channel values, the brighter the streak. Pair this with
procedural glowing geometry from [mesh](../mesh/) for effects like reactor cores and
projectiles.
