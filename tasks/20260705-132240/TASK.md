# breach -- multiple enemy archetypes with distinct stats

- STATUS: CLOSED
- PRIORITY: 60
- TAGS: spike,breach,example

## Goal

Give `14_breach` 2-3 distinct enemy archetypes instead of the single `Enemy`
stat block (`ENEMY_HEALTH`/`ENEMY_SPEED_BASE`/`ENEMY_DPS`, one octahedron mesh),
so waves read as escalating, varied threats. First cut: a fast-but-weak rusher
and a slow-but-tanky brute (optionally a third). Each archetype gets its own
stats (health, speed, DPS, size/mesh scale, colour) and weighted spawn share that
shifts as waves ramp.

Keep the roster as an in-file table/enum, NOT a JSON catalog: the `12_bastion`
data-catalog spike concluded a `SpecCatalog<T>` type should wait for a second
user, so enemy stats stay game-local data here.

A ranged or exploder enemy (projectiles / self-destruct AoE the arena has never
had) is explicitly OUT of scope for this task -- note it as a possible follow-up.

## Notes

- Spike: tasks/20260705-132024/SPIKE.md
- Reuse `mesh/builder` (octahedron variants at different scales), `health`,
  existing wave ramp (`wave_size`/`enemy_speed`); extend `spawn_wave` to pick an
  archetype per spawn by weighted roll.
- Straight-line AI + open arena is a hard constraint from the breach retro: do
  NOT add interior cover or an archetype that needs pathfinding/avoidance.
- Pure logic (the stat table, wave-composition / weighted-roll function) gets
  `#[cfg(test)]` tests. Assert the FINAL shipped roster count/weights (bastion
  embedded-catalog-count lesson: re-run tests after the data is finalized).
- Verify: `cargo clippy --all-targets`, headless `BCS_AUTOPILOT` run (aiming
  autopilot already scores kills), then run for real.

## Steps

- [x] **Archetype data table (in-file, not JSON).** `EnemyKind { Grunt, Rusher, Brute }`
  with `stats(kind) -> EnemyStats { health, speed_mult, dps, scale, color, emissive }`.
  Grunt keeps the current feel (health/dps = existing consts, mult/scale 1.0); Rusher is
  fast/weak/small, Brute is slow/tanky/big. Add `dps` to the `Enemy` component (per-enemy
  now, not a global const).
- [x] **Pure weighted spawn.** `archetype_weights(wave) -> [f32;3]` (grunts only early,
  rushers phase in ~wave 1+, brutes ~wave 3+, grunt clamped to a floor) and
  `pick_archetype(wave, roll) -> EnemyKind` over the cumulative weights. Both pure and
  unit-tested (wave 0 is all grunts; late waves can roll each kind; rusher share rises
  with wave).
- [x] **Spawn per archetype.** `spawn_wave` rolls a kind per ring position, then builds
  with its stats: `Health::new(stats.health)`, `Enemy { speed: enemy_speed(wave) *
  stats.speed_mult, dps: stats.dps }`, scaled octahedron mesh + collider, per-kind
  emissive material (bloom).
- [x] **Per-enemy melee.** `enemy_melee` sums each in-range enemy's `dps` (query
  `&Enemy`) instead of `attackers * ENEMY_DPS`, so a brute hurts more than a rusher;
  keep the attacker count for the juice throttle.
- [x] **Tests + verify.** Unit-test `archetype_weights`/`pick_archetype` and that each
  kind's stats differ as intended. Update the `death_app`/`kill_enemy` test enemy to the
  new `Enemy` shape. `cargo fmt`, `cargo clippy --all-targets`, `cargo test --example
  14_breach`, ascii, headless `BCS_AUTOPILOT`, real run. Update the `//!` header and
  AGENTS note.
