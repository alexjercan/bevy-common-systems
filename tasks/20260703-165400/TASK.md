# 11_overload: dashboard-survival game on the status bar

- STATUS: OPEN
- PRIORITY: 20
- TAGS: feature,example

Low-priority pick from the 01-05 games spike (see
`docs/2026-07-03-example-games-ideation.md`). A reaction game whose entire
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
