//! Viewport-capture harness: force a window size, reach a state, snap a PNG.
//!
//! [`ScreenshotPlugin`] is an env-gated sibling of
//! [`AutopilotPlugin`](super::autopilot::AutopilotPlugin) that shares the same
//! state-driver idea: it advances the game to a named state, waits for it to
//! settle, then captures the primary window to a file and exits. It exists to
//! make the "Seeing the screen" workflow AGENTS.md documents reusable -- it
//! caught a real responsive-layout regression (shop buttons below the fold at
//! phone width) that every non-visual check missed.
//!
//! It is inert unless the `BCS_SHOT` environment variable is set. A value of
//! the form `WxH` (for example `390x844`) overrides the window resolution
//! before the capture, which is exactly the phone-width check that found the
//! regression:
//!
//! ```rust,no_run
//! use bevy::prelude::*;
//! use bevy_common_systems::debug::harness::prelude::*;
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
//! app.add_plugins(
//!     ScreenshotPlugin::new(GameState::Playing)
//!         .settle_frames(12)
//!         .path("shot.png"),
//! );
//! # }
//! ```
//!
//! Then `BCS_SHOT=390x844 cargo run --example my_game --features debug` writes
//! `shot.png` at phone width and exits.

use bevy::{
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured},
    state::state::FreelyMutableState,
    window::PrimaryWindow,
};

use super::{super::inspector::DebugEnabled, completion, AUTOPILOT_ENV, SCREENSHOT_ENV};

/// Safety bound: if the target state is not reached within this many frames the
/// harness exits with an error instead of hanging forever (for example if the
/// game keeps overriding `NextState` back to a menu). At 60 fps this is ~30s.
const MAX_WAIT_FRAMES: u32 = 1800;

/// Env-gated plugin that advances to a state, settles, and captures a PNG.
///
/// Build it with the target [`State`] and, optionally, a settle-frame count,
/// output path, and default resolution. When `BCS_SHOT` is set the plugin
/// (optionally) resizes the primary window, sets `NextState` to the target,
/// waits [`settle_frames`](Self::settle_frames) frames after the state is
/// entered, spawns a [`Screenshot`] with [`save_to_disk`], and writes
/// [`AppExit::Success`] once the capture lands on disk. When the env var is
/// unset it adds nothing.
///
/// Distinct from Bevy's internal `bevy::render::view::screenshot::ScreenshotPlugin`
/// (the render-side plugin that services capture requests); this is the
/// headless-verification harness that drives one. Neither is in `bevy::prelude`,
/// so there is no name collision, but do not confuse the two.
pub struct ScreenshotPlugin<S: States + FreelyMutableState> {
    state: S,
    settle_frames: u32,
    path: String,
    resolution: Option<(f32, f32)>,
}

impl<S: States + FreelyMutableState> ScreenshotPlugin<S> {
    /// Create a screenshot harness targeting `state`, with sensible defaults
    /// (8 settle frames, `screenshot.png`, no resolution override).
    pub fn new(state: S) -> Self {
        Self {
            state,
            settle_frames: 8,
            path: "screenshot.png".to_string(),
            resolution: None,
        }
    }

    /// Number of frames to wait after the target state is entered before
    /// capturing, so animations and one-shot layout settle. Defaults to 8.
    pub fn settle_frames(mut self, frames: u32) -> Self {
        self.settle_frames = frames;
        self
    }

    /// Output path for the PNG. Defaults to `screenshot.png`.
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    /// Default window resolution to force before capture. The `WxH` value of
    /// the `BCS_SHOT` env var, if present, takes precedence over this.
    pub fn resolution(mut self, width: f32, height: f32) -> Self {
        self.resolution = Some((width, height));
        self
    }
}

/// Internal driver state; kept out of the prelude per the crate conventions.
#[derive(Resource)]
struct ScreenshotConfig<S: States + FreelyMutableState> {
    state: S,
    settle_frames: u32,
    path: String,
    resolution: Option<(f32, f32)>,
    advanced: bool,
    frames_in_state: u32,
    waited_frames: u32,
    shot_taken: bool,
}

impl<S: States + FreelyMutableState> Plugin for ScreenshotPlugin<S> {
    fn build(&self, app: &mut App) {
        let Ok(env_value) = std::env::var(SCREENSHOT_ENV) else {
            return;
        };

        // Both harnesses drive `NextState`; running them together would fight
        // over it. Autopilot wins -- the screenshot harness stands down.
        if std::env::var(AUTOPILOT_ENV).is_ok() {
            warn!(
                "ScreenshotPlugin: both {SCREENSHOT_ENV} and {AUTOPILOT_ENV} are set; \
                 deferring to the autopilot and doing nothing"
            );
            return;
        }

        // A `WxH` env value overrides the builder's default resolution.
        let resolution = parse_resolution(&env_value).or(self.resolution);

        debug!(
            "ScreenshotPlugin: build ({SCREENSHOT_ENV} active, target {:?}, resolution {:?})",
            self.state, resolution
        );

        app.insert_resource(ScreenshotConfig::<S> {
            state: self.state.clone(),
            settle_frames: self.settle_frames,
            path: self.path.clone(),
            resolution,
            advanced: false,
            frames_in_state: 0,
            waited_frames: 0,
            shot_taken: false,
        });

        if resolution.is_some() {
            app.add_systems(Startup, resize_window::<S>);
        }
        // The screenshot is for verifying the game's own layout, so hide the
        // inspector/diagnostics overlay if InspectorDebugPlugin is present.
        app.add_systems(Startup, hide_debug_overlay);
        app.add_systems(Update, screenshot_drive::<S>);
        completion::register(app, completion::SCREENSHOT);
    }
}

