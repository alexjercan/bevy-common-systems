# 14_breach: a grounded first-person arena shooter

- DATE: 2026-07-05
- Example: `examples/14_breach.rs`
- Spike: `docs/spikes/20260705-103116-grounded-fps-example.md`
- Task: `tasks/20260705-103236`

## What it is

`14_breach` is the gallery's first first-person game: a grounded, Doom-like arena
shooter. You stand in a walled arena, waves of glowing octahedron enemies converge
from a ring around you, and a hitscan rifle guns them down before they melee you to
death. It headlines three things no prior example showed:

1. **The first-person viewpoint as a real game.** `camera/wasd` only ever appeared
   in the `01/02/04/05` free-fly tech demos.
2. **The crate's first avian `SpatialQuery` raycast** (the gun is hitscan).
3. **A game-local first-person character controller** (walk + gravity +
   collide-and-slide + grabbed-mouse look).

## Why the design is what it is

### The controller: dynamic body + velocity control, not kinematic

The crate's `camera/wasd` is a free-fly spectator camera (accumulates mouse delta
into yaw/pitch and integrates movement straight into `Transform`, with no gravity,
ground, collision or cursor grab), so it cannot be a grounded FPS controller and, used
as-is, would fight a physics body for ownership of the `Transform`. So the player is
game-local:

