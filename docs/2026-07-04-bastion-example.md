# 12_bastion -- defend-the-core tower defense

`examples/12_bastion.rs` is a small (~950 line) tower-defense prototype. It is
the first example to exercise three crate modules that no other example touched,
and it grew out of the game-ideas spike
(`docs/spikes/20260704-220530-new-prototype-game-ideas.md`), which merged an
earlier two-game proposal into this single game on the user's steer.

## What it demonstrates

A glowing **Core** (a `Health` pool) sits at the center of a circular arena.
Enemies spawn all around the border and converge inward from every bearing; one
that reaches the Core damages it, and Core health at zero ends the run. Kills
earn credits, spent on placing and upgrading towers. Waves ramp count, speed and
toughness.

The point of the example is that its three core interactions each land on a
previously-undemoed module:

- **`camera/project` (the headline).** `pointer_on_plane` turns the pointer/tap
  pixel into the world point on the `y = 0` ground plane where a tower is placed
  (a ghost preview and range ring are shown there); `world_to_screen` anchors the
  floating "+N" credit popups over a killed enemy. This is the first example to
  import the harvested `camera/project` module rather than hand-rolling
  `viewport_to_world`.
- **`transform/point_rotation` drives the orbit camera.** An invisible pivot at
  the Core carries `PointRotation`; pointer drag (or A/D / arrow keys) feeds
  yaw/pitch deltas so the whole view orbits the battlefield. The camera is a
  child at a fixed angled offset, so the pleasant starting framing is decoupled
  from the orbit.
- **`transform/smooth_look_rotation` drives every tower turret.** A tower
  auto-targets the nearest enemy in range and its turret rotates toward it at a
  rate-limited `speed`; a fast enemy can briefly out-slew a cheap turret until it
  is upgraded, so the turn rate is a real per-tower stat, not decoration.

It reuses the established shape and juice kit: menu/playing/game-over `States`,
`SfxPlugin` one-shots, a `ui/status` FPS overlay plus an in-game HUD,
`mesh/explode` shards on a kill (a game-local `On<Insert, ExplodeFragments>`
observer turns them into short-lived `helpers/temp` debris), `ui/popup` "+N",
`camera/shake`, `feedback` screen flash on death, `scoring/streak` for the kill
combo, `time/cooldown` for the per-tower fire cadence, `input/pointer` for a
unified mouse+touch pointer, and the same wasm/trunk web build.

## Key design decisions

- **Camera angle.** An angled perspective camera (back + up from the pivot,
  looking at the Core) rather than a steep top-down, so the Core and towers read
  with depth and `camera/post` bloom looks good on the emissive Core/turrets.
  Verified with a `ScreenshotPlugin` grab.
- **Orbit rig, not a look controller.** `point_rotation` is really a
  look-direction controller (yaw about the current up, pitch about the current
  right). To turn it into an orbit camera, its output quaternion is applied to
  the *pivot* at the Core, with the camera as a child at a fixed offset -- so
  rotating the pivot swings the camera around the Core. Yaw is left unclamped
  (full 360); pitch is clamped in-game (the module has no min/max) by gating the
  pitch delta against the pivot's current `forward.y`, keeping the view from
  flipping under the ground or tipping past the horizon.
- **Tap vs drag on one pointer.** The pointer does double duty: a drag orbits the
  camera, a tap places/selects. `orbit_camera` owns a `DragState` that tracks the
  press, marks it a drag once it moves past a pixel threshold, and emits a
  one-frame `released_tap` on a clean release. `place_or_select` consumes that
  (pinned `.after(orbit_camera)` so the state is fresh -- the same
  ordering-against-a-real-edge caveat that bit `camera/shake` and the dev
  harness). Space is a keyboard placement path (also what the autopilot drives),
  placing at a ring-front point in front of the camera so orbiting moves the
  spot.
- **Game-local spec catalog.** Tower and enemy stats live in plain
  `TowerSpec`/`EnemySpec` structs with `tower_specs()` / `enemy_specs()` tables
  and spawn-by-index helpers. They are deliberately shaped so a follow-up task
  (`tasks/20260704-220719`) can move them to external JSON and evaluate a
  `SpecCatalog<T>` crate module, without reshaping the game. The MVP is not
  blocked on that abstraction.
- **Hitscan, not projectiles.** A tower deals damage directly to its target when
  the cooldown is ready and the turret is roughly aligned, drawing a one-frame
  gizmo tracer plus a brief muzzle spark. Projectile entities were left out to
  keep the first cut within the line budget; they are an obvious polish follow-up.

## Difficulties hit

- **`mesh/explode` has no fragment-spawned observer.** The AGENTS module map and
  an earlier read suggested an `on_fragments_spawned` hook, but the crate only
  inserts an `ExplodeFragments` component; `10_asteroids` defines its own
  `On<Insert, ExplodeFragments>` observer. This game does the same.
- **Enemy materials.** The first draft botched the per-enemy material (a stray
  `Handle::default()`); fixed by pre-building one material per enemy spec in
  `setup` and indexing it, which also keeps spawning cheap.
- **Autopilot + screenshot conflict.** Trying to capture a frame with towers by
  running `BCS_AUTOPILOT` and `BCS_SHOT` together did not work -- both plugins
  drive `NextState`, so the screenshot never fired. Verified the visual instead
  by temporarily lengthening the autopilot's Playing hold and grabbing the live
  window with `scrot`; the HUD (Core %, Credits, Score) and the moving enemies
  confirmed the full loop. Lesson: the two harness plugins are mutually
  exclusive; use one or the other per run.

## Post-merge fixes

Two control bugs the user hit playing the merged build, fixed on
`fix/12-bastion-orbit-upgrade`:

- **Orbit did nothing.** `orbit_camera` wrote `PointRotationInput` and read
  `PointRotationOutput` (for the pitch clamp) but never copied the output
  quaternion onto the rig's `Transform.rotation` -- and `PointRotationPlugin`
  only maintains the Output component, it does not touch the Transform. So the
  pivot stayed at identity and neither A/D nor drag moved the camera. Fixed by
  adding `&mut Transform` to the rig query and `transform.rotation = out.0`. This
  slipped through the original cycle because verification confirmed "no panic +
  towers/enemies present" but never confirmed the view actually *rotated* -- a
  proxy check, not the real behavior.
- **Upgrade was unreachable.** A tap only selected a tower when *not* in build
  mode, so after building you had to press Q before U would do anything, and the
  HUD gave no hint. Now a real tap on an existing tower selects it in any mode
  (Space stays pure placement), and the HUD's second line shows the selected
  tower, its level, and `press U to upgrade (Nc)`.

## Verification

- `cargo clippy --all-targets` and `--features debug`: clean.
- `cargo test --example 12_bastion`: 5 pure-logic tests pass (ring-point on the
  arena, angle-diff wrapping, placement validity, upgrade cost, wave growth).
- `cargo fmt --check` and `scripts/check-ascii.sh`: clean.
- Boots and reaches the render loop; `BCS_AUTOPILOT=1 ... --features debug` runs
  the full menu -> playing -> game-over cycle with `no panic`, driving tower
  placement and firing.
- `trunk build --release --example 12_bastion ...`: wasm build succeeds; the web
  game is registered in `web/src/games.ts` and `web/scripts/build-games.sh`.

## Possible follow-ups

- Data-driven specs (`tasks/20260704-220719`).
- Visible projectiles instead of hitscan; enemy health pips (a `world_to_screen`
  bar over each enemy, sketched but deferred).
- A dedicated on-screen build/upgrade UI for touch instead of number keys.
