//! Cross-platform persistence for a Bevy [`Resource`].
//!
//! Every game wants a couple of values to survive a restart -- a high score, a
//! settings blob -- but the storage plumbing differs per platform, so games
//! leave "high score" as an in-memory resource that resets on launch. This
//! module owns that plumbing once: [`PersistPlugin<T>`] loads a serializable
//! resource from durable storage on startup and writes it back whenever it
//! changes, on both native and wasm.
//!
//! - Native: the resource is stored as JSON under the OS data directory
//!   (`dirs::data_dir()/bevy_common_systems/<key>.json`), or under
//!   `$BCS_PERSIST_DIR` if that environment variable is set.
//! - Wasm: it is stored in `localStorage` under `bevy_common_systems.<key>`.
//!
//! The value type must be a `Resource` that is `Serialize + DeserializeOwned +
//! Default`. Add the plugin with a unique, filename-safe key; the resource is
//! then loaded before `Startup` (falling back to `Default` when nothing is
//! stored or a stored blob fails to parse) and auto-saved on change.
//!
//! ```rust
//! # use bevy::prelude::*;
//! # use bevy_common_systems::prelude::*;
//! # use serde::{Serialize, Deserialize};
//! #[derive(Resource, Default, Serialize, Deserialize)]
//! struct HighScore(u32);
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     // Loads the stored HighScore on startup, saves it whenever it changes.
//!     .add_plugins(PersistPlugin::<HighScore>::new("my_game.high_score"));
//!
//! fn on_new_best(mut high: ResMut<HighScore>, score: u32) {
//!     if score > high.0 {
//!         high.0 = score; // the change auto-persists
//!     }
//! }
//! ```

use std::marker::PhantomData;

use bevy::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

mod backend;

pub mod prelude {
    pub use super::{PersistKey, PersistPlugin, PersistSystems};
}

/// System sets for the persist plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PersistSystems {
    /// Saves the resource when it changes. Runs in `Update`. (Loading happens
    /// once in `Plugin::build`, not in a system, so the resource exists before
    /// any startup system or state transition can read it.)
    Save,
}

/// The storage key for a persisted resource of type `T`, inserted by
/// [`PersistPlugin`]. Read only if you save `T` manually; the plugin's
/// auto-save handles the common case.
#[derive(Resource, Debug, Clone)]
pub struct PersistKey<T> {
    /// The unique, filename-safe key the value is stored under.
    pub key: String,
    _marker: PhantomData<fn() -> T>,
}

/// Loads a serializable [`Resource`] `T` on startup and persists it on change.
///
/// See the [module docs](self) for the storage locations and value-type bounds.
pub struct PersistPlugin<T> {
    key: String,
    _marker: PhantomData<fn() -> T>,
}

impl<T> PersistPlugin<T> {
    /// Persist `T` under `key` (a unique, filename-safe string, e.g.
    /// `"my_game.high_score"`).
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            _marker: PhantomData,
        }
    }
}

impl<T: Resource + Serialize + DeserializeOwned + Default> Plugin for PersistPlugin<T> {
    fn build(&self, app: &mut App) {
        debug!("PersistPlugin: build (key {})", self.key);

        // Load synchronously here, not in a system: the resource must exist
        // before any startup system or the initial state transition reads it
        // (the game's menu, say, shows the stored high score). It is one small
        // file read.
        app.insert_resource(PersistKey::<T> {
            key: self.key.clone(),
            _marker: PhantomData,
        })
        .insert_resource(load_value::<T>(&self.key))
        .add_systems(
            Update,
            save_persisted::<T>
                .run_if(resource_exists::<T>)
                .run_if(resource_changed::<T>)
                .in_set(PersistSystems::Save),
        );
    }
}

/// Read and deserialize the persisted value, or `Default` when nothing is stored
/// (or a stored blob fails to parse).
fn load_value<T: DeserializeOwned + Default>(key: &str) -> T {
    match backend::load(key) {
        Some(raw) => match serde_json::from_str::<T>(&raw) {
            Ok(value) => {
                trace!("persist: loaded {key}");
                value
            }
            Err(err) => {
                warn!("persist: stored {key} failed to parse, using default: {err}");
                T::default()
            }
        },
        None => {
            trace!("persist: no stored {key}, using default");
            T::default()
        }
    }
}

/// Serialize and store the resource whenever it changes.
fn save_persisted<T: Resource + Serialize>(value: Res<T>, key: Res<PersistKey<T>>) {
    match serde_json::to_string(value.as_ref()) {
        Ok(raw) => {
            trace!("persist: saving {}", key.key);
            backend::save(&key.key, &raw);
        }
        Err(err) => warn!("persist: {} failed to serialize: {err}", key.key),
    }
}

// The wasm backend needs a browser, so the end-to-end plugin test is native-only
// (the pure serialize/deserialize path is shared). It points the storage at a
// temp dir via `BCS_PERSIST_DIR`, so it is hermetic and does not touch the real
// data directory.
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Resource, Default, Serialize, Deserialize, PartialEq, Debug)]
    struct Best(u32);

    fn run_once(key: &str) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, PersistPlugin::<Best>::new(key)));
        app.update(); // PreStartup load, then first Update save
        app
    }

    #[test]
    fn value_survives_across_two_app_runs() {
        let dir = std::env::temp_dir().join(format!("bcs_persist_plugin_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        // This is the ONLY test that sets `BCS_PERSIST_DIR` / drives the plugin's
        // real storage; any new plugin-level storage test must share this fixture
        // (or serialize), or it will race on the process-global env var.
        std::env::set_var("BCS_PERSIST_DIR", &dir);

        // First launch: loads the default (nothing stored yet), then a change is
        // written back.
        {
            let mut app = run_once("best");
            assert_eq!(
                *app.world().resource::<Best>(),
                Best(0),
                "default on a clean store"
            );
            app.world_mut().resource_mut::<Best>().0 = 7; // change -> auto-saves
            app.update();
        }

        // Second launch: a fresh app loads the value the first one saved.
        {
            let app = run_once("best");
            assert_eq!(
                *app.world().resource::<Best>(),
                Best(7),
                "restored across launches"
            );
        }

        std::env::remove_var("BCS_PERSIST_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
