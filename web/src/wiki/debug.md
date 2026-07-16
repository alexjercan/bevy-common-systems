# debug

Optional debugging and headless-verification tooling: a wireframe view, an egui
inspector, and an env-gated harness that drives a game through its state machine
or captures a screenshot. All of it lives behind the `debug` cargo feature so a
release build pays nothing.

```rust
use bevy::prelude::*;
use bevy_common_systems::debug::prelude::*;
```

## The debug feature

Everything in this module is gated on the `debug` feature. Enable it in
`Cargo.toml`:

```toml
[dependencies]
bevy_common_systems = { version = "*", features = ["debug"] }
```

...or per invocation when running an example:

```sh
cargo run --example my_game --features debug
```

The feature pulls in `bevy-inspector-egui` and avian3d physics debug plugins.
The `debug::prelude` re-exports `WireframeDebugPlugin`, `InspectorDebugPlugin`,
and the harness plugins (`AutopilotPlugin`, `ScreenshotPlugin`).

## wireframe

[`WireframeDebugPlugin`] renders every mesh in wireframe and lets you toggle it
at runtime. It registers Bevy's built-in `WireframePlugin`, inserts a
`DebugEnabled(true)` resource, and drives the global wireframe config from it:

```rust
fn build(app: &mut App) {
    app.add_plugins(WireframeDebugPlugin);
}
```

Press `F11` (the `DEBUG_TOGGLE_KEYCODE`) to toggle it. You can also flip the
`DebugEnabled` resource from your own systems.

## inspector

[`InspectorDebugPlugin`] adds a full egui inspector window ("Debug Inspector"),
plus avian3d physics gizmos and diagnostics UI. It manages the primary egui
context, keeping it on a window-targeting camera (and off any render-to-texture
camera). It also gates on its own `DebugEnabled(true)` resource, toggled with
`F11`:

```rust
fn build(app: &mut App) {
    app.add_plugins(InspectorDebugPlugin);
}
```

Because it pulls in egui and physics debug plugins, add it conditionally so a
non-debug build stays clean:

```rust
#[cfg(feature = "debug")]
app.add_plugins(InspectorDebugPlugin);
```

## The test harness

The `harness` submodule provides two env-gated plugins for exercising a game
headlessly. Both are inert unless their environment variable is set, so a game
can add them unconditionally and pay nothing in a normal run. Both drive a
`States` machine, so autopilot wins if both env vars are set at once.

[`AutopilotPlugin`] force-drives your state machine along a scripted timeline of
`(state, seconds)` steps, runs an optional per-frame input closure, logs every
transition, and exits cleanly with `AppExit::Success`. Activate it with
`BCS_AUTOPILOT`:

```rust
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

fn build(app: &mut App) {
    app.add_plugins(
        AutopilotPlugin::new()
            .hold(GameState::Menu, 0.5)
            .hold(GameState::Playing, 3.0)
            .hold(GameState::GameOver, 0.5)
            .input(|world, elapsed| {
                if elapsed > 0.5 {
                    world.resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Space);
                }
            }),
    );
}
```

[`ScreenshotPlugin`] advances to a named state, waits N settled frames, writes a
PNG, and exits. Activate it with `BCS_SHOT`; a `WxH` value (for example
`390x844`) also overrides the window resolution before the capture:

```rust
fn build(app: &mut App) {
    app.add_plugins(
        ScreenshotPlugin::new(GameState::Playing)
            .settle_frames(12)
            .path("shot.png"),
    );
}
```

Then run it, forcing a phone-width window:

```sh
BCS_SHOT=390x844 cargo run --example my_game --features debug
```

If `InspectorDebugPlugin` is present, the screenshot harness hides its overlay
first so the capture shows the game's own layout. See the
[examples](../examples/) for games wired with these plugins.
