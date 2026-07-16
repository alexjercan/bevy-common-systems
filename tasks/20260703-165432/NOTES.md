# 08_dropzone: landing a ship on the noise planet

Date: 2026-07-03
Task: `tasks/20260703-165432` (08_dropzone)

## What this example is

`examples/08_dropzone.rs` is a small lunar-lander game and the crate's headline
demo of `physics/pd_controller`. A noise-displaced planet (the `02_planet`
recipe) sits at the origin with its own radial gravity. You fly a ship down onto
it: thrust counteracts gravity, and the PD controller rotates the avian3d rigid
body toward whatever attitude you steer to. Land slow and upright to score; hit
too hard or too tilted and the hull breaks apart via `mesh/explode`.

It is the first example that runs a real avian simulation (the earlier ones add
`PhysicsPlugins` only for the debug renderer), and it pulls several crate pieces
together at once: `pd_controller`, `mesh/builder` (+ a trimesh collider),
`mesh/explode`, `helpers/temp`, `camera/skybox`, `camera/post`, `camera/chase`,
`ui/status` and `audio`. It follows the `06_fruitninja` shape: Bevy states
(Menu / Playing / Result), a wasm-friendly window, and the shared placeholder
sounds.

## Key decisions

### Applying the PD controller output to avian (the missing glue)

`PDControllerPlugin` only *computes* a torque into `PDControllerOutput`; nothing
in the crate applied it to a body, and no example did either. avian 0.7 replaced
the old `ExternalForce`/`ExternalTorque` API with persistent, per-body force
components. The example uses three of them and overwrites them every
`FixedUpdate`:

- `ConstantTorque` <- `PDControllerOutput` (attitude control).
- `ConstantLocalLinearAcceleration` along local +Y for thrust.
- `ConstantLinearAcceleration` = `-radial_up * GRAVITY` for planet gravity.

Ordering matters: `set_attitude_target` writes `PDControllerInput` *before*
`PDControllerSystems::Sync`, and `apply_ship_forces` reads `PDControllerOutput`
*after* it, both scheduled in `FixedUpdate` around the plugin's system set.

Gravity and thrust use *acceleration* components (mass-independent) rather than
forces, so the flight feel does not depend on the collider's derived mass and is
much easier to tune. The controller torque is a real torque because the PD math
already scales by the body's inertia tensor.

### Radial gravity, upright-relative attitude

Global gravity is set to `Gravity(Vec3::ZERO)`; each frame the ship's gravity is
recomputed toward the planet centre from its own position. The target attitude
is `from_rotation_arc(Vec3::Y, radial_up)` (upright relative to the surface)
times a small player lean, so releasing the steering keys self-levels the ship.
The ship starts near the +Y pole and only needs to operate over a small patch,
so the `from_rotation_arc` singularity at the antipode never comes up.

### Trimesh collider from `TriangleMeshBuilder`

The planet collider is built inline from the builder's triangles:
`vertices_and_indices()` -> `Collider::trimesh(verts, [u32;3] triples)`. The task
suggested extracting a `collider-from-TriangleMeshBuilder` helper "if a second
game needs it too". Only this game needs it, so it stays inline to keep the
library minimal and single-concern; lift it into `mesh/builder` when a second
consumer appears.

### Procedural starfield skybox (no binary asset)

`SkyboxConfig` needs a stacked cubemap `Image` (6 square faces, height = 6x
width). Rather than commit a PNG, the example builds that image in code
(`starfield_cubemap`): a near-black sky with scattered stars, added to
`Assets<Image>` before the camera is spawned so the plugin's insert observer can
reinterpret it. This keeps the repo asset-free for the skybox and works on wasm
unchanged.

### Crash effect reuses the 05_explode pattern

On a hard impact the ship gets `ExplodeMesh`; an `On<Insert, ExplodeFragments>`
observer turns each slice into an independent flying fragment with
`FragmentMotion` + `TempEntity`, integrated by a plain system with radial
gravity (decoupled from avian, exactly like `05_explode`). The hull shell
despawns with the `Playing` state.

### Sounds and web

Reuses the existing placeholder sounds: `launch` on start, `golden` on a good
landing, `bomb` on a crash. Registered for the web showcase like `06_fruitninja`
(`web/games/08_dropzone/index.html` with the shared Safari audio-unlock shim and
the `sounds` copy-dir, plus entries in `web/scripts/build-games.sh` and
`web/src/games.ts`). No texture asset is copied because the skybox is generated
at runtime.

## Physics tuning notes / risk

The task flagged this as higher physics-tuning risk. It has since been
play-tested (`tasks/20260703-213510`) by running the example under a scripted
autopilot that logged telemetry, and the physics constants held up:

- PD response is crisp: a commanded full lean rises to target in ~1 s with no
  overshoot and returns to upright in ~0.6 s. The `max_torque` clamp (4000) is
  never reached, so `frequency 2.2 / damping 1.0` was kept.
- Gravity 5.5 vs thrust 13 gives a winnable bang-bang descent: from the start
  altitude the autopilot lands at ~1-4 m/s with ~40% fuel to spare, so the fuel
  budget (100 / burn 14) is comfortable, not starved.
- High-speed impact does NOT tunnel: a free-fall crash (~14 m/s) is caught by
  the trimesh and correctly classified, so no `SweptCcd` was needed.
- The game ends on first contact, so long-term resting stability on the bumpy
  trimesh is moot (there is no post-touchdown sim to jitter).

Two things did change during tuning (see
`tasks/20260703-213510/RETRO.md`):

- The chase camera was reframed (lower, pitched down) so the planet surface and
  the ship's glowing thruster stay in view during the approach; before, the
  camera looked out into empty space at altitude and the thruster was hidden
  under the hull, so the `camera/post` bloom never showed.
- Impact speed is now captured pre-solve (`ApproachSpeed`, updated once per
  render frame in `PreUpdate`, before the fixed-physics loop). Reading the live
  `LinearVelocity` in `resolve_landing` under-reported the hit because avian's
  `FixedPostUpdate` solver had already absorbed it - a hard crash could read
  below the 4.5 m/s limit and be scored as a soft landing. `PreUpdate` (rather
  than `FixedUpdate`) matters because a stuttering frame runs several physics
  substeps; capturing before the whole loop means no collision substep can
  overwrite the value with a post-impact reading.
