# build examples/12_bastion -- defend-the-core tower defense (camera/project + rotation modules)

- STATUS: CLOSED
- PRIORITY: 2
- TAGS: spike,examples,game

> Spike: tasks/20260704-220530/SPIKE.md (read first,
> the revised Recommendation). This merges the original 12_warden + 13_turret
> ideas into one game per the user's steer.

## Goal

A small (~2000 line, target ceiling) unconventional tower-defense prototype that
closes ALL THREE never-demoed modules in one game: `camera/project`,
`transform/smooth_look_rotation`, and `transform/point_rotation`.

Concept: a **Core** (a `Health` pool -- the thing you defend) sits at the center
of a circular play plane; **enemies spawn all around the border and converge
inward** from every bearing. Kills earn credits; spend them to **place towers**
on the plane around the Core and to **upgrade** them. An enemy reaching the Core
damages it (`HealthApplyDamage`); Core health zero ends the run. Waves ramp
count/speed/HP.

Module homes (the point of the example):

- **`camera/project` (headline).** `pointer_on_plane` maps mouse/tap to the
  world placement point (ghost tower + range ring there); `world_to_screen`
  anchors floating UI -- enemy health pips, "+N" credit popups on kill, the
  click-to-upgrade panel over a selected tower. First validated user of the
  harvested module (no example imports it today).
- **`transform/smooth_look_rotation` (tower turrets).** Towers auto-target the
  nearest enemy in range; the turret rotates toward it with `SmoothLookRotation`
  -- rate-limited, so a fast enemy can out-slew a cheap turret until upgraded.
  Turn-rate is a real tunable stat.
- **`transform/point_rotation` (orbit camera).** Camera rides a rig aimed at the
  Core; mouse-drag / A/D feed `PointRotationInput` to accumulate yaw/pitch and
  orbit the view around the battlefield.

Keep the first cut simple (the brief): 2-3 tower archetypes, a one-axis upgrade
per tower, 1-2 enemy types, hand-authored waves. Reuse the established shape and
juice kit: menu/playing/game-over `States`, `SfxPlugin` one-shots, `ui/status`
HUD (credits/wave/Core integrity), `mesh/explode` on kills, `ui/popup`,
`camera/shake`, `feedback`, `tween`, `scoring/streak`, `time/cooldown` (per-tower
fire cadence), `input/pointer`, wasm/trunk build. Touch-native: tap-to-place,
drag-to-orbit.

Build the tower/enemy stats as a **game-local serde catalog** (TowerSpec /
EnemySpec spawned by name) so the modding follow-up (tasks/20260704-220719) can
build on it -- but do NOT block this task on any crate-level abstraction.

Open at implementation (from the spike): camera angle (angled perspective vs
top-down; pick with `ScreenshotPlugin`); clamp the orbit pitch in-game
(`point_rotation` has no min/max -- consider harvesting one back); keep placement
usable at grazing angles (constrain pitch or snap to a ring/grid);
nearest-in-range auto-target is the simple default. Watch the line budget -- this
game is more mechanic than any single earlier example, so lean on the juice kit
and keep variety minimal.

## Steps

Build incrementally; after each milestone run the real compile gate
`cargo clippy --all-targets` (NOT bare `cargo build` -- it skips examples).
Everything below is one file, `examples/12_bastion.rs`, unless noted.

- [x] **Scaffold + boot.** Create `examples/12_bastion.rs`: module `//!` doc
  (mirror `10_asteroids.rs:1-36`), `use` block, `#[derive(Parser)] struct Cli`
  with `#[command(name = "12_bastion")]`, a tunable `const` block, and `main()`
  with `DefaultPlugins`+wasm-canvas `WindowPlugin`, `PhysicsPlugins::default()`
  (needed by the debug gizmos even if unused) + `Gravity(Vec3::ZERO)`,
  `ClearColor`, `#[cfg(feature="debug")] InspectorDebugPlugin`,
  `FrameTimeDiagnosticsPlugin`, the crate plugins (see Notes), `init_state`,
  resource inits, `Startup(setup)`, and `run()`. Add a stub `GameState`
  {Menu,Playing,GameOver} and empty `setup`. Confirm it compiles and boots.
