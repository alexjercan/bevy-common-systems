# 08_dropzone Tier-A fun pass: landing pad, fuel cans, contact juice

Date: 2026-07-04
Task: `tasks/20260704-103544` (from spike `tasks/20260704-102022`, Part 1 Tier A)

## What changed

`examples/08_dropzone.rs` went from "a physics demo you can land" to "a small
game with a goal and a route to plan", without touching the flight model (PD
controller, gravity, thrust) or the Menu/Playing/Result state machine's shape.
Four additions, all from the spike's Tier A:

- **A1 - landing pad + positional score.** A fixed glowing beacon (a flat ring
  plus a tall thin light column, both emissive so `camera/post` bloom lights
  them) sits on the surface, offset `PAD_ANGLE` (0.32 rad) from the +Y spawn
  pole. `landing_score` gains a `proximity_bonus` that fades linearly from
  `PAD_PROXIMITY_MAX` at the pad centre to zero at `PAD_REWARD_RADIUS`, measured
  as great-circle surface distance from the touchdown point to the pad. A HUD
  item shows live distance to the pad ("pad Nm") as a homing hint.
- **A2 - fuel cans.** Three emissive green canisters are strung down the
  start->pad descent and pushed off that line (`FUEL_CAN_OFFSET` along a vector
  perpendicular to the descent chord), so grabbing one is a deliberate detour.
  A distance check collects them: fuel is topped up (capped at `START_FUEL`),
  `pickup.wav` chirps, and a "+FUEL" popup floats up at the can's screen
  position.
- **A3 - descent timer.** A run timer ("t Ns" on the HUD) and a `time_bonus`
  that decays to zero at `PAR_TIME` and never goes negative, so a brisk descent
  is rewarded but a careful slow one is never punished.
- **A4 - contact juice.** On any touchdown a dust puff kicks up (reusing the
  `FragmentMotion` integrator and `helpers/temp`, exactly like the crash debris)
  and the camera takes a punch (a ported `07_orbit` `CameraShake`, applied after
  `ChaseCameraSystems::Sync` so the jolt is additive on the chase transform). A
  soft landing now freezes the hull in place (`RigidBody::Static` + zeroed
  velocity) and keeps it visible on the result screen, so you actually see the
  ship parked on the pad instead of it vanishing the instant you touch down.

## Key decisions

### Pad placed on the real surface, scored by angle

The mesh's `apply_noise` displaces each unit-sphere vertex to
`p * (1 + noise(p))`, then `with_scale` multiplies by the radius. So the surface
radius at a direction is `PLANET_BASE_RADIUS * (1 + noise(dir))`. The pad
evaluates the *same* `ScaledNoise` at its direction to sit flush on the terrain
rather than floating or burying. Proximity is scored by the angle between the
touchdown ground-track and the pad direction times the radius (great-circle
surface distance), which is robust regardless of terrain height.

### Distance-to-a-target, not raw distance

The original request floated "landing further from the start = more points". The
spike argued (and this implements) the inverse: reward landing *close to a
designated pad*. Raw distance perversely rewards flying off to nowhere; a pad
offset from the spawn pole gives the same "go somewhere" thrill but with an
actual goal and a lateral-steering decision during descent.

### Landed ship kept visible; crash keeps the proven despawn timing

Previously the ship carried `DespawnOnExit(Playing)`, so it vanished the moment
the state flipped to Result - fine for a crash (the fragments carry the show),
wrong for a landing (you never see it land). Now the ship spawns *without*
`DespawnOnExit`; a soft landing freezes it and it is cleaned up by a
`despawn_ships` system on leaving Result. The crash branch *re-adds*
`DespawnOnExit(Playing)` so the shattered hull still disappears as its fragments
spawn - keeping the exact, already-proven ordering (fragments spawn via the
`On<Insert, ExplodeFragments>` observer before the state-exit despawn runs).

### Fuel cap on pickup

