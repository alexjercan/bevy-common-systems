# input

The `input` module collects small, game-agnostic input building blocks: a
unified pointer that collapses mouse, touch, and cursor into one resource;
cursor grab / release for mouse-look; and a couple of input-to-state helpers.
The point is to stop every game from re-deriving "where is the player pointing,
is it down, did it just go down" or hand-rolling the same "press Escape to give
up" system.

All snippets assume:

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
```

## The pointer resource

`UnifiedPointerPlugin` maintains a `UnifiedPointer` resource each frame in
`PreUpdate`, reading raw `Touches` + `ButtonInput<MouseButton>` plus the primary
window cursor. An active touch wins over the mouse cursor, so a finger drives
aiming on a touch build and the mouse drives it on desktop with no per-platform
branching. It is named `UnifiedPointer` (not `Pointer`) to avoid clashing with
Bevy's `bevy_picking` prelude type. The resource exposes `screen_pos:
Option<Vec2>`, `pressed`, and `just_pressed`.

```rust
fn wire_up() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UnifiedPointerPlugin);
}

fn aim(pointer: Res<UnifiedPointer>) {
    if pointer.pressed {
        if let Some(screen_pos) = pointer.screen_pos {
            let _ = screen_pos; // ... aim toward it ...
        }
    }
}
```

The pure `active_pointer_pos(touch_pos, cursor_pos)` helper encodes the
touch-wins rule and is reusable if you resolve press through another path. For a
`bevy_enhanced_input`-driven variant of the same resource, see the
[helpers](../helpers/) `EnhancedInputPointerPlugin`.

## Cursor grab

For mouse-look, `grab_cursor` and `release_cursor` toggle the per-window
`CursorOptions` (in current Bevy the grab state is a component, not a `Window`
field, which is easy to miss). `grab_cursor` locks and hides the cursor;
`release_cursor` frees and shows it. The policy of when to grab stays with the
game -- call these from your own state-transition systems.

```rust
use bevy::window::{CursorOptions, PrimaryWindow};

fn grab(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    grab_cursor(&mut cursor);
}

fn release(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    release_cursor(&mut cursor);
}
```

## Input state helpers

Two helpers bridge input to state. `AnyStartPress` is a `SystemParam` bundling
mouse, keyboard, and touch so the "advance on any press" check every menu does
(left click, `Space`, `Enter`, or a fresh touch) is one call; it reads raw Bevy
input, so it needs no plugin. `any_start_pressed` is its run-condition form.
`set_state_on_key(key, target)` is a factory returning a ready-to-add system that
sets a next state when a key is just pressed.

```rust
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState { #[default] Menu, Playing, GameOver }

fn dismiss_menu(start: AnyStartPress, mut next: ResMut<NextState<GameState>>) {
    if start.just_pressed() {
        next.set(GameState::Playing);
    }
}

fn wire(app: &mut App) {
    // Escape gives up the current run.
    app.add_systems(
        Update,
        set_state_on_key(KeyCode::Escape, GameState::GameOver)
            .run_if(in_state(GameState::Playing)),
    );
}
```