- [x] **Persistent scene + orbit camera rig.** In `setup`: spawn a camera *rig*
  parent entity carrying `PointRotation` (identity), and a child `Camera3d` at a
  fixed local offset+pitch looking back at the rig origin (the Core), with
  `PostProcessingCamera`, `CameraShake`, and an `AmbientLight` (per-camera in
  0.19). Spawn a directional light, a ground disk mesh for the play plane
  centered at origin, and the central **Core** entity (`Health::new`, a mesh +
  emissive material). Add the FPS `status_bar` overlay (copy the
  `status_bar`/`status_bar_item` block in Notes). Verify with a `BCS_SHOT` grab.
- [x] **Orbit control via `point_rotation`.** System (runs in Menu+Playing):
  read `UnifiedPointer` drag delta and/or A/D keys, write `PointRotationInput`
  on the rig (x=yaw, y=pitch); clamp accumulated pitch in-game (point_rotation
  has no min/max -- clamp the applied `PointRotationOutput` euler, or gate the
  input at limits). Apply `PointRotationOutput` (a `Quat`) to the rig
  `Transform.rotation`. Confirm dragging orbits the view around the Core and the
  ground stays pickable (see grazing-angle note).
- [x] **Data-local spec catalog.** Define game-local `TowerSpec` and `EnemySpec`
  structs (derive `serde::Deserialize` + `Clone`) holding stats: for towers
  `name, cost, range, fire_rate, damage, turn_speed, upgrade_cost`; for enemies
  `name, hp, speed, damage_to_core, reward, radius`. Build a couple of each as
  `const`/`fn` tables and a `spawn_tower_by_name` / `spawn_enemy_by_name` that
  reads a spec and spawns the entity. Keep the JSON-loading path OUT of scope
  (that is the follow-up task 20260704-220719); just structure it so the loader
  can slot in later.
- [x] **Enemy waves + convergence + Core damage.** A `Wave` resource and a
  spawner: each wave spawns N enemies at random points on the arena border
  (angle around the Core, at arena radius). Movement system: each enemy steps
  toward the Core (`(core_pos - pos).normalize() * speed * dt`). On reaching the
  Core radius, `commands.trigger(HealthApplyDamage { entity: core, source:
  Some(enemy), amount: spec.damage_to_core })` and despawn the enemy. Clearing
  all enemies advances the wave (ramp count/speed/hp). Register an
  `On<Add, HealthZeroMarker>` observer that, when the Core dies while Playing,
  starts the death beat (see lifecycle note) -> GameOver.
- [x] **Tower placement via `camera/project` (the headline).** A placement mode:
  while active, each frame call `pointer_on_plane(camera, cam_gt,
  pointer.screen_pos?, Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))` to get the
  world point; show a translucent ghost tower + a range-ring gizmo there. On
  `pointer.just_pressed`, if credits >= cost and the spot is valid (min spacing
  from Core/other towers), spend credits and `spawn_tower_by_name`. Towers are
  a body + a **turret child** entity carrying `SmoothLookRotation { axis: Y,
  speed: spec.turn_speed, .. }`.
- [x] **Tower targeting + firing via `smooth_look_rotation` + `Cooldown`.**
  Per-tower system: find nearest enemy within `range`; compute the yaw angle to
  it and write it to the turret's `SmoothLookRotationTarget`; apply
  `SmoothLookRotationOutput` as `Quat::from_axis_angle(Y, out)` to the turret
  transform. Gate firing on a per-tower `Cooldown` (`tick(dt)`, `if ready()` and
  turret roughly aimed -> `trigger()` + deal damage). Firing deals
  `HealthApplyDamage` to the target enemy (hitscan for the first cut; a visible
  tracer/projectile is optional polish). Enemy `HealthZeroMarker` observer:
  insert `ExplodeMesh { fragment_count }`, award credits + `Streak::hit`, spawn a
  "+N" `popup` at `world_to_screen(enemy_pos)`, `play_sfx`, add camera-shake
  trauma, and (own `On<Insert, ExplodeFragments>` observer) spawn the shards as
  short-lived `TempEntity` meshes that fly out (mesh/explode has NO built-in
  fragment observer -- see Notes).
- [x] **Tower selection + one-axis upgrade.** Tapping an existing tower (pointer
  pick within radius) selects it; anchor a small upgrade panel via
  `world_to_screen(tower_pos)` showing the cost. A confirm (button/key) spends
  credits and bumps one stat (e.g. `damage *= 1.5` or `range += k`). Keep it to
  a single upgrade axis per the brief.
- [x] **HUD via `ui/status`.** Add `status_bar_item`s for Credits, Wave, and
  Core integrity (a `value_fn` reading each resource/`Health`, a `color_fn`
  shading Core integrity green->amber->red). FPS item already added in setup.
