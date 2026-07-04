# 08_dropzone hazards pass: obstacles, asteroids, wind, ship integrity

Date: 2026-07-04
Task: `tasks/20260704-103553` (from spike `tasks/20260704-102022`, Part 1 Tier B:
B1 obstacles + B4 wind), sequenced after the Tier-A fun pass
(`tasks/20260704-103544`).

## What changed

`examples/08_dropzone.rs` went from "a small landing game" to "the same game, but
dangerous", without touching the flight model (PD controller, gravity, thrust),
the Menu/Playing/Result state machine, or turning it into a bigger game. Four
hazard families were added, all reusing existing crate systems:

- **Rough-terrain obstacles (B1).** `ROCK_COUNT` rock monoliths (static cuboid
  colliders) are ringed around the pad at `ROCK_RING_ANGLE` from the pad
  direction, spanning `ROCK_RING_SPAN_FRAC` of the circle so a threadable gap is
  left on the final approach. Each stands radially on the real terrain (the pad
  noise is sampled for the surface height, as the beacon already did). Hitting
  one costs integrity (see below); a fast smash empties the pool and ends the
  run.
- **Drifting asteroids (B1).** `ASTEROID_COUNT` lumpy asteroids (a noise-displaced
  `TriangleMeshBuilder` octahedron) wander through the descent corridor on the
  crate's `RandomSphereOrbit` driver, seeded near the +Y pole (high phi) and in
  an altitude band below the ship's spawn so a run never opens inside one. A
  graze damages the ship and shatters the asteroid via `mesh/explode`.
- **Ship structural integrity.** The ship carries a `Health` pool
  (`SHIP_MAX_INTEGRITY`). Obstacle and asteroid contacts route through
  `HealthApplyDamage` scaled by impact speed (`impact_damage`: linear with a
  floor, so a slow graze still stings and a fast hit can be one-shot lethal).
  When integrity hits zero the `HealthZeroMarker` observer (`on_ship_destroyed`)
  ends the run as a structural-failure crash (explode + Result). A `hull %` HUD
  gauge shows it, green -> amber -> red.
- **Wind / gust (B4).** A `Wind` resource evolves one phase into a slowly
  rotating bearing and a smooth 0..1 gust envelope; the resulting tangential
  acceleration is folded into the ship's existing world-space acceleration
  channel (one more term alongside gravity, exactly the pattern the spike
  called for). It is telegraphed by translucent streak particles blown downwind
  past the ship (denser as the gust builds) and a `wind %` HUD gauge.

All hazard magnitudes scale off one `HAZARD_DIFFICULTY` scalar (rock count,
asteroid count, wind peak), so a future difficulty ramp is a one-liner; the ramp
itself is deliberately out of scope (spike idea C1 was cut).

## Decisions and trade-offs

- **Terrain contact stays instant-crash; only obstacles/asteroids cost
  integrity.** The task asked this to be decided explicitly. Keeping a hard
  ground impact catastrophic preserves the core landing game unambiguously (land
  soft-and-upright, or die), and reserves the `Health` pool for the new
  hazards, where "a graze chips, a smash kills" is the whole point. A rock or an
  asteroid is a structural hit; the ground is the ground.
- **Discrete rock colliders instead of cranking `TERRAIN_AMPLITUDE`.** The task
  offered either. Cranking global amplitude would also disturb the pad's flush
  placement (the beacon and pad ring sit on the sampled noise surface) and the
  landing feel across the whole planet. Discrete monoliths near the pad are
  controllable, create a deliberate gap to thread, and are the reason the
  approach now has skill expression. `TERRAIN_AMPLITUDE` is left at 0.10.
