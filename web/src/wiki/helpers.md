# helpers

The `helpers` module is a collection of small utilities and controllers:
immediate and timed entity despawning, a `bevy_enhanced_input` bridge for the
crate's unified pointer, and a WASD fly-camera controller. They are the
odds-and-ends that do not warrant their own module but show up in most games.

All snippets assume:

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
```

## DespawnEntity

`DespawnEntityPlugin` despawns any entity the same frame a `DespawnEntity` marker
is added to it, via an insert observer. It saves writing a one-off cleanup system
for one-time effects or temporary entities: mark it and it is gone.

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DespawnEntityPlugin)
        .run();
}

fn cleanup(mut commands: Commands, entity: Entity) {
    commands.entity(entity).insert(DespawnEntity); // removed this frame
}
```

## TempEntity

`TempEntityPlugin` auto-despawns an entity after a set duration. Add
`TempEntity(seconds)` and the plugin inserts an internal timer, ticks it each
frame, and despawns the entity when it finishes -- ideal for transient VFX,
one-shot decals, or a floating label with a fixed life.

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TempEntityPlugin)
        .run();
}

fn spawn_spark(mut commands: Commands) {
    commands.spawn((
        // ... your VFX bundle ...
        TempEntity(5.0), // despawns after 5 seconds
    ));
}
```

## The WASD controller

`WASDCameraControllerPlugin` binds `bevy_enhanced_input` to a fly camera: WASD
(or the left stick) for horizontal movement, mouse motion (or the right stick)
for yaw / pitch look, `Space` / `LeftShift` for vertical, and the right mouse
button to enable look while held. Add the plugin, then insert the
`WASDCameraController` marker on an entity; an observer sets up the `Camera3d`,
the `WASDCamera` (see [camera](../camera/)), and all the input bindings. Removing
the marker tears the bindings back down.

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WASDCameraControllerPlugin)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(WASDCameraController);
}
```

The module also ships `EnhancedInputPointerPlugin` (in `helpers/pointer`), the
enhanced-input counterpart to the raw [input](../input/) `UnifiedPointerPlugin`:
it drives the same `UnifiedPointer` resource from a press action. Use one or the
other, never both, since they both own that resource.
