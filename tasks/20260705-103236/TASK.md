# Build examples/14_breach -- grounded first-person arena shooter

- STATUS: CLOSED
- PRIORITY: 80
- TAGS: spike,feature,example,fps

## Goal

Add `examples/14_breach` (name tunable): a grounded, Doom-like first-person arena
shooter, and the gallery's first first-person game. It headlines the first-person
viewpoint as a real game, the crate's first avian `SpatialQuery` raycast (hitscan),
and a new game-local first-person character controller.

Core (net-new, since `camera/wasd` is free-fly only and there is no character
controller in the crate): a `FirstPersonController` with walk + gravity + ground
check + collide-and-slide against level colliders, and always-on yaw/pitch from a
grabbed mouse (`CursorGrabMode::Locked`, released on Escape/menu) with a pitch
clamp; the `Camera3d` rides at eye height. LMB fires a hitscan ray
(`SpatialQuery::cast_ray`) on a `time/cooldown`; first enemy hit takes
`HealthApplyDamage` + `feedback/flash`, dies via `mesh/explode`; muzzle flash
(`camera/post` bloom), tracer (`helpers/temp`), recoil (`camera/shake`), gunshot
`SfxPlugin`. Waves of pathing enemies (capsule/octahedron `Health` shapes) close in
and deal contact damage (spikes `feedback/screen_flash`); player `Health` zero ends
the run. Blocky arena (floor + walls + cover) as static avian colliders. Shape:
menu/playing/game-over, `ui/status` HUD (health/ammo/wave), `ui/menu`, `audio`,
`persist`+`HighScore` (best wave), `input/state`, wasm build. Touch: dual virtual
sticks + fire button via `ui/touchpad` (clunky but touch-enterable; desktop is
primary).

Ship the controller game-local; do NOT build a crate abstraction here (that is the
follow-up 20260705-103238).

## Steps

- [x] **Scaffold `examples/14_breach.rs`** (11_overload/08_dropzone shape): `//!` doc
  header + controls + `cargo run --example 14_breach`; clap `Cli` parsed first;
  `Window` with wasm `canvas`/`fit_canvas_to_parent`; `DefaultPlugins`;
  `PhysicsPlugins::default()`; keep default `Gravity` (-Y) OR set
  `Gravity(Vec3::new(0.0, -20.0, 0.0))`; `GameState { #[default] Menu, Playing,
  GameOver }` + `init_state`; `ClearColor`; a `#[derive(PhysicsLayer, Default, Clone,
  Copy)] enum GameLayer { #[default] Default, World, Player, Enemy }`; `Startup`
  `setup` (Camera setup is per-player below, but load `SoundBank<Sfx>` here and spawn
  a `DirectionalLight`/scene light); crate plugins: `HealthPlugin`, `SfxPlugin`,
  `FlashPlugin`, `ScreenFlashPlugin`, `CameraShakePlugin`, `PostProcessingDefaultPlugin`,
  `ExplodeMeshPlugin`, `TempEntityPlugin`, `StatusBarPlugin`, `TouchpadPlugin`,
  `PersistPlugin::<HighScore<u32>>::new("14_breach.high_score")`,
  `FrameTimeDiagnosticsPlugin` (guarded). Boots to Menu.
- [x] **Arena.** `OnEnter(Playing)` spawn floor (large `Cuboid`) + 4 perimeter walls +
  a few cover cuboids, each `RigidBody::Static` + `Collider::cuboid(..)` +
  `Mesh3d(meshes.add(Cuboid::new(..)))` + `MeshMaterial3d<StandardMaterial>` +
  `CollisionLayers::new([GameLayer::World], [GameLayer::Player, GameLayer::Enemy])`,
  tagged `DespawnOnExit(GameState::Playing)`.
- [x] **Player + FP controller.** Spawn the player: `RigidBody::Dynamic` +
  `Collider::capsule(r, h)` + `LockedAxes::new().lock_rotation()` (upright, solver does
  collide-and-slide -- kinematic does NOT auto-resolve vs statics) +
  `CollisionLayers::new([GameLayer::Player],[GameLayer::World, GameLayer::Enemy])` +
  `Visibility::default()` + `Health::new(100.0)` + a `FirstPersonController { yaw, pitch,
  speed }` + `Transform` at a spawn point, `DespawnOnExit(Playing)`. As a CHILD:
  `(Camera3d::default(), Transform::from_xyz(0.0, EYE_H, 0.0), PostProcessingCamera,
  AmbientLight { .. }, CameraShake { .. }, PlayerCamera)`. Body rotation stays identity;
  the camera child carries the full view rotation.
