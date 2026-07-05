# Review: breach -- enemy archetypes

- VERDICT: APPROVE
- ROUNDS: 1

## Verified

- `cargo fmt --check`, `cargo clippy --all-targets` (clean), `cargo test --example
  14_breach` (16 pass), `scripts/check-ascii.sh` (clean).
- Headless `BCS_AUTOPILOT` run: full cycle, no panic. (The 4s Playing hold only reaches
  wave 0, which is all-grunts by design, so archetype variety is proven by the pure
  tests, not the autopilot -- consistent with the "autopilot proves boot, tests prove
  logic" rule.)
- Real windowed run reached the render loop.

## Findings / checks

- No correctness issues.
- Kept the roster an in-file `EnemyKind::stats` table, NOT a JSON catalog, per the
  bastion data-catalog spike ("wait for a second user").
- Weighted spawn is a pure `archetype_weights`/`pick_archetype` over cumulative weights,
  unit-tested: wave 0 is all grunts, rushers phase in by wave 1, brutes by wave 3, the
  rusher share rises with the wave, and weights sum to 1 at several waves. Stat
  distinctness (rusher faster/weaker, brute slower/tankier/harder) is asserted too.
- Melee is now per-enemy `dps` summed over in-range attackers (was `count * ENEMY_DPS`),
  so a brute genuinely hurts more; the attacker count still gates the juice throttle.
- Straight-line AI + open arena constraint respected: no new obstacles, no archetype
  needing pathfinding (a ranged/exploder enemy was explicitly deferred).

## Nits (non-blocking)

- `spawn_wave` still builds a fresh mesh per enemy (pre-existing); could cache one mesh
  per archetype. Not worth changing in this task.
