//! A `bevy_enhanced_input` bridge that drives the crate's [`UnifiedPointer`].
//!
//! The core [`input/pointer`](crate::input::pointer) module resolves a
//! [`UnifiedPointer`] from raw [`Touches`] + mouse, with no input-framework
//! dependency. This helper is the alternative for games that already route input
//! through `bevy_enhanced_input`: it drives the *same* [`UnifiedPointer`]
//! resource from an enhanced-input press action, so a game gets one shared
//! pointer abstraction whichever input path it uses. It is the pointer analogue
//! of [`helpers/wasd`](crate::helpers::wasd), which binds enhanced-input for the
//! WASD camera.
//!
//! [`EnhancedInputPointerPlugin`] owns the whole resource -- both the press
//! (`pressed`/`just_pressed`, from a `PointerPress` action bound to the left
//! mouse button and a touch `Binding::Custom`) and the position (`screen_pos`,
//! an active touch winning over the cursor via
//! [`active_pointer_pos`](crate::input::pointer::active_pointer_pos)). Use it
//! *instead of*, not alongside, [`UnifiedPointerPlugin`]: both write
//! [`UnifiedPointer`] every frame, so adding both would have them fight.
//!
//! Unlike the core module, this pulls in `bevy_enhanced_input` -- which is why it
//! lives in `helpers/` and the core `input/pointer` stays dependency-free.
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! # fn wire_up() {
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     // Drives `UnifiedPointer` from an enhanced-input press action instead of
//!     // the raw `UnifiedPointerPlugin`.
//!     .add_plugins(EnhancedInputPointerPlugin);
//! # }
//!
//! fn aim(pointer: Res<UnifiedPointer>) {
//!     if pointer.just_pressed {
//!         // ... a click or tap began this frame ...
//!     }
//! }
//! ```

use bevy::{input::InputSystems, prelude::*, window::PrimaryWindow};
use bevy_enhanced_input::prelude::*;

use crate::input::pointer::{active_pointer_pos, UnifiedPointer};

pub mod prelude {
    pub use super::EnhancedInputPointerPlugin;
}

/// Input-context marker for the pointer's enhanced-input action.
#[derive(Component, Debug, Clone)]
struct PointerInputContext;

/// The `Binding::Custom` id that carries touch-pressed state into
/// enhanced-input. Registered once at startup.
#[derive(Resource)]
struct TouchPointerId(CustomInput);

/// The press/hold action, bound to the left mouse button and the touch custom
/// input so either device actuates it.
#[derive(InputAction)]
#[action_output(bool)]
struct PointerPress;

/// Drives the crate's [`UnifiedPointer`] from a `bevy_enhanced_input` press
/// action plus a touch/cursor position resolve.
///
/// Add this OR [`UnifiedPointerPlugin`](crate::input::pointer::UnifiedPointerPlugin),
/// never both -- they both own the [`UnifiedPointer`] resource.
pub struct EnhancedInputPointerPlugin;

impl Plugin for EnhancedInputPointerPlugin {
    fn build(&self, app: &mut App) {
        debug!("EnhancedInputPointerPlugin: build");

        // The enhanced-input runtime may already be present (another controller,
        // e.g. helpers/wasd, could have added it); only add it once.
        if !app.is_plugin_added::<EnhancedInputPlugin>() {
            app.add_plugins(EnhancedInputPlugin);
        }
        app.add_input_context::<PointerInputContext>();
        app.init_resource::<UnifiedPointer>();
        app.register_type::<UnifiedPointer>();

        app.add_observer(on_pointer_press_start);
        app.add_observer(on_pointer_press_complete);

        app.add_systems(Startup, setup_pointer_action);
        // Stage the touch value and the pointer position after Bevy reads raw
        // input and before enhanced-input evaluates the action this frame.
        app.add_systems(
            PreUpdate,
            stage_pointer_input
                .after(InputSystems)
                .before(EnhancedInputSystems::Prepare),
        );
        // `just_pressed` is set by the press observer and holds for exactly one
        // frame; clearing it at frame end keeps it edge-triggered.
        app.add_systems(Last, clear_pointer_just_pressed);
    }
}

/// Register the touch custom input and spawn the pointer action. Done in one
/// place so the id is in scope for both the `Binding::Custom` and the
/// [`TouchPointerId`] resource `stage_pointer_input` reads.
fn setup_pointer_action(mut commands: Commands, mut custom_inputs: ResMut<CustomInputs>) {
    let touch_id = custom_inputs.register_input();
    commands.insert_resource(TouchPointerId(touch_id));
    commands.spawn((
        Name::new("Pointer Input"),
        PointerInputContext,
        actions!(
            PointerInputContext[
                (
                    Name::new("Input: Pointer Press"),
                    Action::<PointerPress>::new(),
                    bindings![MouseButton::Left, Binding::Custom(touch_id)],
                ),
            ]
        ),
    ));
}

/// Feed touch state into enhanced-input and resolve the pointer position.
///
/// On desktop `Touches` is always empty, so the custom input stays `false` and
/// the position falls back to the mouse cursor -- identical to a mouse-only game.
fn stage_pointer_input(
    touch_id: Res<TouchPointerId>,
    touches: Res<Touches>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut custom_inputs: ResMut<CustomInputs>,
    mut pointer: ResMut<UnifiedPointer>,
) {
    trace!("stage_pointer_input");

    let touch_pos = touches.iter().next().map(|touch| touch.position());
    custom_inputs.insert(touch_id.0, ActionValue::Bool(touch_pos.is_some()));

    let cursor_pos = window
        .iter()
        .next()
        .and_then(|window| window.cursor_position());
    pointer.screen_pos = active_pointer_pos(touch_pos, cursor_pos);
}

/// Mark the pointer pressed on the press edge (a click or tap begins).
fn on_pointer_press_start(_: On<Start<PointerPress>>, mut pointer: ResMut<UnifiedPointer>) {
    pointer.pressed = true;
    pointer.just_pressed = true;
}

/// Clear the pressed state when the button / finger is released.
fn on_pointer_press_complete(_: On<Complete<PointerPress>>, mut pointer: ResMut<UnifiedPointer>) {
    pointer.pressed = false;
}

/// Reset the one-frame `just_pressed` edge at the end of every frame.
fn clear_pointer_just_pressed(mut pointer: ResMut<UnifiedPointer>) {
    pointer.just_pressed = false;
}