- [x] **Look + cursor grab.** `OnEnter(Playing)`: grab+hide cursor (`Single<&mut
  CursorOptions, With<PrimaryWindow>>`, `grab_mode = Locked`, fall back `Confined`,
  `visible=false`). A `Playing` system reads `Res<AccumulatedMouseMotion>.delta`, updates
  `controller.yaw -= dx*sens` and `controller.pitch` (clamp +/- ~1.54 rad), and writes the
  camera child local rotation `Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0)`. Release
  the cursor (`grab_mode=None`, `visible=true`) `OnExit(Playing)` / on GameOver / Escape.
- [x] **Movement.** `Playing` system: read WASD (+ touch move stick) into a `Vec2`
  intent; `world = Quat::from_rotation_y(controller.yaw) * Vec3::new(strafe, 0, -fwd)`;
  set player `LinearVelocity.x/.z = world.xz.normalize_or_zero() * speed`, PRESERVE
  `.y` (gravity). Factor the intent->world-vector math into a pure `fn move_dir(yaw, Vec2)
  -> Vec3` for unit testing.
- [x] **Gun (hitscan, the first `SpatialQuery` demo).** LMB / fire button gated by a
  `Cooldown`: `SpatialQuery::cast_ray(cam.translation(), Dir3::new(cam.forward())?,
  RANGE, true, &SpatialQueryFilter::from_mask([GameLayer::Enemy, GameLayer::World])
  .with_excluded_entities([player]))`. On an enemy hit -> `commands.trigger(
  HealthApplyDamage { entity, source: Some(player), amount })` + `Flash` on the enemy
  mesh child. Spawn a tracer (thin `Cuboid` from muzzle to `origin+dir*dist`, `TempEntity`)
  + muzzle flash (emissive bloom), kick `CameraShake` (`input.add_trauma += 0.25`), play
  a shoot `SfxPlugin` one-shot.
- [x] **Enemies + waves.** A `Wave { number, alive }` resource. Spawn `wave_size(n)`
  enemies on a ring at the arena edge (pure `fn wave_size(u32)->u32`, `fn
  ring_positions(count, radius)->Vec<Vec3>`): each `RigidBody::Dynamic` +
  `Collider::capsule` + `CollisionLayers([Enemy],[World,Player,Enemy])` + `Health` +
  an octahedron `Mesh3d` (via `TriangleMeshBuilder::new_octahedron`, emissive) child +
  an `Enemy { attack: Cooldown }` + `DespawnOnExit(Playing)`. A `Playing` system sets
  each enemy's horizontal `LinearVelocity` toward the player (speed ramps with wave). On
  `alive == 0` -> next wave.
- [x] **Contact damage.** `Playing` system: for each enemy within melee range of the
  player with its `attack` `Cooldown` ready -> `trigger(HealthApplyDamage { entity:
  player, amount })`, `attack.trigger()`, re-spike the damage vignette (`ScreenFlash`
  on the persistent overlay) + `CameraShake`. (Distance check avoids the
  `CollisionEventsEnabled` plumbing.)
- [x] **Enemy death.** Observe `On<Add, HealthZeroMarker>` (or poll) for enemies:
  `insert ExplodeMesh { fragment_count }` on the enemy, decrement `Wave.alive`,
  `Score += 1`, play a death `SfxPlugin` one-shot, then despawn the body a frame later
  (or convert to fragments). Add an `On<Insert, ExplodeFragments>` observer that spawns
  each fragment as a short-lived (`TempEntity`) dynamic body flying along
  `fragment.direction` (mirror `10_asteroids` `on_fragments_spawned`).
- [x] **HUD + crosshair + vignette.** `ui/status` items (HEALTH, WAVE, SCORE, FPS) via
  `status_bar`/`status_bar_item` reading the player `Health`/`Wave`/`Score`; a center
  crosshair `Node`; a persistent full-screen damage-vignette overlay (`screen_flash_node()`
  + `BackgroundColor(red.with_alpha(0))` + `despawn_on_end:false`), re-spiked on hit.
