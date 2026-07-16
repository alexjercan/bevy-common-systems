# camera

The `camera` module collects the camera systems a 3D game reaches for again and
again: a smooth chase camera, a free WASD flycam, default bloom/tonemapping post
processing, cubemap skyboxes, trauma-driven screen shake, and screen <-> world
projection helpers. Each follows the crate convention of a config component plus a
plugin, and (where relevant) a separate input component your game writes to.

## chase

`ChaseCamera` is a third-person follow camera. It reads an "anchor frame" from
`ChaseCameraInput` (the target `anchor_pos` and `anchor_rot`), places itself at
`offset` in that frame, looks toward `focus_offset` ahead of the target, and lerps
there using `smoothing`. Add `ChaseCameraPlugin`, spawn a camera with `ChaseCamera`,
and update `ChaseCameraInput` each frame from your controller.

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

fn build(app: &mut App) {
    app.add_plugins(ChaseCameraPlugin);
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        ChaseCamera {
            offset: Vec3::new(0.0, 4.5, -11.0),
            focus_offset: Vec3::new(0.0, 1.5, 16.0),
            smoothing: 0.1,
        },
    ));
}

fn drive(mut q: Query<&mut ChaseCameraInput>, target: (Vec3, Quat)) {
    for mut input in &mut q {
        input.anchor_pos = target.0;
        input.anchor_rot = target.1;
    }
}
```

The plugin runs in the `ChaseCameraSystems::Sync` set, which the shake plugin orders
itself around. See `examples/07_orbit.rs` and `examples/08_dropzone.rs`.

## wasd

`WASDCamera` is a free-look flycam: `look_sensitivity` for mouse rotation and
`wasd_sensitivity` for movement. Add `WASDCameraPlugin`, and each frame write the
`WASDCameraInput` fields `pan` (mouse delta), `wasd` (planar move), and `vertical`.

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

fn build(app: &mut App) {
    app.add_plugins(WASDCameraPlugin);
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        WASDCamera { look_sensitivity: 0.1, wasd_sensitivity: 0.5 },
    ));
}
```

If you do not want to wire up input by hand, the `helpers` module ships a
`WASDCameraController` component and `WASDCameraControllerPlugin` that read the real
mouse and keyboard into `WASDCameraInput` for you -- see
`examples/01_sphere.rs` and `examples/02_planet.rs`.

## post

`PostProcessingDefaultPlugin` gives cameras a sensible HDR look. When you tag a
camera with `PostProcessingCamera`, an observer inserts `Tonemapping::TonyMcMapface`
and `Bloom::NATURAL`, so HDR emissive materials (from [material](../material/))
bloom.

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

For different defaults, insert `Tonemapping` and `Bloom` yourself instead of using
this plugin.

## skybox

`SkyboxConfig` turns a stacked cubemap image into a rendered skybox on a camera.
Add `SkyboxPlugin`, then insert `SkyboxConfig` on the camera with the `cubemap`
image handle and a `brightness` multiplier; an observer reinterprets the image as a
cube texture and attaches Bevy's `Skybox`.

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

fn build(app: &mut App) {
    app.add_plugins(SkyboxPlugin);
}

fn spawn_camera(mut commands: Commands, assets: Res<AssetServer>) {
    let starfield = assets.load("skybox.png");
    commands.spawn((
        Camera3d::default(),
        SkyboxConfig { cubemap: starfield, brightness: 130.0 },
    ));
}
```

`SkyboxConfig` requires a `Camera`. See `examples/08_dropzone.rs`,
`examples/12_bastion.rs`, and `examples/14_breach.rs`.

## shake

`CameraShake` implements classic trauma shake with no drift. Game code adds trauma
through `CameraShakeInput::add_trauma`; the plugin decays it by `decay` per second
and, while trauma is positive, offsets the camera by up to `max_offset` (translation)
and `max_kick` (rotation), scaled by `trauma^exponent`. It applies and un-applies the
offset in the `Restore`/`Apply` sets around any base driver, so it composes with
`ChaseCamera` automatically.

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

fn build(app: &mut App) {
    app.add_plugins(CameraShakePlugin);
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        CameraShake { max_offset: Vec3::splat(0.4), ..default() },
    ));
}

fn on_hit(mut q: Query<&mut CameraShakeInput>) {
    for mut input in &mut q {
        input.add_trauma += 0.5; // set `reset = true` to clear on restart
    }
}
```

Read the current shake from `CameraShakeOutput` if you want a HUD to react. See
`examples/12_bastion.rs`.

## project

Two free functions convert between world and screen space. `pointer_on_plane`
casts a viewport pointer position into the world and intersects it with an infinite
plane -- ideal for picking a gameplay point on a play plane from the cursor.
`world_to_screen` does the reverse, returning the pixel position of a world point (or
`None` when it is off-screen or behind the camera) so you can anchor UI over an
entity.

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

fn pick(camera: &Camera, cam_tf: &GlobalTransform, pointer: Vec2) -> Option<Vec3> {
    pointer_on_plane(camera, cam_tf, pointer, Vec3::ZERO, InfinitePlane3d::new(Vec3::Z))
}

fn anchor_ui(camera: &Camera, cam_tf: &GlobalTransform, world_pos: Vec3) -> Option<Vec2> {
    world_to_screen(camera, cam_tf, world_pos)
}
```
