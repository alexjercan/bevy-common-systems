# 08_dropzone: hazards pass (obstacles, asteroids, wind, rough terrain, ship integrity)

- STATUS: CLOSED
- PRIORITY: 3
- TAGS: feature,dropzone

## Goal

Second follow-up from the fun+mobile spike (`tasks/20260704-102022`, Part 1
Tier B: B1 obstacles + B4 wind). Add danger and skill expression to the descent
WITHOUT turning dropzone into a bigger/more complicated game (no pickups beyond
fuel, no upgrades, no cargo, no multi-leg -- those were cut). Best done AFTER the
Tier-A fun pass (`tasks/20260704-103544`), which adds the pad/fuel/juice this
builds on.

Scope = the hazard/obstacle family only:
- static rough-terrain hazards and obstacles you can hit,
- moving asteroids/debris (using the `transform/*` orbit family),
- wind/gust lateral force you fight with lean,
- and, if asteroids are in, ship structural integrity / health so a graze
  damages rather than instantly killing.

## Baseline (from the spike, for grounding)

- The planet is a `TriangleMeshBuilder` sphere with a trimesh collider built
  inline; `TERRAIN_AMPLITUDE` is currently low (0.10). Rougher terrain = crank
  amplitude and/or add rock meshes with colliders.
- Gravity and thrust are applied as avian *acceleration* components
  (`ConstantLinearAcceleration` / `ConstantLocalLinearAcceleration`), overwritten
  each `FixedUpdate`. Wind is just one more tangential acceleration component --
  the pattern already exists.
- `07_orbit` already drives wandering orbs/hazards on a sphere with
  `RandomSphereOrbit`; asteroids can reuse that.
- `HealthPlugin` exists (`Health` component, `HealthApplyDamage` entity event
  propagating up the hierarchy, `HealthZeroMarker` at zero) and is unused by
  dropzone today; it is the natural home for ship structural integrity.
- Crash today = insert `ExplodeMesh` on hard impact; the explode path is reusable
  for asteroid-kill destruction.

## Steps

- [x] Obstacles / rough terrain (B1). Added discrete rock monoliths (static
      cuboid colliders, `Obstacle` marker) ringed around the pad, spanning
      `ROCK_RING_SPAN_FRAC` of the circle so a gap is left to thread. Chose
      discrete rocks over cranking `TERRAIN_AMPLITUDE` (which would disturb the
      pad's flush placement); noted in the doc. A rock hit costs integrity and
      can end the run.
- [x] Moving asteroids / debris (B1). `ASTEROID_COUNT` lumpy asteroids
      (`TriangleMeshBuilder` octahedron) wander the corridor on `RandomSphereOrbit`,
      seeded near the pole. A graze damages the ship and shatters the asteroid
      via `mesh/explode` (proximity check, not a physics collider -- avoids the
      tunneling risk of teleporting a kinematic body along an orbit).
- [x] Ship structural integrity / health. Ship carries `Health::new(SHIP_MAX_INTEGRITY)`;
      obstacle/asteroid hits route through `HealthApplyDamage` scaled by impact
      speed (`impact_damage`, unit-tested), a light graze chips and a hard hit
      can empty it. `HealthZeroMarker` -> `on_ship_destroyed` ends the run as a
      structural-failure crash. A `hull %` HUD gauge surfaces it. DECISION:
      terrain contact stays instant-crash; only obstacles/asteroids cost
      integrity (keeps the core landing game unambiguous). Noted in the doc.
- [x] Wind / gust (B4). A `Wind` resource evolves one phase into a rotating
      bearing + smooth gust envelope; the tangential acceleration is folded into
      the ship's existing `ConstantLinearAcceleration` (one more term). Telegraphed
      by translucent streak particles blown downwind and a `wind %` HUD gauge.
- [x] Difficulty knobs. All hazard magnitudes scale off one `HAZARD_DIFFICULTY`
      scalar (rock count, asteroid count, wind peak); the ramp itself is left out
      of scope, constants are tunable.
- [x] Verify: fmt/clippy(--all-targets)/test/ascii all clean. Ran the example
      (reaches render loop); a temporary env-gated autopilot (since removed) flew
      Menu -> Playing -> Result confirming each hazard path (asteroid damage +
      shatter, obstacle impact kill, integrity drain, wind pushes the ship) with
      no panic. No tunneling observed (asteroids use proximity; rocks are static),
      so no CCD added. Web showcase rebuilt (`npm run build` exit 0, webpack ok).
      Decisions in `docs/2026-07-04-dropzone-hazards.md`.

## Close-out

Shipped the whole hazard/obstacle family on `feature/08-dropzone-hazards`,
reusing existing crate systems (`transform/random_sphere_orbit`, `health`,
`mesh/explode`) per AGENTS.md. `resolve_landing` was refactored into
`resolve_collisions`, which classifies each ship contact as planet (landing eval,
unchanged) vs obstacle (integrity damage) -- stricter and more correct than the
old "any contact = touchdown". Nine in-module unit tests (added `impact_damage`).
Review (REVIEW.md): round 1 APPROVE -- an independent skeptical pass found no
CRITICAL/MAJOR issues, only three MINOR cosmetic/theoretical edges (one
documented with an inline comment). See
`docs/2026-07-04-dropzone-hazards.md` for the decisions and trade-offs.

Scope held: flight model (PD/gravity/thrust) and the Menu/Playing/Result shape
untouched; no out-of-scope mechanics added.

## Notes

- Deliberately OUT of scope (user cut): pickups beyond fuel, upgrades, cargo,
  multi-leg refuel, multiple planets/leaderboards. Keep it "the same game, but
  dangerous", not a bigger game.
- Structural integrity is the one place `HealthPlugin` earns its keep in this
  example -- only add it if asteroids/grazes are in, otherwise instant-crash is
  fine and simpler.
- Faithful to AGENTS.md: reuse existing crate systems (`transform/*`, `health`,
  `mesh/explode`), example is the integration test (run it), wasm-friendly,
  plain ASCII.
- Depends conceptually on `tasks/20260704-103544` (Tier-A) landing on top of it;
  sequence after that unless done together.