Collected fuel is capped at `START_FUEL` so the "%" gauge never exceeds 100 and
the reading stays meaningful. Overfill was considered and rejected as it would
break the gauge's semantics for no gameplay gain.

### Dust and shake reuse existing machinery

Dust particles are just `FragmentMotion` + `TempEntity` entities, integrated by
the existing `move_fragments` (which already applies radial gravity and runs in
all states, so dust settles into the Result screen and auto-despawns). The
camera punch is the `07_orbit` `CameraShake` pattern verbatim, ordered after the
chase sync in PostUpdate. No new plugins or dependencies.

## Testing / verification

- Four in-module unit tests cover the pure scoring and placement logic: a
  bullseye beats a far/slow landing by exactly the proximity+time bonus, the
  proximity bonus clamps to zero past the reward radius, a slower-than-par
  landing is never penalised, and the fuel cans sit off the descent line and
  above the surface. (`cargo test --example 08_dropzone`: 4 passed.)
- `cargo fmt --check`, `cargo clippy --all-targets`, `scripts/check-ascii.sh`
  all clean.
- Ran the example (`DISPLAY=:0`): it reaches the render loop cleanly (menu). To
  exercise the Playing->Result path without a keyboard, a temporary env-gated
  autopilot (`DROPZONE_SMOKE`, since removed) flew 741 frames of real physics to
  a successful soft landing (score 497, confirming pad+fuel+time bonuses all
  compute), with no gameplay panic and the hull frozen on the surface.

## Added during review: randomized layout + guide

Play-review feedback asked for a fresher, more navigable run. Three follow-on
changes (REVIEW.md R1.1-R1.3):

- **Randomized pad each run.** The pad moved from a fixed `setup` entity to a
  per-run entity rolled in `start_run`: a random azimuth and a polar angle in
  `[PAD_ANGLE_MIN, PAD_ANGLE_MAX]` = `[0.18, 0.5]` rad around the +Y pole, kept
  well clear of the antipode (where the `from_rotation_arc` upright target is
  singular) and within a fuelled lateral reach. The terrain noise is retained in
  a `PlanetNoise` resource so the beacon still sits flush on the surface. The pad
  is a per-run entity (marker `Pad`) kept visible into Result and cleared by
  `cleanup_run_scene` on leaving Result, alongside the parked ship.
- **Randomized, self-refilling fuel field.** Fuel cans now spawn at random spots
  in the descent cap (`random_fuel_can_pos`) and a `maintain_fuel_cans` system
  keeps `FUEL_CAN_TARGET` (3) of them present: while at target the refill timer
  stays primed, and once a can is collected a replacement spawns after
  `FUEL_CAN_SPAWN_INTERVAL` (2.5 s), so the field refills over time rather than
  instantly or emptying out. Shared can mesh/material live in a `FuelCanAssets`
  resource so top-ups do not allocate.
- **Diegetic guide arrow.** An emissive cone (`GuideArrow`) hovers
  `GUIDE_ARROW_HEIGHT` above the ship and, each frame in `Playing`, points along
  the ship's ground track toward the pad (the tangent-plane component of the
  ship->pad vector), collapsing to "point up" when the ship is directly over the
  pad. It complements the numeric "pad Nm" HUD readout with an in-world heading.

## Difficulties

- The landed-ship-stays-visible feature is the one change that touched entity
  lifecycle rather than just adding entities. The trap was that removing
  `DespawnOnExit(Playing)` outright would leave a crashed hull lingering; the
  fix (re-add it only in the crash branch) preserves the original, load-bearing
  spawn->explode->despawn ordering that the earlier example was careful about.
- Verifying gameplay in a background session with no input-injection tool
  (`xdotool` absent). Solved by reviving the tuning task's env-gated autopilot
  idea just long enough to prove the cycle, then deleting it - the example stays
  minimal but the novel path was actually flown, not just reasoned about.