- [x] **Menu + game-over states.** `spawn_menu` (title + "tap to play" + best
  score, using the `centered_screen`/`screen_text` helpers), `pulse_menu_title`,
  `menu_click` (advance on `UnifiedPointer.just_pressed` or Space),
  `spawn_game_over` (score + wave reached + "New best!"), `gameover_click`,
  `record_high_score`, `play_game_over_sfx`, `giveup_on_escape`. Load `SfxAssets`
  in `setup` (reuse existing wavs: menu_select, launch/shot, bomb/explode, hurt,
  level_up/wave_clear, game_over).
- [x] **Juice pass.** Screen flash on Core damage and on death (`screen_flash`
  scoped with `DespawnOnExit(GameState::Playing)`); entity `Flash` on an enemy
  when hit; camera-shake trauma on kills/Core-hit/death; a "WAVE N"/streak
  banner; per-event sfx with pitch variation. Do not over-build -- keep the line
  budget in mind.
- [x] **Touch.** Verify tap-to-place and drag-to-orbit work through
  `UnifiedPointer` on a narrow (`BCS_SHOT=390x844`) frame; if placement vs orbit
  gesture conflict, add a small mode toggle button (build/orbit) rather than
  overloading the same drag. Keep menu keys distinct from gameplay keys.
- [x] **Harness.** Add the `#[cfg(feature="debug")]` block with
  `AutopilotPlugin::new().hold(Menu,0.6).hold(Playing,~4.0).hold(GameOver,0.8)
  .input(|world, elapsed| { guard to Playing; synthesize placing a tower +
  orbiting })` and `ScreenshotPlugin::new(GameState::Playing).settle_frames(30)`
  (copy the shape from `11_overload.rs:156-192`).
- [x] **Web/wasm registration (3 places).** (a) Create
  `web/games/12_bastion/index.html` by copying `web/games/10_asteroids/index.html`
  and changing the `<title>`, the example-name comment, and keeping the
  `audio-unlock`, `rust`, and `assets/sounds` copy-dir trunk links. (b) Append
  `"12_bastion web/games/12_bastion"` to the `games=(...)` array in
  `web/scripts/build-games.sh`. (c) Append a `{ id: '12_bastion', title:
  'Bastion', blurb, controls, accent }` entry to `web/src/games.ts`. Run
  `npm ci` then `npm run build` in `web/` to verify (fresh worktree has no
  node_modules).
- [x] **Tests.** Add `#[cfg(test)] mod tests` covering the pure TD math (e.g.
  border-spawn point on the arena circle, nearest-enemy-in-range selection, the
  yaw-to-target angle, upgrade cost/stat math). Mirror the in-example test style
  of `10_asteroids`.
- [x] **Docs + module map.** Write `tasks/20260704-220736/NOTES.md`
  (concept, which modules it demos and why, decisions: camera angle,
  auto-target rule, pitch clamp, the game-local spec catalog and its relation to
  the modding follow-up). Add the `12_bastion` bullet to the example list in
  `AGENTS.md`.
- [x] **Verify (full gate).** `cargo fmt`, `cargo clippy --all-targets`,
  `cargo clippy --all-targets --features debug`, `cargo test --examples`,
  `./scripts/check-ascii.sh`, boot once under `timeout` (confirm it reaches the
  render loop), and run `BCS_AUTOPILOT=1 ... --features debug` under `timeout`
  to confirm the autopilot cycle completes with `no panic`.

## Notes

Relevant files: `examples/10_asteroids.rs` (closest template -- top-down 3D,
health, explode, shake, UnifiedPointer, fragment observer, states, sfx),
`examples/07_orbit.rs` (cleaner death observer + `advance_pressed`),
`examples/11_overload.rs` (the AutopilotPlugin/ScreenshotPlugin harness block to
copy). Examples are auto-discovered -- NO `Cargo.toml [[example]]` entry needed.
`clap` and bevy `wav` are already dev-enabled.

Crate plugins to add (all in `bevy_common_systems::prelude`): `ExplodeMeshPlugin,
PostProcessingDefaultPlugin, CameraShakePlugin, FlashPlugin, ScreenFlashPlugin,
StatusBarPlugin, HealthPlugin, SfxPlugin, UnifiedPointerPlugin, PopupPlugin,
PointRotationPlugin, SmoothLookRotationPlugin`. (`Cooldown` and `Streak` are
plain value types -- no plugin; tick them yourself. `pointer_on_plane`/
`world_to_screen` are free functions -- no plugin.)