- [x] **Menu + game over.** `OnEnter(Menu)` overlay (`centered_screen` + `screen_text`
  + `TitlePulse`/hand-rolled pulse, best line); `menu_start` on click/key/touch -> Playing.
  `OnEnter(GameOver)`: release cursor, `record_high_score` (chained) then
  `spawn_game_over` (final score/wave, "New best!"), game-over sfx; `gameover_dismiss`
  -> Menu. `set_state_on_key(KeyCode::Escape, GameState::GameOver)` while Playing.
- [x] **Touch (charter compromise).** Dual virtual sticks + fire button via `ui/touchpad`:
  left half `stick_deflection` -> move intent, right half -> look delta, a fire button
  (`button_grid_at`) -> shoot; gate the on-screen controls with `RevealOnTouch`. Honestly
  clunky; desktop mouse+keyboard is primary. Keep it additive to the same input paths.
- [x] **Pure-logic unit tests.** `#[cfg(test)]` for `move_dir(yaw, input)` (forward at
  yaw 0 is -Z; strafe is +X; yaw rotates correctly), `wave_size` (monotonic ramp),
  `ring_positions` (count, radius, even spacing), and pitch clamping. Per the crate
  convention + the `13_glide` lesson (test logic-that-drives-behavior, not screenshots).
- [x] **Harness wiring** (`#[cfg(feature="debug")]`): `InspectorDebugPlugin`;
  `AutopilotPlugin::new().hold(Menu,0.6).hold(Playing,4.0).hold(GameOver,0.8).input(|world,
  _| { if Playing { press W + LMB } })` (enemies converge and kill the player -> GameOver);
  `ScreenshotPlugin::new(Playing).settle_frames(30)`. Ensure cursor-grab does not wedge the
  headless autopilot run (guard the grab if needed).
- [x] **Verify (full gate).** `cargo fmt`, `cargo clippy --all-targets` + `--features
  debug` (clean), `cargo test --example 14_breach` (pure-logic tests), `cargo test
  --examples`, `scripts/check-ascii.sh`. RUN it under `timeout` to confirm it reaches the
  render loop (physics example -- dropzone startup-hang lesson; a swap-chain log line means
  startup finished). `BCS_AUTOPILOT=1 ... --features debug` -> confirm Menu->Playing->
  GameOver + `cycle complete, no panic` + no runtime errors. `BCS_SHOT=390x844` grab.
- [x] **Wasm/web registration.** Append `"14_breach web/games/14_breach"` to
  `web/scripts/build-games.sh`; add `web/games/14_breach/index.html` (copy 09_reactor's,
  retitle, keep `#game-canvas` + audio-unlock + `copy-dir` sounds); add a `14_breach` entry
  to `GAMES` in `web/src/games.ts` (blurb should note desktop-first controls). Build via
  `trunk build --example 14_breach` to confirm the wasm compile.
- [x] **Docs.** Add `tasks/20260705-103236/NOTES.md` (design: dynamic-capsule +
  lock_rotation controller and why not kinematic; the SpatialQuery hitscan; cursor grab;
  the touch compromise). Update `AGENTS.md` example list + module map: first FPS, first
  `SpatialQuery` use, game-local FP controller; note the harvest follow-up 20260705-103238.

## Notes

Spike: tasks/20260705-103116/SPIKE.md

Stepless direction-level task -- run `/plan` before `/work`. Key facts from the
spike survey (so the implementer does not re-derive):

- `camera/wasd` is a free-fly spectator camera (accumulates `WASDCameraInput.pan`
  into yaw/pitch, integrates `wasd`/`vertical` straight into `Transform`, no
  gravity/ground/collision/pitch-clamp/smoothing) and `helpers/wasd` gates look to
  RMB-drag with NO cursor grab. It owns the whole `Transform`, so it fights a
  physics controller -- do NOT use it for the FPS; build the controller.
- No `SpatialQuery`/`RayCaster`/raycast anywhere in the crate or examples -- hitscan
  is net-new against avian's `SpatialQuery` (this is a headline: first raycast demo).
