//! A unified pointer that collapses mouse, touch and cursor into one resource.
//!
//! Almost every pointer-driven game needs the same thing: "where is the player
//! pointing, is it down, and did it just go down this frame", resolved the same
//! way whether the input came from a mouse or a finger. Games keep re-deriving
//! it (three of this crate's examples had their own copy).
//! [`UnifiedPointerPlugin`] owns it: it maintains a [`UnifiedPointer`] resource
//! each frame in `PreUpdate`, reading Bevy's raw [`Touches`] and
//! [`ButtonInput<MouseButton>`] plus the primary window cursor. An active touch
//! wins over the mouse cursor, so a finger drives aiming on a touch build while
//! the mouse drives it on desktop, with no per-platform branching in the game.
//!
//! The plugin deliberately reads raw [`Touches`] rather than routing through
//! `bevy_enhanced_input`, so adding it pulls in no extra input dependency. A
//! game that already drives its press/hold through an enhanced-input action can
//! still reuse the position logic on its own via [`active_pointer_pos`].
//!
//! The resource is named `UnifiedPointer` rather than `Pointer` because Bevy's
//! own prelude already exports a `Pointer` (the `bevy_picking` pointer event);
//! the distinct name lets this one live in the crate prelude without colliding.
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(UnifiedPointerPlugin);
//!
//! fn aim(pointer: Res<UnifiedPointer>) {
//!     if pointer.pressed {
//!         if let Some(screen_pos) = pointer.screen_pos {
//!             // ... aim toward `screen_pos` ...
//!             let _ = screen_pos;
//!         }
//!     }
//! }
//! ```

use bevy::{ecs::system::SystemParam, prelude::*, window::PrimaryWindow};

pub mod prelude {
    pub use super::{
        active_pointer_pos, any_start_pressed, AnyStartPress, UnifiedPointer, UnifiedPointerPlugin,
        UnifiedPointerSystems,
    };
}

/// System sets for the unified pointer plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum UnifiedPointerSystems {
    /// Resolves the [`UnifiedPointer`] resource from raw mouse and touch input.
    /// Runs in `PreUpdate`, so any `Update` system reads a pointer that reflects
    /// this frame's input.
    Resolve,
}

/// The current pointer, unified across mouse and touch.
///
/// An active touch takes priority over the mouse cursor, so a finger drives the
/// pointer on a touch device and the cursor drives it otherwise. Maintained by
/// [`UnifiedPointerPlugin`]; read it from any system as `Res<UnifiedPointer>`.
///
/// Named `UnifiedPointer` to avoid clashing with Bevy's prelude `Pointer` (the
/// `bevy_picking` pointer event).
#[derive(Resource, Default, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct UnifiedPointer {
    /// On-screen position (logical window pixels) of the active pointer this
    /// frame, if any. `None` when there is no touch and the cursor is outside
    /// the window.
    pub screen_pos: Option<Vec2>,
    /// Whether the pointer is currently down: the left mouse button is held or a
    /// finger is on the screen.
    pub pressed: bool,
    /// True only on the frame the press began (a click or a tap), for
    /// "advance on tap" style checks.
    pub just_pressed: bool,
}

/// The active pointer position: the touch position if there is one, otherwise
/// the cursor position.
///
/// Pure helper factored out so the touch-wins-over-cursor rule is one testable
/// place, and so a game resolving its press through another input path (for
/// example a `bevy_enhanced_input` action) can still reuse the position logic.
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_common_systems::prelude::*;
/// let touch = Vec2::new(10.0, 20.0);
/// let cursor = Vec2::new(30.0, 40.0);
/// // A live touch wins over the cursor.
/// assert_eq!(active_pointer_pos(Some(touch), Some(cursor)), Some(touch));
/// // With no touch, the cursor is used.
/// assert_eq!(active_pointer_pos(None, Some(cursor)), Some(cursor));
/// ```
pub fn active_pointer_pos(touch_pos: Option<Vec2>, cursor_pos: Option<Vec2>) -> Option<Vec2> {
    touch_pos.or(cursor_pos)
}

/// "Did the player press anything to advance this frame" -- a left click, a
/// tap, or `Space` / `Enter`.
///
/// A [`SystemParam`] bundling the raw mouse, keyboard and touch state so the
/// "advance on any press" check every menu / game-over screen does is one call
/// instead of the copy-pasted three-way `||`. It reads raw Bevy input (not
/// [`UnifiedPointer`]) so it needs no plugin and works in any game.
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_common_systems::prelude::*;
/// # #[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
/// # enum GameState { #[default] Menu, Playing }
/// fn dismiss_menu(start: AnyStartPress, mut next: ResMut<NextState<GameState>>) {
///     if start.just_pressed() {
///         next.set(GameState::Playing);
///     }
/// }
/// ```
#[derive(SystemParam)]
pub struct AnyStartPress<'w> {
    mouse: Res<'w, ButtonInput<MouseButton>>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    touches: Res<'w, Touches>,
}

impl AnyStartPress<'_> {
    /// True on the frame the player begins a press meant to advance: a left
    /// mouse click, a `Space` / `Enter` key, or a fresh touch.
    pub fn just_pressed(&self) -> bool {
        self.mouse.just_pressed(MouseButton::Left)
            || self.keys.just_pressed(KeyCode::Space)
            || self.keys.just_pressed(KeyCode::Enter)
            || self.touches.any_just_pressed()
    }
}

/// Run-condition form of [`AnyStartPress::just_pressed`], for gating a system on
/// "any advance press this frame":
/// `system.run_if(any_start_pressed)`.
pub fn any_start_pressed(start: AnyStartPress) -> bool {
    start.just_pressed()
}

/// Maintains the unified [`UnifiedPointer`] resource each frame.
pub struct UnifiedPointerPlugin;

impl Plugin for UnifiedPointerPlugin {
    fn build(&self, app: &mut App) {
        debug!("UnifiedPointerPlugin: build");

        app.init_resource::<UnifiedPointer>()
            .register_type::<UnifiedPointer>()
            .add_systems(
                PreUpdate,
                resolve_pointer.in_set(UnifiedPointerSystems::Resolve),
            );
    }
}

/// Resolve the pointer from raw mouse and touch input each frame.
///
/// Reading the raw state directly (rather than through `bevy_enhanced_input`)
/// keeps the plugin dependency-free. On desktop `Touches` is always empty, so
/// the pointer is simply the mouse; on a touch build a live finger takes over.
fn resolve_pointer(
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut pointer: ResMut<UnifiedPointer>,
) {
    trace!("resolve_pointer");

    let touch_pos = touches.iter().next().map(|touch| touch.position());
    let touch_just = touches.iter_just_pressed().next().is_some();

    pointer.pressed = mouse.pressed(MouseButton::Left) || touch_pos.is_some();
    pointer.just_pressed = mouse.just_pressed(MouseButton::Left) || touch_just;

    let cursor_pos = window
        .iter()
        .next()
        .and_then(|window| window.cursor_position());
    pointer.screen_pos = active_pointer_pos(touch_pos, cursor_pos);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_wins_over_cursor() {
        let touch = Vec2::new(10.0, 20.0);
        let cursor = Vec2::new(30.0, 40.0);
        assert_eq!(active_pointer_pos(Some(touch), Some(cursor)), Some(touch));
    }

    #[test]
    fn cursor_used_when_no_touch() {
        let cursor = Vec2::new(30.0, 40.0);
        assert_eq!(active_pointer_pos(None, Some(cursor)), Some(cursor));
    }

    #[test]
    fn none_when_neither_present() {
        assert_eq!(active_pointer_pos(None, None), None);
    }
}