Exact API facts gathered (so you do not re-read the crate):

- `pointer_on_plane(camera: &Camera, cam_gt: &GlobalTransform, viewport_pos:
  Vec2, plane_origin: Vec3, plane: InfinitePlane3d) -> Option<Vec3>`.
  `world_to_screen(camera, cam_gt, world_pos: Vec3) -> Option<Vec2>` (None if
  off-screen/behind).
- `PointRotation { initial_rotation: Quat }`; you insert only it, the plugin
  auto-adds `PointRotationInput(Vec2)` (write, x=yaw y=pitch radians delta) and
  `PointRotationOutput(Quat)` (read). Updates in `PostUpdate`. NO pitch clamp --
  clamp in-game.
- `SmoothLookRotation { axis: Vec3, initial: f32, speed: f32(rad/s), min:
  Option<f32>, max: Option<f32> }`; auto-adds `SmoothLookRotationTarget(f32)`
  (write angle) and `SmoothLookRotationOutput(f32)` (read). Wraps to shortest
  path, clamps to min/max.
- `Health { current, max }` / `Health::new(max)`; `HealthApplyDamage { entity,
  source: Option<Entity>, amount }` is an auto-propagating EntityEvent -- trigger
  via `commands.trigger(...)`; `HealthZeroMarker` inserted at zero (react with
  `On<Add, HealthZeroMarker>`).
- `ExplodeMesh { fragment_count: usize }`; the plugin inserts
  `ExplodeFragments(Vec<ExplodeFragment{origin,mesh,direction:Dir3}>)`. There is
  NO `on_fragments_spawned` observer in the crate -- 10_asteroids defines its own
  `On<Insert, ExplodeFragments>` observer (`examples/10_asteroids.rs:1209`);
  mirror that.
- `Cooldown::new(dur)` (starts ready) / `.tick(dt)` / `.ready()` / `.trigger()`
  / `.fraction()`. `Streak::new(window)` / `.hit()->count` / `.tick(dt)->Option`
  / `.reset()`.
- `popup(position: Vec2, text, font_size: f32, color) -> impl Bundle` -- position
  is a SCREEN/viewport point, so pair it with `world_to_screen`. `commands.spawn(
  popup(screen, "+10", 28.0, col))`.
- `UnifiedPointer` resource fields: `screen_pos: Option<Vec2>` (viewport px, NOT
  world), `pressed: bool`, `just_pressed: bool`. Add `UnifiedPointerPlugin`.
