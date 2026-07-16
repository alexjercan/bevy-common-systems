# 09_reactor -- rules-as-machine incremental on the modding bus

`examples/09_reactor.rs` is the headline demo of the `modding` module. It is an
idle/incremental game where the modding event bus from `examples/03_modding` is
not a side feature but the entire game mechanic: the player builds their reactor
at runtime by installing `EventHandler` entities. It grows out of `03_modding`
(same events, registry, filters and actions) and follows the `06_fruitninja`
shape for everything around the core loop (states, sounds, wasm gallery build).

## What it is

A `ReactorWorld` holds three resources -- ENERGY, HEAT and CREDITS. The engine
`fire`s a `tick` event every half second (the idle heartbeat) and `click` /
`sell` events when you tap the controls. Every rule that reacts to those events
is a JSON-authored handler:

- Two **built-in** handlers ship with every reactor: "Manual Tap" (`click` ->
  +energy) and "Sell" (`sell` -> convert all energy to credits). They wire the
  manual controls onto the bus so even hand-play goes through the event queue.
- The **shop** is a palette of six machine parts, each a `HandlerSpec`. Buying a
  part charges its (geometrically scaling) credit cost and spawns another
  `EventHandler` entity: a Solar Array (+energy, clean), a Fuel Rod (+energy AND
  +heat, the core loop), a Heat Sink and Coolant Pump (shed heat), a Market
  Uplink (auto-sells energy for credits) and a Turbine (recovers very-high heat
  back into energy).

The tension is heat: fuel rods are the real scaling engine (clean Solar is a
deliberately weak bootstrap), but they add heat, and the grid adds ambient heat
that grows with every credit "tier" you cross. The tier ladder is *uncapped* and
geometric, so ambient heat scales with everything you have ever earned and never
plateaus: there is no set-and-forget equilibrium -- you have to keep expanding
cooling as you grow, and a lapse lets HEAT reach 100 and melts the reactor down,
ending the run. Score is total credits earned, kept as a per-process best and
shown on the menu and game-over screens.

Menu / Playing / GameOver are Bevy `States`, exactly like 06/07/08/11. Every
gameplay beat (tap, buy, alarm, tier-up, menu select, meltdown) plays a one-shot
through `SfxPlugin`.

## Modules exercised

- `modding` -- the whole simulation. `ReactorWorld` is the `EventWorld`;
  `TickEvent` / `ClickEvent` / `SellEvent` are `EventKind`s fired with
  `commands.fire`; the filters (`min_energy`, `min_heat`) and actions
  (`add_energy`, `add_heat`, `add_credits`, `sell_all`) are Rust trait objects
  registered by name on the `EventHandlerRegistry`. The shop specs and built-ins
  are JSON `HandlerSpec`s built with `build_handler`, so the reactor is just the
  event queue draining every frame. This is the module's first use where the
  handlers are authored/installed *as gameplay* rather than fixed at startup.
- `ui/status` -- a compact telemetry HUD (TIER, PARTS, FPS) in the corner, its
  `value_fn` closures reading `ReactorWorld` / `Shop` straight out of the World.
- `audio` -- one-shot SFX for every beat, reusing existing placeholders.

No 3D scene is needed, so it renders with a plain `Camera2d` and a dark
`ClearColor`.

## Design decisions

- **The rules ARE the mod system.** The task framed this as a game where placing
  `EventHandler` entities is the play. So the shop palette is literally a list of
  `HandlerSpec`s, buying a part is a `build_handler` call, and an installed part
  is an `EventHandler` entity tagged `InstalledHandler`. The "gameplay" and the
  "modding data" are the same artefact -- which is the whole point of the demo.
  Even the manual TAP/SELL buttons fire events onto the bus rather than mutating
  the world directly, so nothing bypasses the queue.

- **`ReactorWorld` is the state; sync is a no-op.** `03_modding` mirrors a
  separate `SomeCounter` resource through `world_to_state_system` /
  `state_to_world_system`. Here the `EventWorld` resource *is* the single source
  of truth (the queue hands it to every action, the UI reads it back), so both
  sync systems are empty. This is the simpler, more common shape and worth
  showing alongside 03's mirrored variant.

- **Filters gate, actions transfer.** Rather than one bespoke action per part,
  the parts compose a small vocabulary: signed `add_energy` / `add_heat` (a
  negative amount is a cooler or a cost), `add_credits`, and `min_energy` /
  `min_heat` filters that gate the expensive parts (the pump only runs when hot
  and you can afford it; the market only sells when there is surplus). This keeps
  the JSON legible and shows real multi-filter, multi-action handlers. `sell_all`
  is the one bespoke action, because the manual sell's amount is dynamic (all
  current energy) rather than a fixed param like the Market Uplink.

- **Built-in specs authored from the tuning constants.** The Manual Tap and Sell
  handler JSON is built with `format!` from `TAP_ENERGY` / `SELL_RATE` so the
  documented constants and the JSON literals cannot drift apart (the shop parts
  use static amounts baked into their spec strings, kept next to their display
  metadata).

- **Score = held + spent.** Credits are spent installing parts, so "current
  credits" is not the score. The `Shop` tracks total credits `spent`, and the
  score is `credits + spent` = every credit ever earned. Grid tiers key off that
  cumulative number, not the current balance.

## Tuning

The economy constants (tick interval, tap/sell amounts, ambient-heat-per-tier,
part costs and growth, heat thresholds, tier thresholds) are collected at the top
of the file with doc comments. They were reasoned first and then checked with a
temporary env-gated autopilot harness that drove the whole state machine
headlessly (menu -> playing -> meltdown -> menu) and logged the energy/heat/
credits arc; the harness was removed before commit, per the standing "verify
stateful gameplay headlessly, then remove the harness" gotcha. A run is meant to
bootstrap by hand in the first minute, idle-scale in the middle, and stay a heat
juggle as tiers climb. They are easy to retune in one place.

## Sounds

No new placeholder sounds were added: the reactor reuses `pickup` (manual tap),
`golden` (buying a part), `alarm` (heat critical) and `level_up` (grid tier up),
plus the shared `menu_select` and `game_over`. As with the other games these are
generated sine blips; drop real WAVs in at the same paths with no code change.
See `assets/sounds/README.md`.

## Web / wasm

Wired into the gallery like 06/07/08/11: a `web/games/09_reactor/index.html` (a
copy of the 11_overload page with the title/comments changed; it keeps the shared
audio-unlock shim and the `assets/sounds` copy-dir), an entry in
`web/scripts/build-games.sh`, and a `Game` entry in `web/src/games.ts`. The whole
game is UI, so no image asset is shipped.

## Verification

- `cargo build --example 09_reactor`, `cargo fmt --check`,
  `cargo clippy --all-targets` and `--features debug` (clean),
  `scripts/check-ascii.sh`.
- `cargo test --example 09_reactor` -- nine in-file tests: the actions mutate and
  clamp, `sell_all` converts all energy, the filters gate on world state, every
  part and built-in spec builds from its JSON against the registry, cost growth,
  tier counting, score arithmetic and the heat-colour thresholds. One is an
  end-to-end integration test: it installs the Fuel Rod handler from JSON, fires
  a `tick` through the real `GameEventsPlugin` queue, and asserts the world's
  energy and heat rose -- driving the whole registry -> queue -> action path, not
  just the pieces.
- Ran the example on a display: it boots to the render loop (window created,
  adapter initialised, swap-chain configured) with no panics; the autopilot
  harness above confirmed the full menu/playing/meltdown cycle before removal.
- Scoped `trunk build --release --example 09_reactor` of the new web page.