- a `RigidBody::Dynamic` capsule with `LockedAxes::ROTATION_LOCKED`, driven by
  writing `LinearVelocity` (xz from input, y left to gravity). avian's solver then
  does **collide-and-slide against the static level for free**. This was the key
  decision: a `RigidBody::Kinematic` body is *not* pushed back by static geometry
  (avian leaves kinematic-vs-static resolution to you -- `10_asteroids` reflects its
  ship's velocity off the walls by hand), so a kinematic FPS body would need a
  hand-rolled shapecast collide-and-slide. Dynamic + locked rotation is far simpler
  and robust.
- The body stays axis-aligned (identity rotation). Yaw lives on a
  `FirstPersonController`; the `Camera3d` is a **child at eye height** whose local
  rotation carries the full view (`Quat::from_euler(YXZ, yaw, pitch, 0)`), and the
  WASD move intent is rotated by the same yaw. This clean split means the physics
  body never rotates (no fighting the locked axes) while the view turns freely.

### Look: grabbed cursor + AccumulatedMouseMotion

Look is always-on (not the RMB-drag `helpers/wasd` uses). On entering Playing the
cursor is locked and hidden via the `CursorOptions` component (a per-window component
in Bevy 0.19, not `window.cursor`), and released on exit / game-over. The per-frame
delta comes from the `AccumulatedMouseMotion` resource; pitch is clamped to +/-1.54
rad so the view cannot flip.

### The gun: hitscan via SpatialQuery::cast_ray

Left-click (gated by a `time/cooldown`) casts a ray from the camera's `GlobalTransform`
forward with `SpatialQuery::cast_ray(origin, Dir3, range, solid, &filter)`, the filter
masking `[Enemy, World]` and excluding the player entity. The first enemy hit takes
`HealthApplyDamage`, a `feedback/flash`, and on death an `ExplodeMesh` burst; a tracer
(`helpers/temp`), a recoil `camera/shake` and a gunshot round it out. This is the
crate's first use of avian's spatial queries.

## Bugs found and fixed

### Tracer/flash vs despawn race (runtime error)

The first working autopilot run flooded the error handler with `Entity despawned`
errors: `player_shoot` triggered `HealthApplyDamage` and *then* inserted `Flash` on the
same enemy. A lethal hit runs the whole death chain (`HealthZeroMarker` ->
`on_health_zero` -> `ExplodeMesh` -> `on_fragments_spawned` -> despawn) during the
command flush, so the `Flash` queued afterwards landed on a despawned entity. Fix:
insert `Flash` **before** triggering the damage, so it applies while the enemy is still
alive. (Same class of bug as the `13_glide` tween-completion-vs-despawn race: order the
side effect before the thing that despawns.)

### Verifying an aim-based FPS headlessly

A first-person gun can't be verified by the usual "press W + fire" autopilot: the
player faced a wall and shot it while being swarmed, scoring zero kills, so the
raycast -> damage -> kill path was never exercised (a screenshot at Playing entry only
shows the initial scene, per the `13_glide` state-entry-screenshot lesson). The fix was
to make the `AutopilotPlugin` input closure **aim**: it queries the nearest enemy and
sets the `FirstPersonController` yaw to face it before pressing fire (the look system
can't be driven by injected mouse motion). With aiming, the run scores kills (the
persisted best went from 0 to a positive number), which is the headless proof that the
whole gun path works. Spawning the player at the arena centre (rather than the edge)
also makes both the game and the autopilot sensible.

## Modules exercised

New: the FP controller, avian `SpatialQuery` (hitscan), avian `PhysicsLayer` /
`CollisionLayers` for the ray filter, and Bevy 0.19 cursor grab. Reused: `health`
(player + enemies), `mesh/explode` (enemy gibs) + `mesh/builder` (octahedron enemies),
`feedback/flash` (hit-flash) + `feedback/screen_flash` (damage vignette), `camera/shake`
+ `camera/post` (bloom on tracers/enemies), `time/cooldown` (fire rate + enemy attack
cadence), `helpers/temp` (tracers, gibs), `ui/status` (HUD), `ui/menu`, `audio`,
`persist` + `scoring/high_score`, `ui/touchpad` (dual-stick touch), `input/state`.

## Melee reliability and the open arena (found in review)

The first cut had a per-enemy attack `Cooldown` gating a distance check, and a few
cover blocks. Both were wrong, and a headless probe (extending the autopilot's
Playing hold and disabling its fire so the player stands defenceless) exposed it: a
passive player survived 20-30s, so the core "survive the swarm" threat barely
functioned and the player-death path was never actually reached. Two root causes and
fixes:

- **Melee was cooldown-gated and unreliable.** Enemies jostle in and out of
  `MELEE_RANGE` faster than a 0.9s cooldown cycles, so a standing player took almost
  no damage. Replaced with **continuous proximity damage**: every frame, each enemy
  within range drains `ENEMY_DPS * dt` (summed over attackers), so a swarm reliably
  melts you. Also, the player no longer physically collides with enemies (only the
  world) -- dynamic-vs-dynamic knockback was flinging approaching enemies out of
  range; now they overlap you and the distance melee lands.
- **Enemies got stuck on cover.** The straight-line enemy AI has no obstacle
  avoidance, so any interior block sat on some enemy's radial path and stranded it.
  The arena is now **open** (floor + perimeter walls only); enemies always reach, and
  the player kites in the open.

With both fixes a defenceless player dies in ~10s (~7s for enemies to cross the arena
plus ~3s to be melted), and the death path (`RunOver` -> `check_run_over` -> GameOver)
is covered by a headless `App` unit test rather than trusted to the eye -- the
autopilot force-transitions Playing->GameOver on a timer, so it can NOT prove the
lose condition (the `13_glide` state-entry-screenshot lesson applied to a state
machine).

## Known limitations

- **Enemy AI is straight-line.** No pathfinding; the open arena is what keeps that
  from being a problem. A smarter AI / cover would need navigation.
- **Touch is a compromise.** Dual virtual sticks + a fire button back the wasm build,
  but an FPS is the hardest genre for touch; the gallery blurb says desktop-first.

## Follow-up

The game-local `FirstPersonController` (walk + gravity + collide-and-slide +
grabbed-mouse look) is the subject of the harvest follow-up `tasks/20260705-103238` --
whether it (and/or `camera/wasd` upgrades: always-on look, a cursor-grab helper, pitch
clamp) should become a crate module. Kept game-local here on purpose.
