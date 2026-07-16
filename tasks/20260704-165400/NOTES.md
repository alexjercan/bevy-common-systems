# 11_overload -- dashboard-survival on the status bar

`examples/11_overload.rs` is the headline demo of the `ui/status` module: a small
reaction/survival game whose entire play surface is a `status_bar` full of
`status_bar_item` gauges. It grows out of `examples/04_status_item` (which shows
the bar with FPS / kernel / version items) and follows the `06_fruitninja` shape
for everything around the core mechanic (states, sounds, wasm gallery build).

## What it is

You run a failing reactor. Four gauges -- HEAT, PRES, FLUX, CHRG -- climb and
random-walk upward on their own. Each is a `status_bar_item` whose `color_fn`
shades the reading green (calm) -> amber (>= 60) -> red (>= 85). Press the number
key under a gauge (1/2/3/4, or the numpad) to vent it back down. The catch:
venting one gauge pushes a coupled neighbour up (the coupling forms a cycle
0->1->2->3->0), so there is no free vent and you are always juggling.

While any gauge sits in the red, the reactor's `Health` drains (through
`HealthPlugin`'s `HealthApplyDamage` event, scaled by how many gauges are red)
and an alarm beeps. When the hull hits zero (`HealthZeroMarker`) the run ends at a
meltdown screen. A difficulty level ramps every 14 s, multiplying every gauge's
climb rate, so the console eventually outruns you. Score is how long you lasted;
a best time is kept for the process and shown on the menu and meltdown screens.

Menu / Playing / GameOver are Bevy `States`, exactly like 06/07/08. Every event
(vent, alarm, level-up, menu select, meltdown) plays a one-shot through
`SfxPlugin`.

## Modules exercised

- `ui/status` -- the whole game board. Gauges, HULL, LVL, TIME and FPS are each a
  `status_bar_item`. The `value_fn` closures read a single `ReactorState`
  resource out of the `World`; the `color_fn` closures downcast the boxed reading
  and threshold it. This is the module's first use as a *game surface* rather than
  a passive metrics overlay.
- `health` -- the lose condition. The reactor is a plain entity carrying
  `Health`; danger applies `HealthApplyDamage` and an `On<Add, HealthZeroMarker>`
  observer flips the state to GameOver (the 07_orbit pattern).
- `audio` -- one-shot SFX for every gameplay beat, including a per-shot pitch
  bump on the alarm as more gauges go red.

No 3D scene is needed, so it renders with a plain `Camera2d` and a dark
`ClearColor`.

## Design decisions

- **Standalone vs folding into 08_dropzone's HUD.** The task offered the
  alternative of adding the gauge mechanic to 08's fuel/altitude/hull HUD instead
  of shipping a standalone game. Shipped standalone: it is the clearest possible
  demo of `ui/status` as an interactive surface, and it keeps 08 focused on the
  PD controller. The overlap with 06 (health/audio) is real but shallow; the new
  ground here is the status bar as gameplay.

- **One `ReactorState` resource as the single source of truth.** The status bar's
  `value_fn` only gets `&World`, so anything shown on the bar has to live in a
  resource or component the closure can read. Putting all four gauges, the level,
  the elapsed time and a mirror of the hull health in one resource means every
  item is a two-line closure and the simulation systems all mutate one place. The
  hull is mirrored from the `Health` component each frame (`mirror_health`)
  because the bar cannot see the component directly.

- **Typed readings (`GaugeReading` / `HullReading`) instead of raw numbers.** The
  bar boxes the `value_fn` result as `dyn Any` and hands it back to `color_fn`, so
  the value type carries both the `Display` formatting (rounded integer percent)
  and the number the threshold logic downcasts to. Two distinct newtypes give the
  gauges and the hull opposite colour ramps (high is bad for a gauge, good for the
  hull) while sharing the same machinery.

- **Coupling makes it a game.** Without coupling the optimal play is to mash every
  key every frame. The vent -> neighbour-up cycle turns it into a juggling act:
  you have to choose which gauge to relieve knowing it worsens another. The unit
  tests pin the coupling cycle (every gauge is some other gauge's partner, none
  couple to themselves) so a future edit cannot silently reintroduce a free vent.

- **Closure-returning colour fns.** `gauge_color_fn()` / `hull_color_fn()` return
  `impl Fn(Box<&dyn Any>) -> Option<Color>` rather than being free functions with
  a `Box<&dyn Any>` parameter. That matches the crate's own `status_fps_color_fn`
  and avoids clippy's `boxed_local` / `redundant_allocation` lints, which do fire
  on a free function with that signature but not on the closure form.

## Tuning

The flight-feel constants (climb rates, vent amount, coupling, red-damage rate,
level interval) are collected at the top of the file with doc comments. They were
reasoned first and then checked by running the example; a run is meant to be
frantic but survivable for a couple of minutes with good juggling, ramping until
the console outpaces you. They are easy to retune in one place.

## Sounds

Two new placeholder sounds were added to `scripts/gen-placeholder-sounds.py`:
`vent` (a soft relief blip) and `alarm` (a sharp warning beep). The example also
reuses the shared `menu_select`, `game_over` and `level_up` placeholders. As with
the other games these are generated sine blips; drop real WAVs in at the same
paths with no code change. See `assets/sounds/README.md`.

## Web / wasm

Wired into the gallery like 06/07/08: a `web/games/11_overload/index.html` (a
copy of the 06 page with the title/comments changed; it keeps the shared
audio-unlock shim and the `assets/sounds` copy-dir), an entry in
`web/scripts/build-games.sh`, and a `Game` entry in `web/src/games.ts`. The whole
game is UI, so no image asset is shipped.

## Verification

- `cargo build --example 11_overload`, `cargo fmt --check`,
  `cargo clippy --all-targets` and `--features debug` (clean),
  `scripts/check-ascii.sh`.
- `cargo test --example 11_overload` -- seven in-file unit tests covering the
  colour thresholds (both ramps), the `Display` rounding, the `dyn Any` downcast
  miss, the level climb scaling, `red_count`, and the coupling cycle invariant.
- Ran the example on a display: it boots to the render loop (window created,
  adapter initialised, swap-chain configured) with no panics.
- Scoped `trunk build --release --example 11_overload` of the new web page.
