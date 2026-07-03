# 08_dropzone: land a ship on the noise planet with PD controller

- STATUS: OPEN
- PRIORITY: 50
- TAGS: feature,example

Second pick from the 01-05 games spike (see
`docs/2026-07-03-example-games-ideation.md`). Hover a ship over the
noise-displaced planet; rotate it toward a target orientation with
`PDControllerPlugin` (avian3d torque); touch down softly on flat terrain for
points, crash too fast and break apart via `mesh/explode`. Add a `SkyboxPlugin`
starfield and `camera/post` bloom on the thrusters; gauges (altitude/fuel/
speed) via `ui/status`. Follow the 06_fruitninja shape (states, sounds, wasm).

The only credible gameplay demo of `physics/pd_controller`, and it pulls in
`camera/skybox` + `camera/post` at once. Higher physics-tuning risk, so it
comes after 07_orbit. Needs a collider from the planet mesh (avian trimesh);
consider extracting a `collider-from-TriangleMeshBuilder` helper if a second
game needs it too. Grows out of `examples/02_planet`.

Scope: this is a library example. Keep it small (~1000 LoC), basic but fun for
~15 minutes -- a landing challenge, not a full flight sim.

