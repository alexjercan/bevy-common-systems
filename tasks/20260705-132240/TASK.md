# breach -- multiple enemy archetypes with distinct stats

- STATUS: OPEN
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

- Spike: docs/spikes/20260705-132024-breach-fun-pass.md
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