- `CameraShake` on the camera; write `CameraShakeInput.add_trauma += x`. If a
  system writes the camera base every frame it must run
  `.after(CameraShakeSystems::Restore).before(CameraShakeSystems::Apply)`; a
  static/orbit-rig base is fine as-is (the rig is the base, shake rides on the
  child camera's own transform).
- `screen_flash(color, peak_alpha, decay) -> impl Bundle`; `Flash { color,
  duration, channel }` inserted on a `MeshMaterial3d<StandardMaterial>` entity.
- SFX: `commands.play_sfx(h)`, `commands.play_sfx_volume(h, v)`, or
  `commands.trigger(PlaySfx::new(h).with_volume(v).with_speed(pitch))`.
- Status bar: `commands.spawn((status_bar(StatusBarRootConfig::default()),));`
  then per item `commands.spawn((status_bar_item(StatusBarItemConfig { icon:
  None, value_fn, color_fn, prefix, suffix }),));` -- `value_fn: Fn(&World)->
  Option<Arc<dyn StatusValue>>`, `color_fn: Fn(Box<&dyn Any>)->Option<Color>`.

Lifecycle idioms: state entities carry `DespawnOnExit(GameState::X)` (no manual
OnExit cleanup). Death uses a `DyingTimer{remaining: Option<f32>}` beat before
`next.set(GameOver)` (`10_asteroids.rs:1344`). Menu/game-over advance on
`UnifiedPointer.just_pressed || keys.just_pressed(Space)`.

Bevy 0.19 UI gotchas (copy, do not improvise): `TextFont.font_size =
FontSize::Px(..)`, `TextLayout { justify: Justify::Center, ..default() }`,
`AmbientLight` is per-camera, `Node { border_radius: BorderRadius::MAX, .. }`.
An HDR-emissive material must NOT set `unlit: true` (kills bloom). An entity with
child meshes but no own mesh needs an explicit `Visibility`.

Open questions (decide at implementation, non-blocking): camera angle
(perspective vs steeper top-down -- pick via `BCS_SHOT`); how the drag gesture
splits between orbit and placement (mode toggle if they conflict);
nearest-in-range is the default auto-target (swap to most-progressed if it feels
bad). Watch the ~2000-line ceiling: this game has more mechanics than any single
earlier example, so keep tower/enemy variety minimal and lean on the juice kit.

Assumption: the data-driven JSON loader and any `SpecCatalog<T>` crate module are
explicitly OUT of scope here -- they are the follow-up task 20260704-220719,
which depends on this one shipping.

## Outcome (closed)

Shipped `examples/12_bastion.rs` (~950 lines), a working defend-the-core tower
defense that closes all three never-demoed modules: `camera/project`
(`pointer_on_plane` placement + `world_to_screen` "+N" popups),
`transform/point_rotation` (orbit camera on a pivot at the Core) and
`transform/smooth_look_rotation` (rate-limited tower turrets). Plus the juice kit,
a game-local `TowerSpec`/`EnemySpec` catalog, sounds, HUD, and a registered wasm
web build.

Verified: `cargo clippy --all-targets` and `--features debug` clean; `cargo fmt
--check` clean; `cargo test --example 12_bastion` 5/5 pass; `check-ascii` clean;
`trunk build --release --example 12_bastion` succeeds; web `eslint` clean; boots
to the render loop and `BCS_AUTOPILOT=1 --features debug` completes menu ->
playing -> game-over with `no panic`, and a live `scrot` during autopilot
confirmed the full loop (towers placed, enemies killed, Core damaged, score
rising).

Deviations from the plan, deliberately:

- The spec catalog uses plain structs indexed by position (`tower_specs()` /
  `enemy_specs()` + `spawn_tower`/`spawn_enemy` by index), NOT `serde::Deserialize`
  + spawn-by-name. serde is unneeded for the game-local version and the JSON path
  is explicitly the follow-up task's job; the structs are still shaped so the
  loader can slot in.
- Credits / Wave / Core-integrity ride a single in-game HUD `Text` node rather
  than separate `status_bar_item`s; the status bar carries the FPS overlay. Same
  information, less UI churn, and the color-coded `color_fn` idea is folded into
  the HUD string. The `ui/status` module is still demoed (FPS item).
- Tap-vs-drag on one pointer is disambiguated by a move threshold (drag = orbit,
  clean release without moving = place/select), which is the plan's preferred
  "avoid the conflict" resolution -- no separate mode-toggle button was needed.
- Enemy hit-flash (`feedback/flash` on enemies) was dropped to stay within the
  line budget; `feedback` is still demoed via the death `screen_flash`. Enemy
  health pips (a `world_to_screen` bar per enemy) were also deferred; the "+N"
  popup is the `world_to_screen` showcase.
- Firing is hitscan (one-frame gizmo tracer + muzzle spark), not projectile
  entities -- an obvious polish follow-up noted in the docs.

Bugs hit and fixed during implementation: a botched per-enemy material (stray
`Handle::default()`) -> pre-built one material per enemy spec in `setup`; a
double-kill race (two towers dropping the same enemy in one frame) -> mark the
enemy `spent` before triggering the kill; a system-ordering hazard where
`place_or_select` read the `DragState` that `orbit_camera` writes -> pinned
`.after(orbit_camera)`; `DirectionalLight` field rename (`shadows_enabled` ->
gone) and a mesh borrow conflict in the fragments observer (`get(..).cloned()`);
and the discovery that `mesh/explode` has no `on_fragments_spawned` observer (the
crate only inserts `ExplodeFragments`), so a game-local observer was added like
`10_asteroids`.

Self-reflection: the biggest time sink was verifying the visual with towers,
because `BCS_AUTOPILOT` and `BCS_SHOT` cannot run together (both drive
`NextState`, so the screenshot never fires) -- worth remembering as a harness
constraint. The camera-rig design (apply `point_rotation`'s look quaternion to a
pivot, keep the camera at a fixed child offset) was the one non-obvious call and
came out clean; deriving the pitch-clamp sign from the module's math up front
avoided a fiddly trial-and-error loop. Line budget held at ~950, well under the
~2000 ceiling, by leaning on the juice kit and keeping tower/enemy variety at two
each.
