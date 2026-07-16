# persist

Cross-platform save and load for a single Bevy `Resource`. A game usually wants
a couple of values to survive a restart -- a high score, a settings blob -- but
the storage plumbing differs per platform. This module owns that plumbing once:
[`PersistPlugin<T>`] loads a serializable resource on startup and writes it back
whenever it changes, on both native and wasm.

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
```

## Persisting a resource

The value type must be a `Resource` that is `Serialize + DeserializeOwned +
Default`. Add the plugin with a unique, filename-safe key:

```rust
#[derive(Resource, Default, serde::Serialize, serde::Deserialize)]
struct HighScore(u32);

fn wire_up(app: &mut App) {
    app.add_plugins(DefaultPlugins)
        // Loads the stored HighScore on startup, saves it whenever it changes.
        .add_plugins(PersistPlugin::<HighScore>::new("my_game.high_score"));
}
```

After that, just mutate the resource and the change auto-persists:

```rust
fn on_new_best(mut high: ResMut<HighScore>, score: u32) {
    if score > high.0 {
        high.0 = score; // the change is written back automatically
    }
}
```

The plugin also inserts a [`PersistKey<T>`] resource holding the key string
(read it only if you save `T` by hand) and puts its auto-save system in the
[`PersistSystems`]`::Save` set, in `Update`.

## Native vs wasm

The two backends expose the same load/save pair over raw JSON strings, under a
`bevy_common_systems` namespace so different games do not collide:

- Native: stored as JSON under the OS data directory,
  `dirs::data_dir()/bevy_common_systems/<key>.json`, or under the directory in
  the `BCS_PERSIST_DIR` environment variable when it is set (handy for a
  portable install, a sandbox, or a test). Keys containing a path separator
  (`/`, `\`) are refused so a typo cannot escape the namespace.
- Wasm: stored in the browser's `localStorage` (via `web-sys`) under
  `bevy_common_systems.<key>`.

Your game code is identical on both; only the backend differs, selected at
compile time by the target arch. See [web-builds](../web-builds/) for wasm
packaging.

## Save and load

Loading happens synchronously in `Plugin::build`, not in a system, so the
resource exists before any startup system or state transition reads it (a menu
showing the stored high score, say). If nothing is stored, or a stored blob
fails to parse, the plugin falls back to `T::default()` and logs a warning
rather than failing.

Saving is a change-detected system: it serializes `T` to JSON and writes it
through the backend only on the frames `T` actually changes
(`resource_changed::<T>`). You normally never call load or save yourself -- the
plugin covers the common case, and mutating the resource is the whole API.
