# 10_asteroids: slice-on-hit 3D shooter (fragments as physics bodies)

- STATUS: CLOSED
- PRIORITY: 15
- TAGS: feature,example

Lowest-priority pick from the 01-05 games spike (see
`tasks/20260703-165138/NOTES.md`). 3D asteroids: a ship fires at
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

## Implementation notes

Built `examples/10_asteroids.rs` (~1050 LoC). Full design rationale in
`tasks/20260703-170744/NOTES.md`; summary:

- Top-down bounded arena on the XY plane in zero gravity (`Gravity(Vec3::ZERO)`).
  Asteroids are `RigidBody::Dynamic` avian bodies that drift, bounce off four
  static walls (`Restitution`), and are pinned to the plane with `LockedAxes`.
- The headline: a bullet hit inserts `ExplodeMesh`; the `on_fragments_spawned`
  observer respawns each sliced shard as a smaller dynamic body (recentered on
  its centroid so it tumbles in place) that inherits the parent's drift plus an
  outward burst. A `generation` cap (large -> medium -> small -> gone) keeps the
  field clearable. Rocks clear -> next, busier wave.
- Ship is a `RigidBody::Kinematic` (velocity + facing authored each frame, rocks
  bounce off it, clamped to the arena by reflecting its own velocity); its cone
  model + flame are children, decoupled from the sphere collider. Bullets are
  kinematic `Sensor` bodies. `CollisionLayers` keep asteroids from colliding with
  each other (classic-accurate + avoids shard-overlap blowups on a split).
  `CollisionStart` messages drive bullet hits and ship damage; `HealthPlugin`
  owns the hull / lose condition with an i-frame blink.
- `camera/post` blooms the emissive bullets and thruster flame. `fit_camera`
  frames the square arena at any aspect (portrait mobile + landscape desktop).
- Unified keyboard + pointer controls (A/D rotate, W thrust, Space fire, or hold
  the mouse / a finger to fly toward it and auto-fire), read directly from
  `ButtonInput` + `Touches` -- no `bevy_enhanced_input` -- so the wasm build is
  touch-playable. Registered in the web gallery (index.html, build-games.sh,
  games.ts).

### Decisions and alternatives

- Bounded bouncing arena over classic wraparound: wraparound fights avian's
  solver and breaks collision continuity; walls give real rigid-body bounces,
  which is the whole point of this example. The one deliberate departure from
  "classic asteroids".
- Sliced wedges reused as the bodies (not fresh small octahedra) to honor "the
  fragments keep drifting as real bodies"; recentering handles the off-center
  spin that would otherwise result.

### Difficulties

- Broad avian 0.7 surface not used by 08_dropzone (sensors, `CollisionLayers` /
  `PhysicsLayer` derive, restitution/friction, `LockedAxes`, kinematic bodies
  moved by `LinearVelocity`). Verified every piece against the avian source
  before writing -> compiled first try, zero API errors.
- `B0004` at runtime: the ship parent carries no mesh (model/flame are
  children), so it needed an explicit `Visibility` for propagation. Caught only
  by actually running the example, per the standing retro lesson.

### Verification

- `cargo build/clippy (both feature configs)/fmt/test` all clean; `check-ascii`
  clean. 9 in-file tests including a headless integration test
  (`splitting_a_rock_spawns_smaller_physics_bodies`) that drives the real
  ExplodeMesh -> body-respawn path.
- Ran on a display: reaches the render loop, enters Playing (ship + wave), no
  panic. wasm/trunk build of the example verified through `trunk build`.

### Self-reflection

Front-loading the avian source reading was the big win again -- a ~1000-line,
physics-heavy example with no compile-error round-trips. The one miss (B0004)
was a hierarchy/visibility subtlety that only a real run surfaces, reinforcing
"run it, do not just build it". Next time, when a parent entity has mesh
children but no mesh of its own, add `Visibility` up front.