- **Asteroid hits use a proximity (distance) check, not physics colliders.**
  This matches how `07_orbit` and the fuel cans already detect contact, and it
  sidesteps the tunneling risk the task flagged: an asteroid driven by
  `RandomSphereOrbit` would have to be a kinematic body teleported along its
  orbit each frame, which fights the solver and can tunnel at speed. A per-frame
  distance test has neither problem. Rocks, being static, keep real colliders
  (the ship should physically stop against them), so no CCD was needed and none
  was added -- consistent with the base game's note that current speeds do not
  tunnel. If a future speed increase tunnels the ship through a rock, enable
  avian `SweptCcd` on the ship.
- **Asteroid shatter without a physics body.** On a hit the asteroid drops its
  `Asteroid` marker (stops repeat hits and the transform sync), gets an
  `ExplodeMesh` (the observer slices it into flying debris), and is hidden +
  `TempEntity`-despawned. `mesh/explode` only adds fragments -- it does not hide
  the shell -- so hiding + auto-despawn is how the spent shell disappears while
  the debris flies (the same reason the ship crash uses `DespawnOnExit`).
- **`Wind` folded into the gravity channel.** avian allows one
  `ConstantLinearAcceleration` per body, so wind is added to the same world-space
  vector as radial gravity (`gravity.0 = -radial_up * GRAVITY + wind.accel`)
  rather than as a second component. Wind is tangential (perpendicular to
  radial up), gravity is radial, so they compose cleanly.

## Verification

`cargo fmt --check`, `cargo clippy --all-targets` (clean bar the transitive
`proc-macro-error2` future-incompat note), `cargo test` (9 example tests incl.
the new `impact_damage` one), and `scripts/check-ascii.sh` all pass. Per the
AGENTS.md "run it, do not just build it" rule, a temporary env-gated autopilot
(`DROPZONE_SMOKE`, the technique from the Tier-A and tuning cycles) flew the real
systems through Menu -> Playing -> Result headlessly and confirmed, with no
panic or query conflict:

- boot to the render loop; 7 asteroids spawn and persist; wind evolves as a
  smooth gust envelope (2.0 -> 3.2 -> 2.9) and moves the ship;
- snapping the ship onto asteroids drained hull 100 -> 47 and dropped the
  asteroid count as they shattered (proximity damage + `mesh/explode` path);
- snapping onto a rock at 6.3 m/s dealt obstacle-branch damage that emptied the
  remaining hull and ended the run (the solid-impact + integrity-kill path);
- a forced lethal `HealthApplyDamage` ended a free-flying run via
  `on_ship_destroyed`.

The harness was removed before commit.

## Tuning follow-up (playtest feedback)

A first playtest found the descent too punishing, so three constants were eased
(the difficulty scalar and the mechanics are unchanged):

- **Rocks** went from 6 uniform full-height (`ROCK_HEIGHT`) monoliths in a tight
  ring to 3 with a **random height** (`ROCK_MIN_HEIGHT`..`ROCK_HEIGHT`, so a low
  one is trivially cleared), pushed **further out and jittered** in distance
  (`ROCK_RING_ANGLE` 0.085 -> 0.16 plus a `ROCK_RING_ANGLE_SPREAD` per-rock
  jitter) with a wider gap (`ROCK_RING_SPAN_FRAC` 0.78 -> 0.7). Because heights
  now vary per rock, each gets a per-run cuboid mesh (matching its collider);
  only the shared rock material stays in `HazardAssets`.
- **Wind** peak was cut ~5x (`WIND_PEAK_ACCEL` 3.2 -> 0.64) to a gentle nudge.
- **Fuel** lasts ~3x longer via lower consumption (`FUEL_BURN` 14.0 -> 14.0/3),
  chosen over raising the max so the 0..100% gauge stays intact.

## Follow-ups (not filed)

- The remaining hazard constants (asteroid count/speed/altitude band, damage
  scaling) are reasoned, not play-tuned on a human descent, like the earlier
  flight-constant tuning cycle (`tasks/20260703-213510`). They want a further
  play-test pass.
- If a fourth per-run resource appears, extract a `reset_run_state` helper in
  `start_run` (carried over from the Tier-A retro; `Wind` is now the sixth reset
  resource, so this is closer to worth doing).
