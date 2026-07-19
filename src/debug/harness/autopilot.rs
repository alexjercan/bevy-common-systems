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

use super::{completion, AUTOPILOT_ENV};

/// Per-frame input hook: full world access plus total elapsed seconds since the
/// autopilot started driving.
type InputFn = dyn Fn(&mut World, f32) + Send + Sync;

/// Message written each time a [`loop_while_pending`]
/// (`AutopilotPlugin::loop_while_pending`) autopilot restarts its cycle
/// because other completion collectors are still pending. The game observes
/// it to reset its scene/script state (re-trigger a scenario load, zero its
/// script resource) so the repeated cycle measures ACTIVITY, not an idle
/// tail.
#[derive(Message)]
pub struct AutopilotLoop;

/// Env-gated plugin that force-drives a [`States`] machine along a scripted
/// timeline for headless verification.
///
/// Build the timeline with [`hold`](Self::hold) and, optionally, attach a
/// per-frame input closure with [`input`](Self::input). When the
/// `BCS_AUTOPILOT` env var is set the plugin sets the first state, holds each
/// step for its duration while advancing `NextState`, logs every transition,
/// and reports completion to the harness [`completion`] protocol after the
/// last step (the app exits when EVERY registered collector - a frame
/// capture, a screenshot - is done, not when the first one finishes). When
/// the env var is unset it adds nothing.
pub struct AutopilotPlugin<S: States + FreelyMutableState> {
    schedule: Vec<(S, f32)>,
    input: Option<Arc<InputFn>>,
    self_completing: bool,
    loop_while_pending: bool,
}

impl<S: States + FreelyMutableState> Default for AutopilotPlugin<S> {
    fn default() -> Self {
        Self {
            schedule: Vec::new(),
            input: None,
            self_completing: false,
            loop_while_pending: false,
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

    /// Mark completion as SCRIPT-OWNED: the timeline is a runway, not the
    /// finish line. The input closure's staged script reports done itself
    /// (`world.resource_mut::<HarnessCompletion>().done(completion::AUTOPILOT)`)
    /// when its final stage lands; if the TIMELINE expires first the script
    /// stalled, and the run exits [`AppExit::error`] naming it - an abort,
    /// not a completion, so a stalled script can never pass as a finished
    /// cycle.
    pub fn self_completing(mut self) -> Self {
        self.self_completing = true;
        self
    }

    /// Repeat the cycle while OTHER completion collectors are still
    /// pending: at the timeline's end, instead of reporting done, write an
    /// [`AutopilotLoop`] message (the game resets its scene/script on it),
    /// zero the cycle clock, and keep driving - a frame capture then
    /// measures repeated ACTIVITY instead of an idle tail. Reports done
    /// normally once nothing else is pending. Ignored (with a warning) when
    /// combined with [`self_completing`](Self::self_completing) - a
    /// script-owned run decides its own repetition.
    pub fn loop_while_pending(mut self) -> Self {
        self.loop_while_pending = true;
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
    self_completing: bool,
    loop_while_pending: bool,
    loops: u32,
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

        let loop_while_pending = if self.loop_while_pending && self.self_completing {
            warn!(
                "AutopilotPlugin: loop_while_pending is ignored with self_completing \
                 (a script-owned run decides its own repetition)"
            );
            false
        } else {
            self.loop_while_pending
        };
        app.insert_resource(AutopilotState::<S> {
            schedule: self.schedule.clone(),
            input: self.input.clone(),
            index: 0,
            elapsed: 0.0,
            state_elapsed: 0.0,
            started: false,
            done: false,
            self_completing: self.self_completing,
            loop_while_pending,
            loops: 0,
        });
        app.add_message::<AutopilotLoop>();
        completion::register(app, completion::AUTOPILOT);
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

    // The timeline is finished (or the script reported done first); stay
    // inert (do not index past the end of the schedule) while the
    // completion watcher waits for any other collectors.
    if st.done
        || (st.self_completing
            && !world
                .resource::<completion::HarnessCompletion>()
                .is_pending(completion::AUTOPILOT))
    {
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

    // In the LOOPING regime, finish the moment the other collectors are
    // done instead of waiting for the current cycle's end - a slow cycle
    // otherwise wastes up to its full length after a capture completes,
    // and can straddle the completion deadline into a false laggard.
    if st.loops > 0
        && !world
            .resource::<completion::HarnessCompletion>()
            .others_pending(completion::AUTOPILOT)
    {
        info!(
            "autopilot: collectors done after {} loop(s); cycle complete, no panic (t={:.1}s)",
            st.loops, st.elapsed
        );
        world
            .resource_mut::<completion::HarnessCompletion>()
            .done(completion::AUTOPILOT);
        st.done = true;
        world.insert_resource(st);
        return;
    }

    let hold = st.schedule[st.index].1;
    if st.state_elapsed >= hold {
        st.index += 1;
        if st.index >= st.schedule.len() {
            if st.loop_while_pending
                && world
                    .resource::<completion::HarnessCompletion>()
                    .others_pending(completion::AUTOPILOT)
            {
                st.loops += 1;
                info!(
                    "autopilot: cycle {} restarting - other collectors still pending",
                    st.loops
                );
                world.write_message(AutopilotLoop);
                // Stay on the final step (no state transitions); zero the
                // clocks so the input script sees a fresh cycle.
                st.index = st.schedule.len() - 1;
                st.elapsed = 0.0;
                st.state_elapsed = 0.0;
                world.insert_resource(st);
                return;
            }
            if st.self_completing {
                // The runway expired with the script still pending: an
                // ABORT, not a completion (error exits do not negotiate).
                error!(
                    "autopilot: timeline expired but the self-completing \
                     script never reported done (t={:.1}s)",
                    st.elapsed
                );
                world.write_message(AppExit::error());
            } else {
                info!("autopilot: cycle complete, no panic (t={:.1}s)", st.elapsed);
                world
                    .resource_mut::<completion::HarnessCompletion>()
                    .done(completion::AUTOPILOT);
            }
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