- No kinematic character controller / collide-and-slide anywhere; `physics/` is just
  `pd_controller`. avian usage to copy: `10_asteroids` kinematic bodies +
  `LinearVelocity`, `08_dropzone` dynamic body + static colliders + trimesh terrain.
- Reuse: `feedback/flash` (enemy hit-flash, needs `MeshMaterial3d<StandardMaterial>`;
  it clones the material), `feedback/screen_flash` (damage vignette -- spawn a
  persistent `despawn_on_end:false` overlay and re-insert `ScreenFlash` per hit),
  `ui/touchpad` (`stick_deflection` for virtual sticks, `button_grid_at` for the
  fire button, `RevealOnTouch` to gate touch UI), `camera/shake`, `camera/post`.
- Open unknowns to settle at impl (see spike): wasm pointer-lock flow, the
  collide-and-slide approach (kinematic shapecast vs depenetration), and how much
  touch to ship.
- Copy the Bevy 0.19 visual/HUD idioms from `11_overload`/`08_dropzone`, not memory.
  Verify with the `AutopilotPlugin`/`ScreenshotPlugin` harness; a mid-game invariant
  is better asserted as a pure-function unit test than a screenshot (state-entry
  screenshot != gameplay verification -- see the `13_glide` retro/gotcha).

### API facts (verified against avian 0.7 / bevy 0.19 / src)

- **Controller decision (resolved):** player = `RigidBody::Dynamic` +
  `Collider::capsule` + `LockedAxes::new().lock_rotation()`, driven by writing
  `LinearVelocity` (xz from input, y left to gravity). avian's solver then does
  collide-and-slide against static level colliders for free. A KINEMATIC body does
  NOT auto-resolve vs statics (asteroids reflects velocity by hand), so do not use
  kinematic. Body rotation stays identity; yaw lives on the controller + camera child.
- **Raycast:** `SpatialQuery` is a `SystemParam` (take by value).
  `cast_ray(origin: Vec3, direction: Dir3, max_distance: f32, solid: bool, filter:
  &SpatialQueryFilter) -> Option<RayHitData>`. `RayHitData { entity, distance, normal }`
  (no hit point -- compute `origin + dir*distance`). `direction` MUST be a `Dir3`
  (`Dir3::new(gt.forward().as_vec3())` or `gt.forward()` is already `Dir3` in 0.19).
  `SpatialQueryFilter::from_mask([GameLayer::Enemy, GameLayer::World])
  .with_excluded_entities([player])`; mask tests a collider's `CollisionLayers`
  memberships. There is also `cast_ray_predicate(.., &|e| bool)` to skip the player.
- **avian setup:** `PhysicsPlugins::default()`; `Gravity(Vec3)` resource.
  `#[derive(PhysicsLayer, Default, Clone, Copy)] enum GameLayer {..}`;
  `CollisionLayers::new([memberships], [filters])`. Collisions (only if needed):
  `MessageReader<CollisionStart>` (singular, `.collider1/.collider2`), and the bodies
  need a `CollisionEventsEnabled` component -- prefer the distance-based melee check to
  avoid this.
- **Cursor grab (bevy 0.19):** `CursorOptions` is a SEPARATE component on the primary
  window entity (not `window.cursor`). `Single<&mut CursorOptions, With<PrimaryWindow>>`;
  set `grab_mode = CursorGrabMode::Locked` (fall back `Confined` on X11), `visible=false`.
  Look delta: `Res<AccumulatedMouseMotion>.delta` (already summed per frame).
- **feedback/flash:** `Flash { color, duration, channel: FlashChannel::{Emissive,
  BaseColor} }`; insert on the entity that owns `MeshMaterial3d<StandardMaterial>` (the
  enemy MESH child, not a bare parent) -- no-material Flash is silently dropped. Add
  `FlashPlugin`.
- **feedback/screen_flash:** `screen_flash_node() -> Node`, `ScreenFlash { peak_alpha,
  decay, despawn_on_end }` (fade time `1/decay`). Persistent vignette: spawn once with
  `despawn_on_end:false` + `BackgroundColor(red.with_alpha(0))`, re-insert `ScreenFlash`
  per hit. Add `ScreenFlashPlugin`.
- **camera/shake:** `CameraShake { decay, max_offset, max_kick, exponent }` on the
  camera (requires Transform etc.); kick via `Single<&mut CameraShakeInput>` ->
  `input.add_trauma += 0.25`. Add `CameraShakePlugin`.
