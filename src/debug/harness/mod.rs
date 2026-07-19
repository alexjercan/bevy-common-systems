//! Headless verification harness plugins (dev tooling, `debug` feature).
//!
//! Two env-gated plugins over a shared state-driver idea, for exercising a game
//! headlessly the way AGENTS.md prescribes -- so the crate's own "an example is
//! not done until it has been run once" rule stops costing a
//! hand-rolled-and-deleted harness per game. Both are dev tools behind the same
//! feature flag as [`InspectorDebugPlugin`](super::inspector::InspectorDebugPlugin),
//! so the "no framework machinery" charter line does not bind them.
//!
//! - [`AutopilotPlugin`](autopilot::AutopilotPlugin) force-drives the game's
//!   [`States`](bevy::prelude::States) machine along a scripted timeline
//!   (Menu -> Playing -> ... -> GameOver), runs an optional per-frame input
//!   closure, logs every transition and a final "cycle complete, no panic"
//!   line, then reports done to the [`completion`] protocol - the app exits
//!   [`AppExit::Success`](bevy::prelude::AppExit) when EVERY registered
//!   collector (autopilot, screenshot, an external frame capture) is done,
//!   never via `std::process::exit` (AGENTS.md: segfaults on wgpu teardown)
//!   and never unilaterally (success negotiates; only failures abort).
//! - [`ScreenshotPlugin`](screenshot::ScreenshotPlugin) overrides the window
//!   resolution, advances to a named state, waits N settled frames, writes a
//!   PNG, and exits.
//!
//! Both are inert unless their env var is set (`BCS_AUTOPILOT` / `BCS_SHOT`), so
//! a game can add them unconditionally and pay nothing in a normal run -- no
//! add/remove churn per verification, unlike the temporary hand-rolled harness
//! this replaces.
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_common_systems::debug::prelude::*;
//!
//! #[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
//! enum GameState {
//!     #[default]
//!     Menu,
//!     Playing,
//!     GameOver,
//! }
//!
//! # fn build(app: &mut App) {
//! // Inert in a normal run; drives a full cycle when BCS_AUTOPILOT is set.
//! app.add_plugins(
//!     AutopilotPlugin::new()
//!         .hold(GameState::Menu, 0.5)
//!         .hold(GameState::Playing, 3.0)
//!         .hold(GameState::GameOver, 0.5)
//!         .input(|world, _elapsed| {
//!             // Poke whatever input the game reads; here, hold Space.
//!             world.resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Space);
//!         }),
//! );
//! # }
//! ```

pub mod autopilot;
pub mod screenshot;

/// The completion protocol lives ungated at the crate root (feature-less
/// consumers register external collectors); re-exported here because the
/// harness plugins are its primary registrants.
pub use crate::completion;

/// Environment variable that activates
/// [`AutopilotPlugin`](autopilot::AutopilotPlugin). Any value (even empty)
/// enables it; when unset the plugin adds nothing.
pub const AUTOPILOT_ENV: &str = "BCS_AUTOPILOT";

/// Environment variable that activates
/// [`ScreenshotPlugin`](screenshot::ScreenshotPlugin). Any value enables it;
/// a value matching `WxH` (for example `800x600`) also overrides the window
/// resolution before the capture.
pub const SCREENSHOT_ENV: &str = "BCS_SHOT";

/// Re-exports the harness plugins.
///
/// ```rust
/// use bevy_common_systems::debug::harness::prelude::*;
/// ```
pub mod prelude {
    pub use super::{
        autopilot::AutopilotPlugin, completion::HarnessCompletion, screenshot::ScreenshotPlugin,
    };
}
