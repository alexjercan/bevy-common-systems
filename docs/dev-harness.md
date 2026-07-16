# dev harness: autopilot + screenshot state-driver plugins

- DATE: 2026-07-04
- TASK: `tasks/20260704-175421`
- SPIKE: `tasks/20260704-175058/SPIKE.md` (Wave 1)

## What this is

`src/debug/harness/` is a pair of env-gated developer-tooling plugins, behind
the existing `debug` feature (like `debug/inspector`), that turn the crate's
own "an example is not done until it has been run once" rule from a per-game
hand-rolled-and-deleted harness into one reusable module.

- `AutopilotPlugin<S: States + FreelyMutableState>` -- force-drives a game's
  state machine along a scripted `(state, seconds)` timeline
  (Menu -> Playing -> ... -> GameOver), runs an optional per-frame input
  closure, logs each transition and a final `cycle complete, no panic` line,
  then exits with `AppExit::Success`. Activated by the `BCS_AUTOPILOT` env var.
- `ScreenshotPlugin<S: States + FreelyMutableState>` -- overrides the window
  resolution, advances to a named state, waits N settled frames, writes a PNG
  via Bevy 0.19's `Screenshot::primary_window()` + `save_to_disk`, then exits.
  Activated by the `BCS_SHOT` env var (a `WxH` value also sets the resolution).

Both are inert unless their env var is set, so a game adds them permanently and
pays nothing in a normal run. That is the whole point: the old harness was
re-invented under a fresh env-var name every time (`DROPZONE_AUTOPILOT`,
`OVERLOAD_SMOKE`, `REACTOR_SHOT`, ...) -- 7 autopilots and 2 screenshot
harnesses across the repo history -- and every cycle paid the tax of adding it,
running once, and deleting it before commit. Now it stays in.

## Why this shape

The spike's central open question was **how much of "drive the game" the plugin
can own generically**. The answer the code lands on:

- The plugin owns the **state clock, transition logging, and clean exit** --
  the parts that are identical across every game.
- The game supplies the **timeline** (as `S` values via `.hold(...)`, because
  the variant names differ: 08_dropzone calls its end state `Result`, the
  others `GameOver`, so the plugin cannot assume a `GameOver` variant) and the
  **input** (as a `Fn(&mut World, f32)` closure, because per-frame gameplay
  input has no common shape: dropzone writes a `ShipInput` thrust/lean resource
  feeding a PD controller, overload taps digit-key vents, reactor fires modding
  events). The lowest common driver both can reach is the Bevy input resources,
  so the reference closures poke `ButtonInput<KeyCode>`.

This split keeps the generic core small while letting any game drive its own
gameplay in one closure. The autopilot force-sets `NextState<S>` directly rather
than simulating menu presses, so it does not depend on the game's own
menu-advance input wiring -- it works even for a game whose Playing->end
transition is physics- or health-driven.

### Key API decisions

- **Env-gate in `Plugin::build`, not a run condition.** When the env var is
  unset, `build` returns early and adds no systems or resources -- zero cost.
- **`AppExit::Success`, never `std::process::exit`.** AGENTS.md records that a
  hard exit segfaults on wgpu teardown; `world.write_message(AppExit::Success)`
  is the clean path. (`AppExit` is a `Message` in Bevy 0.19, so it is
  `write_message` / `MessageWriter`, not the old `EventWriter`.)
- **Exclusive driver system for autopilot.** The input closure takes
  `&mut World` (maximum flexibility -- press keys, mutate any resource), so the
  driver is exclusive and removes its own state resource for the closure's
  duration to avoid an aliasing borrow.
- **Screenshot exit after the file lands.** The capture is async (a frame or two
  after the `Screenshot` entity is spawned), so the exit is a second observer on
  the same capture entity -- it fires on `ScreenshotCaptured` right after
  `save_to_disk` has written the PNG synchronously, then sends `AppExit`.
- **`settle_frames` is explicit, default 8.** The spike flagged "how many frames
  to wait after a transition" as undecided; the plugin makes it a builder knob
  so a game with slow intro animation can raise it. The reference examples use
  30 to be safe.

## How to use it

```
# Drive a full menu -> playing -> end cycle headlessly and check for panics:
BCS_AUTOPILOT=1 cargo run --example 08_dropzone --features debug
# look for: `autopilot: -> Playing`, `autopilot: -> Result`,
#           `autopilot: cycle complete, no panic`

# Capture the Playing screen at phone width:
BCS_SHOT=390x844 cargo run --example 11_overload --features debug
# writes screenshot.png and exits
```

In the game's `main()`, behind the same `#[cfg(feature = "debug")]` guard as the
inspector:

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

## Proof

`08_dropzone` (physics: a real avian3d sim, PD controller, radial gravity) and
`11_overload` (2D: `Camera2d`, status-bar gauges) are both wired onto the
harness and verified headlessly via `BCS_AUTOPILOT=1` on this box (`$DISPLAY`
set) -- the log shows every transition and the `cycle complete, no panic` line,
then a clean exit. One 3D physics game and one 2D game were chosen deliberately
(the spike's suggested pair) to confirm the API generalizes across both a
`ShipInput`-resource driver and a `just_pressed` digit-key driver. A
`RUST_LOG=bevy_common_systems=trace` run of 11_overload confirms the input
closure genuinely drives gameplay: 19 vent sound effects fire during Playing
(the vent path exercised, not a silent no-op). The `ScreenshotPlugin` was
likewise verified end to end -- `BCS_SHOT=390x844` writes a valid PNG of the
Playing HUD at phone width with the inspector overlay hidden.

The input closure runs in `PreUpdate` after `InputSystems` (a pinned ordering
edge), so a poked `just_pressed` survives into the game's `Update` input systems
rather than being cleared -- this is what makes the vent path reliable rather
than order-dependent. Because the closure then runs in every state, the example
closures gate themselves to `GameState::Playing` so they do not trip the menu's
"any key to start" transition early.

## Alternatives considered

- **Simulate menu key-presses instead of force-setting `NextState`.** Rejected:
  it would couple the autopilot to each game's menu-advance predicate, and some
  end-state transitions are not input-driven at all (crash, meltdown). Directly
  setting `NextState` is game-agnostic.
- **One combined `DevHarnessPlugin` with both behaviors.** Rejected: autopilot
  and screenshot have different lifecycles and env gates; two small plugins over
  a shared idea read more clearly than one plugin with two modes. They still
  share the module and prelude.
- **Parse the target state from the `BCS_SHOT` string (`"WxH@StateName"`).**
  Rejected: `S` has no `FromStr`, so a string->state map cannot be generic. The
  game passes the target state as an `S` value to `ScreenshotPlugin::new`; the
  env var only carries the optional `WxH` override.
