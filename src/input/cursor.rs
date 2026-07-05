//! Lock / release the mouse cursor for mouse-look.
//!
//! In Bevy 0.19 the cursor grab state lives on a per-window [`CursorOptions`]
//! component (not a field of `Window`), which is easy to miss. These helpers wrap
//! the two states a first-person game toggles: captured for looking, and freed for
//! menus. The *policy* -- when to grab, and whether to skip it under a headless
//! test harness -- stays with the game; call these from your own systems.
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy::window::{CursorOptions, PrimaryWindow};
//! # use bevy_common_systems::prelude::*;
//! // On entering play, capture the cursor:
//! fn grab(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
//!     grab_cursor(&mut cursor);
//! }
//! // On leaving play, release it:
//! fn release(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
//!     release_cursor(&mut cursor);
//! }
//! ```

use bevy::window::{CursorGrabMode, CursorOptions};

pub mod prelude {
    pub use super::{grab_cursor, release_cursor};
}

/// Capture the cursor for mouse-look: lock it in place and hide it. Uses
/// [`CursorGrabMode::Locked`]; on X11 (which has no true locked mode) the winit
/// backend falls back to confining the cursor.
pub fn grab_cursor(cursor: &mut CursorOptions) {
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;
}

/// Release the cursor: stop grabbing it and make it visible again (for menus).
pub fn release_cursor(cursor: &mut CursorOptions) {
    cursor.grab_mode = CursorGrabMode::None;
    cursor.visible = true;
}