/// Turn off the debug overlay (inspector window, physics gizmos, diagnostics UI)
/// so it does not obscure the captured frame. No-op when
/// [`InspectorDebugPlugin`](super::super::inspector::InspectorDebugPlugin), and
/// therefore its `DebugEnabled` resource, is not present.
fn hide_debug_overlay(debug: Option<ResMut<DebugEnabled>>) {
    if let Some(mut debug) = debug {
        **debug = false;
    }
}

/// Force the primary window to the configured resolution at startup.
fn resize_window<S: States + FreelyMutableState>(
    config: Res<ScreenshotConfig<S>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Some((width, height)) = config.resolution else {
        return;
    };
    if let Ok(mut window) = windows.single_mut() {
        window.resolution.set(width, height);
        // Pin the size so a tiling/reflowing WM cannot resize the window back
        // and undermine a responsive-layout capture.
        window.resizable = false;
        trace!("screenshot: forced window resolution to {width}x{height}");
    }
}

/// Advance to the target state, count settled frames, then capture and exit.
fn screenshot_drive<S: States + FreelyMutableState>(
    mut commands: Commands,
    mut config: ResMut<ScreenshotConfig<S>>,
    state: Res<State<S>>,
    mut next: ResMut<NextState<S>>,
    mut exit: MessageWriter<AppExit>,
) {
    if config.shot_taken {
        return;
    }

    // First run: request the target state (unless already there).
    if !config.advanced {
        if state.get() != &config.state {
            next.set(config.state.clone());
        }
        config.advanced = true;
        trace!("screenshot: advancing to {:?}", config.state);
        return;
    }

    // Wait until the transition has actually applied. Bound the wait so an
    // unreachable target state exits with an error instead of hanging.
    if state.get() != &config.state {
        config.waited_frames += 1;
        if config.waited_frames >= MAX_WAIT_FRAMES {
            error!(
                "screenshot: target state {:?} not reached within {MAX_WAIT_FRAMES} frames; \
                 giving up",
                config.state
            );
            config.shot_taken = true;
            exit.write(AppExit::error());
        }
        return;
    }

    config.frames_in_state += 1;
    if config.frames_in_state < config.settle_frames {
        return;
    }

    let path = config.path.clone();
    info!(
        "screenshot: capturing {:?} after {} settled frames -> {path}",
        config.state, config.frames_in_state
    );

    // save_to_disk writes the PNG synchronously in its observer; a second
    // observer on the same capture reports completion once the frame is on
    // disk (the app exits when every registered collector is done).
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path))
        .observe(
            |_: On<ScreenshotCaptured>, mut completion: ResMut<completion::HarnessCompletion>| {
                info!("screenshot: capture complete");
                completion.done(completion::SCREENSHOT);
            },
        );

    config.shot_taken = true;
}

/// Parse a `WxH` resolution string (case-insensitive `x`), for example
/// `800x600`. Returns `None` when the string is not a valid `WxH` pair of
/// positive dimensions, so a plain toggle value like `1` (or a nonsense
/// `0x0` / `-5x10`) leaves the resolution unchanged.
fn parse_resolution(value: &str) -> Option<(f32, f32)> {
    let (width, height) = value.split_once(['x', 'X'])?;
    let width: f32 = width.trim().parse().ok()?;
    let height: f32 = height.trim().parse().ok()?;
    (width > 0.0 && height > 0.0).then_some((width, height))
}

#[cfg(test)]
mod tests {
    use super::parse_resolution;

    #[test]
    fn parses_valid_wxh() {
        assert_eq!(parse_resolution("800x600"), Some((800.0, 600.0)));
        assert_eq!(parse_resolution("390x844"), Some((390.0, 844.0)));
        // Capital X is accepted too.
        assert_eq!(parse_resolution("1024X768"), Some((1024.0, 768.0)));
        // Surrounding whitespace is trimmed.
        assert_eq!(parse_resolution(" 640 x 480 "), Some((640.0, 480.0)));
    }

    #[test]
    fn rejects_non_resolution_values() {
        // A plain toggle value must not parse as a resolution.
        assert_eq!(parse_resolution("1"), None);
        assert_eq!(parse_resolution(""), None);
        assert_eq!(parse_resolution("wide"), None);
        assert_eq!(parse_resolution("800x"), None);
        assert_eq!(parse_resolution("x600"), None);
        assert_eq!(parse_resolution("800x600x3"), None);
    }

    #[test]
    fn rejects_non_positive_dimensions() {
        // Zero or negative dimensions would be nonsense fed to `resolution.set`.
        assert_eq!(parse_resolution("0x0"), None);
        assert_eq!(parse_resolution("800x0"), None);
        assert_eq!(parse_resolution("-5x10"), None);
        assert_eq!(parse_resolution("10x-5"), None);
    }
}
