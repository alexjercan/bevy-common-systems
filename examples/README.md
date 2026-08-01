# Examples

Each `NN_name.rs` here is a small, complete game that headlines one or more of
the crate's modules. They are the quickstart documentation and the de facto
integration tests: a new feature gets its pure logic unit-tested next to the
code and its ECS side wired into an example.

Run one with:

```sh
cargo run --example 01_sphere
# add the world inspector, wireframe toggle and dev harness:
cargo run --example 01_sphere --features debug
```

Every example is a small clap CLI, so `--help` lists its knobs. The games from
`06_fruitninja` on follow one shape: menu / playing / game-over states,
`SfxPlugin` one-shots, and (for most of them) a wasm build in the
[web showcase](../web/README.md).

## The games

| Example | What it is | Headlines |
| --- | --- | --- |
| `01_sphere` | an octahedron sphere with a WASD camera | `mesh/builder`, `camera/wasd` |
| `02_planet` | the same mesh displaced with Fbm/Perlin noise | noise displacement |
| `03_modding` | the modding event bus end to end, `#[derive(EventKind)]`, JSON-authored handlers | `modding` |
| `04_status_item` | the status-bar HUD with FPS and shell-command items | `ui/status` |
| `05_explode` | press Left Mouse Button to slice a mesh into flying fragments | `mesh/explode` |
| `06_fruitninja` | swipe to slice arcing fruit for score; combos, "+N" popups, blade trail, lethal bombs | `audio`, the states shape |
| `07_orbit` | "Orbit Runner": ride a planet surface, sweep orbs, dodge hazards | the whole `transform` orbit family, `camera/chase` |
| `08_dropzone` | a lunar lander onto a noise planet with radial gravity | `physics/pd_controller`, `camera/skybox` + `post` + `chase` |
| `09_reactor` | a rules-as-machine incremental: shop parts spawn JSON `EventHandler`s, HEAT 100 melts down | `modding` as gameplay |
| `10_asteroids` | top-down shooter; each sliced shard respawns as a real dynamic avian body | physics fragments, a broad avian slice |
| `11_overload` | dashboard survival: four coupled gauges, vents, red gauges drain `Health` | `ui/status` as a game surface |
| `12_bastion` | tower defense; tap to place towers, stats loaded from a JSON catalog | `camera/project`, `point_rotation`, `smooth_look_rotation` |
| `13_glide` | a 2048-style slide-merge puzzle, entirely in Bevy UI | `tween`, `ui/animate`, `persist` + `HighScore` |
| `14_breach` | a Doom-like FPS arena with a hitscan `SpatialQuery` gun | `physics/doom_controller`, `ui/touchpad`, `feedback` |
| `15_integrity` | a grid of connected blocks in zero-g; a blast cascades the structure apart | `integrity`, `ui/health_display`, `ui/objectives` |

Per-game design records live in the task that built each one; the example table
in [`AGENTS.md`](../AGENTS.md) carries the task IDs for the larger games.

## Running an example headlessly

`cargo build` proves an example compiles, not that it boots -- and booting only
reaches the menu. `src/debug/harness/` (feature `debug`) holds two env-gated
plugins that drive an example without a human at the keyboard. Do NOT hand-roll
an autopilot: that was re-invented under a fresh env-var name and deleted seven
times before the harness landed.

- **`AutopilotPlugin<S>`** force-drives a game's state machine along a scripted
  `(state, seconds)` timeline, runs an optional per-frame input closure, logs
  each transition and a final `cycle complete, no panic` line, then exits with
  `AppExit::Success`. Activated by `BCS_AUTOPILOT`.
- **`ScreenshotPlugin<S>`** overrides the window resolution, advances to a named
  state, waits N settled frames, writes a PNG, then exits. Activated by
  `BCS_SHOT` (a `WxH` value also sets the resolution).

Both are inert unless their env var is set, so an example adds them permanently
and pays nothing in a normal run. Keep them in.

```sh
# Drive a full menu -> playing -> end cycle and check for panics:
BCS_AUTOPILOT=1 cargo run --example 08_dropzone --features debug
# look for: `autopilot: -> Playing`, `autopilot: -> Result`,
#           `autopilot: cycle complete, no panic`

# Capture the Playing screen at phone width:
BCS_SHOT=390x844 cargo run --example 11_overload --features debug
# writes screenshot.png and exits
```

Wire them into `main()` behind the same `#[cfg(feature = "debug")]` guard as the
inspector (`08_dropzone` and `11_overload` are the reference):

```rust
#[cfg(feature = "debug")]
{
    app.add_plugins(
        AutopilotPlugin::new()
            .hold(GameState::Menu, 0.6)
            .hold(GameState::Playing, 3.0)
            .hold(GameState::Result, 0.8)
            .input(|world, _elapsed| {
                world.resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Space);
            }),
    );
    app.add_plugins(ScreenshotPlugin::new(GameState::Playing).settle_frames(30));
}
```

The input closure runs in `PreUpdate` after `InputSystems`, so a poked
`just_pressed` survives into the game's `Update` input systems. It runs in every
state, so gate it to the playing state or it will trip the menu's "any key to
start" transition early.

### Limits

- The two plugins are mutually exclusive in one run: both drive `NextState`, so
  the screenshot never settles. For a mid-gameplay frame use `ScreenshotPlugin`
  alone, or the autopilot plus an external screen grab.
- A screenshot taken at *state entry* snaps before any input has been applied,
  so it cannot verify gameplay.
- The autopilot force-drives states on a timer, which makes it structurally
  blind to any transition the *game* makes -- above all the lose condition.
  Verify game-driven transitions with a headless `App` test (`MinimalPlugins` +
  `bevy::state::app::StatesPlugin` + the plugin) that drives the trigger and
  asserts the flip.
