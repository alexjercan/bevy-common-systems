# Orbit Runner (07_orbit): design notes

Task: `tasks/20260703-165427` -- "07_orbit: surface-dodge game on a sphere".
This note records what the example is, why it is built the way it is, the
alternatives weighed, and the bugs/gotchas hit along the way, for future
sessions.

## What it is

`examples/07_orbit.rs` is a small surface-dodge game played on the outside of a
sphere. You pilot a glowing marker that always runs forward across a planet;
you steer left/right. Orbs wander the surface (collect for score), hazards
wander too (touch one and lose health). Survive and the planet fills with more,
faster hazards as difficulty levels climb. Zero health ends the run.

Its real job is to be the first example that drives the crate's whole
spherical-motion family under actual gameplay:

- `DirectionalSphereOrbit` -- the player. Each frame we feed it a direction
  (the marker's surface normal) and read the world position it maps to.
- `RandomSphereOrbit` -- every hazard and orb, wandering on its own.
- `ChaseCamera` (+ `LerpSnap`) -- follows the marker, orbiting the planet.
- `HealthPlugin` -- the lose condition (`HealthZeroMarker` -> game over).
- `SfxPlugin`, `StatusBarPlugin` -- audio and the FPS overlay.

It deliberately mirrors `06_fruitninja`: Bevy states for menu/playing/game-over,
a persistent camera+light, an in-game HUD, generated placeholder sounds, and a
wasm/trunk showcase build. All geometry is procedural (`TriangleMeshBuilder`),
so it grows straight out of `01_sphere`.

## Key design decisions

### Steering by advancing a moving frame, not by driving angles

The marker carries a moving orthonormal frame on the surface: `up` (the outward
normal / the direction handed to the orbit) and `forward` (the unit tangent it
travels along). Each frame `step_runner_frame`:

1. re-orthonormalizes `forward` against `up` (kills accumulated float drift),
2. steers by rotating `forward` about `up`, then
3. travels by rotating both `up` and `forward` about `up x forward` by the arc
   angle `speed * dt / radius`.

Then we write `up` into `DirectionalSphereOrbitInput`; the plugin resolves the
surface position, which we read back for the transform and the camera anchor.

Alternative considered: track `(theta, phi)` and integrate them directly.
Rejected because it has the classic pole singularity (heading and turn rate go
haywire near the poles, and "straight ahead" is not constant in angle space).
The great-circle frame advance is pole-agnostic and keeps a straight course
straight. It is a few more lines but correct everywhere on the sphere, which is
the whole point of a game that roams the entire surface.

The orbit component's own `smoothing` is left at `0.0` for the player: the
smoothing showcase is the *camera* (`ChaseCamera.smoothing` via `LerpSnap`).
Smoothing the direction-to-angle mapping risks a visible spin when `theta`
wraps across `0/TAU`, so we keep the marker's placement exact and put the juice
on the camera, which is the natural place for it.

### Frame rotation for the camera

`frame_rotation(up, forward)` builds the world rotation whose local `-Z` is the
travel direction and local `+Y` is the surface normal, so `ChaseCamera`'s
offset sits it up-and-behind the marker and the focus offset looks ahead. It is
built from `Mat3::from_cols(right, up, back)` with `right = forward x up` and
`back = right x up`, which is right-handed (local `-Z` = forward) and re-derives
an exactly-orthogonal basis even if `up`/`forward` drifted. A unit test pins the
axis alignment so a future refactor cannot silently flip the camera.

### Difficulty and the wandering field

`maintain_objects` tops the field up to a target count every frame, so a
collected orb is replaced and a new level's extra hazards simply appear. Level,
hazard count and wander speed are pure functions of elapsed time
(`level_for` / `hazard_target_for` / `wander_speed_for`), each unit-tested at
its endpoints. Health is a real `Health(3.0)`; a hazard deals `1.0` with a
short invulnerability window (the marker blinks) so one overlap does not drain
all three at once.

### Input

Steering reads keyboard (A/D or arrows) summed with a held pointer's horizontal
offset from screen center, so a mouse or a finger works the same as the keys
(`read_steer`, pure/testable-ish). Menu/game-over "continue" accepts click, tap,
Space or Enter. Unlike `06_fruitninja` this example does not pull in
`bevy_enhanced_input`: steering only needs the raw `ButtonInput`/`Touches`
state, so reading them directly is simpler and one fewer moving part.

## Sounds

Three new placeholder events -- `pickup`, `hurt`, `level_up` -- were added to
`scripts/gen-placeholder-sounds.py` (which now covers both games);
`menu_select` and `game_over` are shared with fruit ninja. See
`assets/sounds/README.md`.

## Web build

Added a trunk page at `web/games/07_orbit/index.html`, an entry in
`web/scripts/build-games.sh`, and a card in `web/src/games.ts`. The page carries
the same WebKit Web-Audio autoplay-unlock shim as fruit ninja (see
`tasks/20260703-200005/RETRO.md`) and the same
`copy-dir` directive that stages `assets/sounds/` into the dist, since the game
loads its sounds at runtime.

## Difficulties / gotchas

- This cycle ran headless: the ECS wiring and camera framing are verified by
  `cargo clippy --all-targets`, the in-module unit tests, and source reasoning,
  not by an interactive playtest. The math that would be hardest to get right
  by eye (the frame stays orthonormal, advancing moves toward the heading, no
  sideways drift with zero steer, the camera axes align) is pinned by tests, so
  the residual risk is feel/tuning (speeds, camera offset), not correctness.
  A click-through playtest is the one thing worth a human pass.
- Heeded the standing retro gotchas: verified build pass/fail via redirect and
  an explicit exit code, never a piped `| tail`; ran `cargo test --example
  07_orbit` (plain `cargo test` does not execute example in-file tests, so it
  would not run these); and mirrored the Safari audio shim rather than shipping
  a silent-on-mobile page.
