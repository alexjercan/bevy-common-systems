# 08_dropzone: hazards pass (obstacles, asteroids, wind, rough terrain, ship integrity)

- STATUS: OPEN
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

- [ ] Obstacles / rough terrain (B1). Add terrain hazards the ship can hit:
      crank local `TERRAIN_AMPLITUDE` for spires/arches the existing trimesh
      already supports, and/or add discrete rock meshes with colliders near the
      pad so the approach has to thread a gap. Hitting one at speed = crash via
      the existing explode path. Cheapest hazard; do first.
- [ ] Moving asteroids / debris (B1). Spawn drifting asteroids using the
      `transform/*` orbit family (`RandomSphereOrbit`, as `07_orbit` does) or
      simple ballistic motion. Collision with the ship deals damage (see
      structural integrity below) and can shatter the asteroid via
      `mesh/explode`.
- [ ] Ship structural integrity / health. Give the ship a `Health` component;
      route asteroid/graze impacts through `HealthApplyDamage` scaled by impact
      speed, so a light graze costs integrity but a hard hit still ends the run
      (via `HealthZeroMarker` -> crash/explode). Surface integrity on the
      `ui/status` bar next to altitude/speed/fuel. Decide + note: does terrain
      contact also cost integrity, or stay instant-crash as today?
- [ ] Wind / gust (B4). Add a time-varying tangential
      `ConstantLinearAcceleration` the player must counter with lean. One more
      acceleration component, same pattern as gravity/thrust. Keep it readable
      (telegraph direction/strength, e.g. a HUD wind indicator or drifting
      particles). Tune so it adds challenge without being unfair.
- [ ] Difficulty knobs. Wire hazard density / wind strength / terrain roughness
      to a difficulty scalar so they can ramp (leave the ramp itself optional;
      the spike's multi-level idea C1 is out of scope, just make the constants
      tunable).
- [ ] Verify: `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test`,
      `./scripts/check-ascii.sh`, then RUN the example and confirm the render
      loop plus each hazard behaves (obstacle crash, asteroid damage + integrity
      drain + kill, wind pushes and is counterable). PHYSICS-TUNING RISK: fast
      asteroid/terrain collisions may reintroduce tunneling the base game
      avoided (the doc notes no `SweptCcd` was needed at current speeds) -- if a
      fast hit tunnels, enable avian CCD on the relevant bodies and note it.
      Rebuild the web showcase (`npm run build`) and document decisions in
      `docs/`.

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
