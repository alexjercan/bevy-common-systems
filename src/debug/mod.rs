//! Debug utilities for your Bevy game.
//!
//! This module groups optional debugging tools such as:
//! - **WireframeDebugPlugin** - toggles global wireframe rendering.
//! - **InspectorDebugPlugin** - enables the Bevy inspector (if enabled).
//! - **harness** - env-gated autopilot / screenshot plugins for headless
//!   verification (`AutopilotPlugin`, `ScreenshotPlugin`).
//!
//! ## Usage
//! Add whichever plugins you want, or pull them all via the `prelude`:
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_common_systems::debug::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(WireframeDebugPlugin)
//!         .add_plugins(InspectorDebugPlugin)
//!         .run();
//! }
//! ```
//!
//! The `prelude` module re-exports the most commonly used debug plugins.

pub mod harness;
pub mod inspector;
pub mod wireframe;

/// Re-exports commonly used debug plugins for convenience.
///
/// ```rust
/// use bevy_common_systems::debug::prelude::*;
/// ```
pub mod prelude {
    pub use super::{
        harness::prelude::*, inspector::InspectorDebugPlugin, wireframe::WireframeDebugPlugin,
    };
}
