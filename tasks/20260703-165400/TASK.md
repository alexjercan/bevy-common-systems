# 11_overload: dashboard-survival game on the status bar

- STATUS: CLOSED
- PRIORITY: 20
- TAGS: feature,example,historical

Low-priority pick from the 01-05 games spike (see
`tasks/20260703-165138/NOTES.md`). A reaction game whose entire
display is the status bar: you run a failing machine (reactor / ship / life
support -- pick a skin) with several gauges that climb and drift on their own,
each a `status_bar_item` whose `color_fn` goes green -> amber -> red at
thresholds. Press keys to vent / cool / patch and pull gauges back to green;
let any sit red too long and you lose (`HealthPlugin`), with alarm sounds.

The cheapest game on the list and a genuinely different genre (no 3D scene
needed), but it demos the fewest new modules and overlaps 06 on health/audio,
hence the low priority. Alternative considered in the spike: fold the gauge
mechanic into 08_dropzone's HUD (fuel/altitude/hull) rather than shipping it
standalone -- decide when picking this up.

Scope: this is a library example. Keep it small (~1000 LoC), basic but fun for
~15 minutes. Follow the 06_fruitninja shape (states, sounds, wasm gallery
build). Grows out of `examples/04_status_item`.

## Decision (picking this up)

Ship it standalone (the alternative was folding gauges into 08_dropzone's HUD).
A standalone example is the clearer demo of `ui/status` as a game surface and
keeps 08 focused on the PD controller. Skin: a failing reactor / life-support
console ("OVERLOAD"). The whole game lives on the `status_bar`: four coupled
gauges (HEAT / PRES / FLUX / CHRG) that climb and random-walk on their own, each
a `status_bar_item` with a shared green -> amber -> red `color_fn`. One key vents
each gauge, but venting nudges a coupled partner up, so it is a juggling act.
While any gauge sits red it drains a `Health` entity (via `HealthApplyDamage`);
`HealthZeroMarker` ends the run. Difficulty ramps the climb rates over time.
Follows the 06_fruitninja state/sounds/wasm shape.

## Steps

- [x] Add placeholder sounds `vent` and `alarm` to
  `scripts/gen-placeholder-sounds.py`, regenerate `assets/sounds/`, and update
  `assets/sounds/README.md` (mention 11_overload uses vent/alarm plus shared
  menu_select/game_over/level_up).
- [x] Write `examples/11_overload.rs`: clap header, `DefaultPlugins` (no window
  title -- mirrors 06/07/08), `StatusBarPlugin` + `HealthPlugin` + `SfxPlugin`,
  Camera2d, and the FrameTimeDiagnostics guard. Menu / Playing / GameOver states
  following the 06_fruitninja shape (state-scoped despawn, centered text UI).
- [x] Model the reactor: a `ReactorState` resource holding the four gauges
  (value, climb rate, coupling), difficulty level, elapsed time, and a mirror of
  the player Health for the status bar. `GaugeReading` / `HullReading` newtypes
  that Display as a rounded percent and are downcast by the `color_fn`s.
- [x] Gauge simulation system (Playing only): integrate climb + random-walk
  drift scaled by difficulty; clamp 0..100.
- [x] Input system: one key per gauge vents it (subtract a chunk, add coupling
  to a partner, play `vent`); ramp difficulty over time (play `level_up`).
- [x] Danger system: while any gauge is red, periodically beep `alarm` and
  `commands.trigger(HealthApplyDamage{..})` on the reactor entity draining
  Health; detect `HealthZeroMarker` -> GameOver (play `game_over`). Mirror
  Health into `ReactorState` for the HULL status item.
- [x] Build the status bar in setup: HEAT/PRES/FLUX/CHRG gauges, HULL, LVL, TIME
  (survival), FPS -- each a `status_bar_item` with value_fn/color_fn.
- [x] Score = survival time; keep a high score across runs (`HighScore`
  resource), shown on the menu and meltdown screens.
- [x] Wire the wasm gallery: `web/games/11_overload/index.html` (mirror 06),
  add to `web/scripts/build-games.sh` and `web/src/games.ts`.
- [x] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
  debug`), `cargo test --example 11_overload` (7 tests), `scripts/check-ascii.sh`,
  and ran the example (`$DISPLAY=:0`) -- boots to the render loop, no panics.
  Scoped `trunk build --release --example 11_overload` for the web page.
- [x] Doc note `tasks/20260704-165400/NOTES.md` describing the example and
  design decisions.

## Implementation notes

Built `examples/11_overload.rs` (918 lines incl. tests): four coupled gauges on
the `status_bar`, vented with 1/2/3/4, draining a `Health` entity while red,
ending at a meltdown screen; difficulty ramps the climb over time. Design write-up
lives in `tasks/20260704-165400/NOTES.md`; AGENTS.md's examples list gained an
`11_overload` entry.

Decisions / alternatives: shipped standalone rather than folding the gauges into
08's HUD (clearer `ui/status` demo, keeps 08 focused). One `ReactorState` resource
is the single source of truth because the status bar's `value_fn` only sees
`&World`; the hull is mirrored from the `Health` component each frame. Typed
`GaugeReading` / `HullReading` newtypes carry both the `Display` formatting and the
number the `color_fn` downcasts, giving the gauges and the hull opposite colour
ramps off shared machinery. Coupling (a vent->neighbour-up cycle) is what turns it
from key-mashing into juggling; unit tests pin both the static coupling cycle
(every gauge is another's partner, none couple to themselves) and the runtime
vent math (`apply_vent`: subtract, couple, clamp), so no future edit reintroduces
a free vent.

Difficulties: (1) clippy flagged `boxed_local` / `redundant_allocation` on free
colour functions taking `Box<&dyn Any>`; fixed by returning closures instead
(`gauge_color_fn()` / `hull_color_fn()`), matching the crate's own
`status_fps_color_fn`. (2) Three `needless_range_loop` warnings from `for i in
0..GAUGE_COUNT` indexing; rewrote as `iter`/`zip`/`enumerate`. Both caught by
running clippy up front, not at review. The example compiled first try because the
UI idioms (FontSize::Px, DespawnOnExit, TextLayout, per-camera lighting -- N/A here
since Camera2d) were copied verbatim from 06/08 rather than written from memory,
per the standing retro lesson.

Self-reflection: went smoothly. Front-loading the API/idiom digest (a parallel
Explore agent over 06/07/08 + the crate preludes) meant zero compile errors and no
Bevy-0.19-drift surprises -- the exact opposite of the dropzone cycle. Next time,
run clippy before the first "done" claim as reflexively as `cargo build`; the six
warnings were trivial but would have been noise in review.