- **camera/post:** add `PostProcessingDefaultPlugin` + the `PostProcessingCamera` marker
  on the `Camera3d`; an observer adds Tonemapping + Bloom. Do NOT set Bloom yourself.
- **mesh/explode:** `insert ExplodeMesh { fragment_count }` (entity needs `Mesh3d` +
  `MeshMaterial3d<StandardMaterial>`); an observer produces `ExplodeFragments(Vec<{origin,
  mesh, direction: Dir3}>)`. Spawn gibs with your own `On<Insert, ExplodeFragments>`
  observer (mirror `10_asteroids::on_fragments_spawned`). Add `ExplodeMeshPlugin`.
- **meshes:** `TriangleMeshBuilder` has `new_octahedron(resolution)` / `new_cone` only --
  NO cuboid/capsule. Use bevy primitives `meshes.add(Cuboid::new(..))`,
  `Capsule3d::new(r, len)`, `Sphere::new(r)`; use `new_octahedron` for a sliceable enemy.
- **camera child + light:** `AmbientLight` is a per-camera COMPONENT in 0.19 (put it on
  the camera). Spawn the camera as a child: `commands.entity(player).with_children(|p|
  p.spawn((...)))`. A parent with no mesh needs explicit `Visibility` (B0004).
- **entry points:** `HealthApplyDamage { entity, source, amount }` via
  `commands.trigger(..)` (auto-propagates, adds `HealthZeroMarker`); `Cooldown::new(d)`
  `.tick(dt)/.ready()/.trigger()`; `TempEntity(secs)`; `set_state_on_key(key, target)`;
  `status_bar`/`status_bar_item`/`status_bar_with_fps`; `centered_screen`/`screen_text`;
  `SoundBank::load`/`.get`, `commands.play_sfx_volume(handle, vol)`.

### Open decisions (resolve at impl, do not guess silently)

- **Cursor grab under the headless autopilot** -- `CursorGrabMode::Locked` on X11 falls
  back to Confined; confirm the `BCS_AUTOPILOT` run is not wedged by the grab (guard the
  grab off when autopilot drives, if needed).
- **Enemy shove** -- both player and enemies are dynamic, so a mob can push the player.
  Acceptable (FPS feel); tune enemy mass/speed if it is too strong.
- **Touch depth** -- ship a best-effort dual-stick scheme; do not over-invest. The
  gallery blurb should say desktop-first.

### Work log (implementation)

- Implemented `examples/14_breach.rs` (~900 lines): dynamic-capsule FP controller
  (lock_rotation + LinearVelocity so the solver does collide-and-slide), grabbed-
  cursor always-on look with pitch clamp, hitscan gun via `SpatialQuery::cast_ray`
  (the crate's first raycast), ring-spawned pathing enemies with contact melee,
  `mesh/explode` gibs, HUD/crosshair/damage-vignette, menu/game-over, dual-stick
  touch, 9 pure-logic unit tests.
- Controller decision: dynamic + `LockedAxes::ROTATION_LOCKED` (kinematic does not
  auto-resolve vs statics). Documented in `tasks/20260705-103236/NOTES.md`.
- Bugs fixed: (1) tracer/flash-vs-despawn race -- insert `Flash` BEFORE the damage
  trigger, since a lethal hit despawns the enemy in the same flush; (2) headless FPS
  verification -- the autopilot input closure AIMS at the nearest enemy (sets
  controller yaw) then fires, since fire-forward never hits and the look system can't
  take injected mouse motion. Spawned the player at arena centre.
- Verified: `cargo clippy --all-targets` clean (plain + `--features debug`); 9 unit
  tests pass; `check-ascii` + `cargo fmt --check` clean; headless `BCS_AUTOPILOT` run
  reaches Menu->Playing->GameOver with "cycle complete, no panic" and zero runtime
  errors, and the persisted best went to 2 kills (proves the raycast->damage->death->
  score->persist path); `BCS_SHOT` app-native screenshot confirms the 3D scene, HUD
  and crosshair render. Web build registered; single-game `trunk build --example
  14_breach` for the wasm compile.
- Known: difficulty is high (standing still you die fast; kiting expected); touch is
  a best-effort compromise (desktop primary).
