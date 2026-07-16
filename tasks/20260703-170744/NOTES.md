# 10_asteroids: slice-on-hit shooter with physics-body fragments

`examples/10_asteroids.rs` is the crate's "physics fragments" showcase and the
counterpoint to `06_fruitninja`: where fruit-ninja slices a shape into fragments
that are hand-integrated and despawn a moment later, asteroids slices a rock and
respawns every shard as a real avian3d rigid body that keeps drifting, bounces
off the arena walls, and becomes a new, smaller hazard you still have to clear.

See `tasks/20260703-170744/TASK.md` for the original ask. This note records what
was built and why, the design decisions, and the difficulties hit along the way.

## What it is

A top-down, single-screen arcade shooter played on the XY plane in zero gravity:

- You fly a ship around a bounded square arena and shoot drifting octahedron
  asteroids (the `02_planet` noise-displaced mesh recipe, pre-scaled to real
  world units so slicing and colliders line up).
- A bullet hit inserts `ExplodeMesh`; the `on_fragments_spawned` observer takes
  each sliced shard and respawns it as a `RigidBody::Dynamic` avian body of the
  next "generation", inheriting the parent rock's drift plus an outward burst.
- Rocks split large -> medium -> small; a hit on the smallest generation just
  destroys it. Clear every body to advance to the next, busier wave.
- Bumping a rock costs a hull point via `HealthPlugin` (with a brief invulnerable
  blink); zero hull ends the run. States drive menu / playing / game-over, and
  `SfxPlugin` plays the one-shots.

Crate pieces exercised: `mesh/explode` (the headline), `camera/post` (bloom on
the glowing bullets and thruster flame), `HealthPlugin`, `SfxPlugin`,
`StatusBarPlugin`, plus a broad slice of avian3d (dynamic/kinematic/static
bodies, `Collider`, `LinearVelocity`/`AngularVelocity`, `Restitution`,
`Friction`, `CollisionLayers`, `LockedAxes`, `Sensor`, `CollisionEventsEnabled`,
`CollisionStart` messages, `Gravity(Vec3::ZERO)`).

## Design decisions

- **Bounded, bouncing arena instead of classic wraparound.** The original game
  wraps at the screen edges. Wraparound means teleporting bodies each frame,
  which fights avian's solver and breaks collision continuity at the seam. Since
  the entire reason this example exists is the physics showcase, the arena is
  instead contained by four static walls with near-elastic `Restitution`, so
  rocks bounce and keep their momentum -- real rigid-body behavior that
  `06_fruitninja`'s hand-rolled kinematics cannot show. This is the one place we
  deliberately depart from "classic asteroids".

- **Asteroids do not collide with each other.** `CollisionLayers` let rocks
  collide with the walls, the ship and bullets but pass through each other. This
  is both classic-accurate (asteroids never bounced off each other in the
  arcade) and, more importantly, avoids a pile of freshly-spawned shards
  resolving mutual overlaps the instant a rock splits (they all start near the
  parent center). It keeps the simulation stable and cheap.

- **Kinematic ship, dynamic rocks, sensor bullets.** The ship is a
  `RigidBody::Kinematic` whose velocity and facing we author every frame (crisp,
  predictable control with inertial drift), so rocks bounce off it but never
  shove it around; a kinematic body is not stopped by the static walls, so the
  ship is clamped to the arena by reflecting its own velocity. Bullets are
  kinematic `Sensor` bodies (detection without a physical shove). The ship's
  visual cone and thruster flame are children, decoupling the look from the
  rotation-invariant sphere collider -- so we never fight avian over the ship's
  `Rotation`.

- **Generation cap for a clearable field.** Each rock carries a `generation`;
  splitting bumps it, and `MAX_SPLIT_GEN` is terminal (a hit there destroys the
  shard with no new bodies). Without the cap, slicing shards into shards would
  never converge and the field would never clear. Up to ~1 + 3 + 9 bodies per
  starting rock keeps entity counts bounded.

- **Unified keyboard + pointer controls.** Classic asteroids is a keyboard game
  (rotate / thrust / fire), which is unplayable one-handed on a phone. To keep
  the wasm/mobile showcase genuinely playable, the pointer scheme is "the ship
  turns to face the pointer; hold to fly toward it and auto-fire, release to
  coast." Both schemes feed the same ship state, so a mouse, a finger or the
  keyboard drive the game identically. Pointer input is read directly from
  `ButtonInput<MouseButton>` + `Touches` (no `bevy_enhanced_input` dependency),
  which keeps the example self-contained.

- **Aspect-fit camera.** `fit_camera` pulls the camera back each frame to frame
  the square arena at the current window aspect, so both a landscape desktop
  window and the portrait mobile canvas show the whole field with no cropping.

## Verification

- Pure helpers (angle wrapping / shortest-arc rotation, wave sizing, drift-speed
  ramp, mesh centroid) have in-file unit tests.
- The headline behavior is covered by a headless integration test
  (`splitting_a_rock_spawns_smaller_physics_bodies`): it drives the real
  `ExplodeMesh` -> `on_fragments_spawned` path in a `MinimalPlugins` app and
  asserts the sliced rock despawns and is replaced by generation-1 dynamic
  bodies with colliders that inherit the parent's drift. This is the CI-visible
  proof the graphical example cannot be.
- The example was run on a display (`cargo run --example 10_asteroids`): it
  reaches the render loop, enters the Playing state (ship + first wave spawned),
  and simulates for the full run with no panic. A `B0004` visibility-propagation
  warning surfaced because the ship parent carries no mesh of its own (its model
  and flame are children); fixed by giving the ship an explicit `Visibility`.

## Difficulties

- **Reading avian 0.7 before writing.** The example uses a chunk of avian that
  `08_dropzone` did not: sensors, collision layers (the `PhysicsLayer` derive),
  restitution/friction, `LockedAxes`, and kinematic bodies moved by
  `LinearVelocity`. All of it was verified against the avian source up front
  (does `integrate_positions` advance kinematic bodies by their velocity? do
  sensors emit `CollisionStart` with `CollisionEventsEnabled`? how do
  `CollisionLayers::new` filters combine?), which is why the ~1000-line example
  compiled first try with zero API errors -- the same "read the engine source"
  win as the dropzone cycle.

- **Shard placement.** Sliced fragments are wedges in the parent's local space,
  centered on the old rock origin. Spawned as-is they would spin about that
  off-center point like they were orbiting it. Each shard mesh is recentered on
  its own centroid (`mesh_centroid` + `Mesh::transformed_by`) and the body is
  placed at the shard's real world position, so it tumbles naturally in place.

## Possible follow-ups (deliberately left out)

- On-screen touch buttons for a pure keyboard-style control on mobile; the
  face-the-pointer scheme covers touch, so this was not needed.
- Ship-death shatter using `ExplodeMesh` on the ship model (the ship dies with a
  flash + shake instead, to avoid threading the child model's world transform
  through the asteroid-only fragment observer).
