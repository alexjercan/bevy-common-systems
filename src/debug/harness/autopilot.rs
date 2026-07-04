//! Scripted state-driver harness: force-advance a game's state machine.
//!
//! [`AutopilotPlugin`] is an env-gated dev tool that drives a game through its
//! [`States`] machine on a fixed timeline for headless verification. It owns
//! the state clock, the transition logging and a clean [`AppExit`]; the game
//! supplies the timeline (a list of `(state, seconds)` steps) and an optional
//! per-frame input closure that pokes whatever it wants to drive gameplay.
//!
//! It is inert unless the `BCS_AUTOPILOT` environment variable is set, so a
//! game adds it unconditionally and pays nothing in a normal run:
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
//!     AutopilotPlugin::new()
//!         .hold(GameState::Menu, 0.5)
//!         .hold(GameState::Playing, 3.0)
//!         .hold(GameState::GameOver, 0.5)
//!         .input(|world, elapsed| {
//!             // Runs every frame with full world access and total elapsed
//!             // seconds; here, thrust continuously once in Playing.
//!             if elapsed > 0.5 {
//!                 world.resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Space);
//!             }
//!         }),
//! );
//! # }
//! ```

use std::sync::Arc;

use bevy::{input::InputSystems, prelude::*, state::state::FreelyMutableState};

use super::AUTOPILOT_ENV;

/// Per-frame input hook: full world access plus total elapsed seconds since the
/// autopilot started driving.
type InputFn = dyn Fn(&mut World, f32) + Send + Sync;

/// Env-gated plugin that force-drives a [`States`] machine along a scripted
/// timeline for headless verification.
///
/// Build the timeline with [`hold`](Self::hold) and, optionally, attach a
/// per-frame input closure with [`input`](Self::input). When the
/// `BCS_AUTOPILOT` env var is set the plugin sets the first state, holds each
/// step for its duration while advancing `NextState`, logs every transition,
/// and writes [`AppExit::Success`] after the last step. When the env var is
/// unset it adds nothing.
pub struct AutopilotPlugin<S: States + FreelyMutableState> {
    schedule: Vec<(S, f32)>,
    input: Option<Arc<InputFn>>,
}

impl<S: States + FreelyMutableState> Default for AutopilotPlugin<S> {
    fn default() -> Self {
        Self {
            schedule: Vec::new(),
            input: None,
        }
    }
}

impl<S: States + FreelyMutableState> AutopilotPlugin<S> {
    /// Create an empty autopilot. Add steps with [`hold`](Self::hold).
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a step: enter `state` and hold it for `seconds` before advancing
    /// to the next step. The first `hold` call names the starting state.
    pub fn hold(mut self, state: S, seconds: f32) -> Self {
        self.schedule.push((state, seconds));
        self
    }

    /// Set the per-frame input closure. It runs every frame in `PreUpdate`,
    /// after Bevy has collected input for the frame (`InputSystems`) but before
    /// the game's `Update` input systems read it, with `&mut World` and the
    /// total elapsed seconds. So it can poke input resources (`ButtonInput`,
    /// `Touches`, ...) or the game's own input components and the game will see
    /// the poke this frame -- including a fresh `just_pressed`, which the input
    /// collection would otherwise have cleared.
    ///
    /// The closure runs in every state, so if it presses keys a menu reacts to
    /// (an "any key to start" screen), gate it to the gameplay state to avoid
    /// tripping those transitions early:
    /// `if *world.resource::<State<GameState>>().get() != GameState::Playing { return; }`.
    pub fn input(mut self, f: impl Fn(&mut World, f32) + Send + Sync + 'static) -> Self {
        self.input = Some(Arc::new(f));
        self
    }
}

/// Internal driver state; kept out of the prelude per the crate conventions.
#[derive(Resource)]
struct AutopilotState<S: States + FreelyMutableState> {
    schedule: Vec<(S, f32)>,
    input: Option<Arc<InputFn>>,
    index: usize,
    elapsed: f32,
    state_elapsed: f32,
    started: bool,
    done: bool,
}

impl<S: States + FreelyMutableState> Plugin for AutopilotPlugin<S> {
    fn build(&self, app: &mut App) {
        if std::env::var(AUTOPILOT_ENV).is_err() {
            return;
        }
        if self.schedule.is_empty() {
            warn!("AutopilotPlugin: {AUTOPILOT_ENV} set but the schedule is empty; doing nothing");
            return;
        }

        debug!(
            "AutopilotPlugin: build ({AUTOPILOT_ENV} active, {} steps)",
            self.schedule.len()
        );

        app.insert_resource(AutopilotState::<S> {
            schedule: self.schedule.clone(),
            input: self.input.clone(),
            index: 0,
            elapsed: 0.0,
            state_elapsed: 0.0,
            started: false,
            done: false,
        });
        // Runs in PreUpdate after input collection so the input closure can set
        // a fresh `just_pressed` that survives into the game's Update systems
        // (Bevy clears `just_pressed` in `InputSystems` every frame).
        app.add_systems(PreUpdate, autopilot_drive::<S>.after(InputSystems));
    }
}

/// Exclusive driver: sets the initial state, advances the timeline, runs the
/// input closure, and exits cleanly after the last step.
///
/// Exclusive because the input closure takes `&mut World`; the driver state is
/// removed for the duration so the closure has unencumbered world access.
fn autopilot_drive<S: States + FreelyMutableState>(world: &mut World) {
    let mut st = world
        .remove_resource::<AutopilotState<S>>()
        .expect("AutopilotState is inserted by AutopilotPlugin::build");

    // The timeline is finished; AppExit is already queued. Stay inert (do not
    // index past the end of the schedule) in case another frame still runs.
    if st.done {
        world.insert_resource(st);
        return;
    }

    // First frame: set the starting state (unless already there, to avoid a
    // spurious OnExit/OnEnter of the default state), then wait a frame for the
    // transition to apply before the clock starts.
    if !st.started {
        let first = st.schedule[0].0.clone();
        if *world.resource::<State<S>>().get() != first {
            world.resource_mut::<NextState<S>>().set(first.clone());
        }
        trace!("autopilot: start in {first:?}");
        st.started = true;
        world.insert_resource(st);
        return;
    }

    let dt = world.resource::<Time>().delta_secs();
    st.elapsed += dt;
    st.state_elapsed += dt;

    if let Some(input) = st.input.clone() {
        input(world, st.elapsed);
    }

    let hold = st.schedule[st.index].1;
    if st.state_elapsed >= hold {
        st.index += 1;
        if st.index >= st.schedule.len() {
            info!("autopilot: cycle complete, no panic (t={:.1}s)", st.elapsed);
            world.write_message(AppExit::Success);
            st.done = true;
            world.insert_resource(st);
            return;
        }
        let next = st.schedule[st.index].0.clone();
        info!("autopilot: -> {next:?} (t={:.1}s)", st.elapsed);
        world.resource_mut::<NextState<S>>().set(next);
        st.state_elapsed = 0.0;
    }

    world.insert_resource(st);
}
