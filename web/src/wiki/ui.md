# ui

The `ui` module is a grab-bag of screen-space UI building blocks that every
game in this crate kept re-implementing: a corner metrics HUD, tween-driven
node animation, centered menu screens, floating "+N" popups, and reveal-on-touch
gating for on-screen controls. Each piece is opinion-light -- the module owns the
widget shape, the game owns the content -- and they compose with plain `bevy_ui`
nodes.

All snippets assume:

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
```

## status

`StatusBarPlugin` maintains a screen-corner metrics HUD (FPS, latency, version).
Spawn a `status_bar(StatusBarRootConfig::default())` root, then spawn
`status_bar_item(StatusBarItemConfig { .. })` children: an observer reparents each
item under the root. Every item has a `value_fn` (reads the `&World` -> an
optional `Arc<dyn StatusValue>`) and a `color_fn` (maps the value to a `Color`).
Ready-made helpers exist: `status_bar_with_fps()`, and the
`status_fps_value_fn` / `status_fps_color_fn` / `status_version_value_fn` /
`status_version_color_fn` builders.

```rust
fn setup(mut commands: Commands) {
    commands.spawn(status_bar(StatusBarRootConfig::default()));

    // The ready-made "NN fps" item (needs Bevy's FrameTimeDiagnosticsPlugin).
    commands.spawn(status_bar_with_fps());

    // A version tag driven by the builder fns.
    commands.spawn(status_bar_item(StatusBarItemConfig {
        icon: None,
        value_fn: status_version_value_fn(env!("CARGO_PKG_VERSION")),
        color_fn: status_version_color_fn(),
        prefix: "".to_string(),
        suffix: "".to_string(),
    }));
}
```

## animate

`UiAnimatePlugin` copies a [tween](../tween/)ed value into a `Node` field or
`BackgroundColor` each frame, the UI-node counterpart to the material-only
[feedback](../feedback/) flash. Spawn a `Tween<Vec2>` / `Tween<f32>` /
`Tween<Vec4>` and tag the entity with the matching marker: `TweenNodeOffset`
(writes `left`/`top` in px), `TweenNodeScale` (writes `width`/`height` as a
percent, `1.0` -> `100%`), or `TweenNodeBackground` (writes `BackgroundColor`,
linear RGBA). The `node_flash(to, duration)` helper builds a white -> `to`
`Tween<Vec4>`; `color_to_vec4` / `vec4_to_color` convert either way.

```rust
fn spawn_tile(mut commands: Commands) {
    // A tile that slides to a target pixel position.
    commands.spawn((
        Node::default(),
        TweenNodeOffset,
        Tween::new(Vec2::ZERO, Vec2::new(120.0, 40.0), 0.12, EaseFunction::QuadraticOut),
    ));
}
```

Add it alongside `TweenPlugin`; the apply step runs after `TweenSystems::Advance`.

## menu

`MenuPlugin` plus two builders cover the full-screen menu / game-over overlay
every game repeats. `centered_screen()` returns the absolutely-positioned,
both-axes-centered column root; `screen_text(text, size, color)` is one centered
`Text` row. Tag a title row with `TitlePulse::new(color)` to breathe its alpha in
a sine wave (`with_speed` / `with_alpha_range` tune it).

```rust
fn spawn_menu(mut commands: Commands) {
    commands.spawn(centered_screen()).with_children(|screen| {
        screen.spawn((
            screen_text("MY GAME", 72.0, Color::WHITE),
            TitlePulse::new(Color::srgb(0.95, 0.85, 0.25)),
        ));
        screen.spawn(screen_text("Tap to play", 32.0, Color::WHITE));
    });
}
```

## popup

`PopupPlugin` animates short-lived floating "+N" text that rises up the screen,
fades out, and despawns itself -- the score / pickup / damage number almost every
game shows. The `popup(position, text, font_size, color)` builder spawns the
common case at an absolute viewport position (project world -> screen yourself via
`camera::project::world_to_screen`). The `Popup` component is the config
(`lifetime`, `rise_speed`, `base_color`); inserting it attaches a private
`Tween<f32>` that owns the fade and despawn.

```rust
fn on_pickup(mut commands: Commands, viewport_pos: Vec2) {
    commands.spawn(popup(viewport_pos, "+10", 28.0, Color::WHITE));
}
```

## touchpad

`TouchpadPlugin` gates on-screen touch controls without any platform sniffing. It
keeps a `TouchSeen` resource (flipped true on the first touch, never reset) and
drives `Visibility` for entities tagged `RevealOnTouch` (shown once touched) or
`HideOnTouch` (a keyboard legend hidden once touched). Two pure hit-test helpers
round it out: `button_grid_at(point, window, cols, rows, zone)` maps a touch to a
row-major grid index, and `stick_deflection(offset, radius, dead)` maps a finger
offset to a dead-zoned unit-disc vector.

```rust
fn spawn_pad(mut commands: Commands) {
    // Hidden on desktop, revealed on the first finger touch.
    commands.spawn((Node::default(), RevealOnTouch));
}

fn on_touch(touches: Res<Touches>, windows: Query<&Window>) {
    let Ok(window) = windows.single() else { return };
    let zone = Rect::new(0.0, 0.84, 1.0, 1.0); // bottom 16% strip
    for touch in touches.iter_just_pressed() {
        if let Some(col) = button_grid_at(touch.position(), window.size(), 4, 1, zone) {
            let _ = col;
        }
    }
}
```

Note the module also ships `health_display` and `objectives` submodules; see
[health](../health/) for the health side.
