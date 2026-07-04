//! Platform storage backend for [`persist`](super): a file under the OS data
//! directory on native, `localStorage` on wasm. Both expose the same
//! `load(key) -> Option<String>` / `save(key, data)` pair over raw JSON strings.

/// Namespace all keys live under: a data subdirectory on native, a key prefix
/// on wasm, so different crates/games do not collide.
const NAMESPACE: &str = "bevy_common_systems";

#[cfg(not(target_arch = "wasm32"))]
pub use native::{load, save};
#[cfg(target_arch = "wasm32")]
pub use web::{load, save};

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::path::{Path, PathBuf};

    use bevy::prelude::warn;

    use super::NAMESPACE;

    /// The per-crate data directory, e.g. `~/.local/share/bevy_common_systems`.
    /// Overridable with the `BCS_PERSIST_DIR` environment variable (a portable
    /// install, a sandbox, or a test can point storage elsewhere).
    fn root() -> Option<PathBuf> {
        if let Some(dir) = std::env::var_os("BCS_PERSIST_DIR") {
            return Some(PathBuf::from(dir));
        }
        dirs::data_dir().map(|dir| dir.join(NAMESPACE))
    }

    /// The file a `key` maps to under `root`.
    fn path_in(root: &Path, key: &str) -> PathBuf {
        root.join(format!("{key}.json"))
    }

    /// Reject keys that would escape the storage directory. Keys are meant to be
    /// filename-safe constants; a path separator would let a typo'd key
    /// (`"a/b"`, `"../x"`) write outside the namespace, so guard against it.
    fn is_safe_key(key: &str) -> bool {
        !key.is_empty() && !key.contains(['/', '\\'])
    }

    /// Read a stored value from an explicit root (the testable core of [`load`]).
    fn load_in(root: &Path, key: &str) -> Option<String> {
        std::fs::read_to_string(path_in(root, key)).ok()
    }

    /// Write a value under an explicit root (the testable core of [`save`]).
    fn save_in(root: &Path, key: &str, data: &str) -> std::io::Result<()> {
        let path = path_in(root, key);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, data)
    }

    pub fn load(key: &str) -> Option<String> {
        if !is_safe_key(key) {
            warn!("persist: refusing unsafe key {key:?}");
            return None;
        }
        load_in(&root()?, key)
    }

    pub fn save(key: &str, data: &str) {
        if !is_safe_key(key) {
            warn!("persist: refusing to save unsafe key {key:?}");
            return;
        }
        let Some(root) = root() else {
            warn!("persist: no data directory available, cannot save {key}");
            return;
        };
        if let Err(err) = save_in(&root, key, data) {
            warn!("persist: failed to write {key}: {err}");
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// A unique temp root per test process so the round-trip does not touch
        /// the real data directory or collide across tests.
        fn temp_root(tag: &str) -> PathBuf {
            std::env::temp_dir().join(format!("bcs_persist_test_{tag}_{}", std::process::id()))
        }

        #[test]
        fn save_then_load_round_trips() {
            let root = temp_root("roundtrip");
            let _ = std::fs::remove_dir_all(&root);

            assert_eq!(load_in(&root, "score"), None, "nothing stored yet");
            save_in(&root, "score", "{\"value\":42}").unwrap();
            assert_eq!(load_in(&root, "score"), Some("{\"value\":42}".to_string()));

            // Overwriting replaces the stored value.
            save_in(&root, "score", "{\"value\":99}").unwrap();
            assert_eq!(load_in(&root, "score"), Some("{\"value\":99}".to_string()));

            let _ = std::fs::remove_dir_all(&root);
        }

        #[test]
        fn keys_map_to_distinct_files_under_the_namespace() {
            let root = Path::new("/tmp/bcs");
            assert_eq!(path_in(root, "a"), Path::new("/tmp/bcs/a.json"));
            assert_ne!(path_in(root, "a"), path_in(root, "b"));
        }

        #[test]
        fn unsafe_keys_are_rejected() {
            // Kept pure (no env / storage) so it never races the env-based
            // two-app test on `BCS_PERSIST_DIR`; `load`/`save` gate on this.
            assert!(is_safe_key("my_game.high_score"));
            assert!(is_safe_key("a-b_c.1"));
            assert!(!is_safe_key(""));
            assert!(
                !is_safe_key("a/b"),
                "a path separator escapes the namespace"
            );
            assert!(!is_safe_key("../evil"));
            assert!(!is_safe_key("a\\b"));
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use bevy::prelude::warn;

    use super::NAMESPACE;

    /// The `localStorage` key a `key` maps to.
    fn full_key(key: &str) -> String {
        format!("{NAMESPACE}.{key}")
    }

    /// The window's `localStorage`, if available (absent in a worker or when the
    /// browser blocks storage).
    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    pub fn load(key: &str) -> Option<String> {
        storage()?.get_item(&full_key(key)).ok()?
    }

    pub fn save(key: &str, data: &str) {
        let Some(storage) = storage() else {
            warn!("persist: localStorage unavailable, cannot save {key}");
            return;
        };
        if storage.set_item(&full_key(key), data).is_err() {
            warn!("persist: failed to write {key} to localStorage");
        }
    }
}
