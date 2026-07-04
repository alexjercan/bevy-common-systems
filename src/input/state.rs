//! Tiny input-to-state helpers.
//!
//! Games repeatedly wire a single key to a state transition -- "press Escape to
//! give up" is the same three-line system in every example. [`set_state_on_key`]
//! is that system as a factory: give it a key and a target state and it returns
//! a ready-to-add system.
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! # #[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
//! # enum GameState { #[default] Playing, GameOver }
//! # let mut app = App::new();
//! # app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
//! #     .init_state::<GameState>();
//! // Escape gives up the current run.
//! app.add_systems(
//!     Update,
//!     set_state_on_key(KeyCode::Escape, GameState::GameOver)
//!         .run_if(in_state(GameState::Playing)),
//! );
//! ```

use bevy::{prelude::*, state::state::FreelyMutableState};

pub mod prelude {
    pub use super::set_state_on_key;
}

/// Build a system that sets `target` as the next state when `key` is just
/// pressed.
///
/// The returned system takes `Res<ButtonInput<KeyCode>>` and
/// `ResMut<NextState<S>>`; gate it with `run_if(in_state(...))` to scope the
/// transition to a phase (for example only give up while `Playing`).
pub fn set_state_on_key<S: FreelyMutableState + Clone>(
    key: KeyCode,
    target: S,
) -> impl FnMut(Res<ButtonInput<KeyCode>>, ResMut<NextState<S>>) {
    move |keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<S>>| {
        if keys.just_pressed(key) {
            next.set(target.clone());
        }
    }
}
