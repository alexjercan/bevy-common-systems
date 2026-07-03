# 07_orbit: surface-dodge game on a sphere (Orbit Runner)

- STATUS: CLOSED
- PRIORITY: 55
- TAGS: feature,example

Top pick from the 01-05 games spike (see
`docs/2026-07-03-example-games-ideation.md`). Ride a marker on a sphere's
surface steering with `directional_sphere_orbit`; obstacles/pickups wander via
`random_sphere_orbit`; a `ChaseCamera` follows with `LerpSnap` smoothing; a hit
ends the run via `HealthPlugin`. Follow the 06_fruitninja shape: menu/playing/
game-over states, `SfxPlugin` sounds, a wasm/trunk build in the web gallery.

Exercises the whole `transform/*` orbit family, `camera/chase` and `meth`,
none of which any current example demos under gameplay. Grows out of
`examples/01_sphere`.

Scope: this is a library example. Keep it small (~1000 LoC), basic but fun for
~15 minutes -- dodge and collect on a sphere, not an open-world runner.

