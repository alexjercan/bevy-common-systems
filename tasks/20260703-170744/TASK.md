# 10_asteroids: slice-on-hit 3D shooter (fragments as physics bodies)

- STATUS: OPEN
- PRIORITY: 15
- TAGS: feature,example

Lowest-priority pick from the 01-05 games spike (see
`docs/2026-07-03-example-games-ideation.md`). 3D asteroids: a ship fires at
drifting octahedron asteroids; a hit inserts `ExplodeMesh` and the asteroid
slices into fragments that keep drifting as real avian3d bodies (new smaller
hazards), unlike 06 where fragments just despawn. Clear the field without
getting hit (`HealthPlugin` on the ship); `camera/post` bloom on the shots.

Ranked last because it overlaps 06_fruitninja on its headline module (slicing)
-- the gallery would show two "slice stuff" games -- and the camera/physics
coverage it adds is better served by 07_orbit and 08_dropzone. Only build it if
we want a physics-fragments showcase specifically.

Scope: this is a library example. Keep it small (~1000 LoC), basic but fun for
~15 minutes -- classic asteroids, not a space sim. Follow the 06_fruitninja
shape (states, sounds, wasm gallery build). Grows out of `examples/05_explode`.

