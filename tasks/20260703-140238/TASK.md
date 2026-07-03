# Fruit ninja: difficulty ramp over time

- STATUS: CLOSED
- PRIORITY: 95
- TAGS: feature,example

## Goal

Make `examples/06_fruitninja.rs` get harder the longer a run lasts: fruit
spawn faster and bombs get more common as elapsed play time grows, turning a
flat game into one with a difficulty curve.

## Steps

- [x] Add an `Elapsed(f32)` resource (or reuse a run timer) reset in
      `start_game`, ticked each frame in `Playing`.
- [x] Replace the fixed `SPAWN_INTERVAL` timer usage in `spawn_projectile` with
      an interval that shrinks with elapsed time (e.g. lerp from 0.9s down to a
      floor like 0.35s over ~60s); set the repeating timer's duration each time
      it fires, or drive spawning from a computed interval.
- [x] Scale bomb probability from `BOMB_CHANCE` up to a cap (e.g. 0.20 -> 0.35)
      as elapsed time grows, used in `spawn_projectile` instead of the constant.
- [x] Add named consts for the start/floor spawn interval and start/cap bomb
      chance and the ramp duration, so the curve is easy to tune.
- [x] Optional: extract the ramp math into a pure helper (e.g.
      `spawn_interval_for(elapsed)` / `bomb_chance_for(elapsed)`) and unit-test
      the endpoints (t=0 -> start, t>=ramp -> floor/cap), matching the example's
      pure-helper + `cargo test --example` pattern.
- [x] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, real boot with no panic.

## Notes

- Current: `SPAWN_INTERVAL = 0.9` (:53), `BOMB_CHANCE = 0.2` (:79),
  `spawn_projectile` at :543 uses `SpawnTimer` + `rng.random_bool(BOMB_CHANCE)`.
- `start_game` (:476) already resets score/timer/blade/combo; add the elapsed
  reset there.
- Keep the curve gentle at first so early game stays approachable.
- No new dependencies.

## Close-out

Added `Elapsed` run clock (ticked in Playing, reset in start_game) and pure
`spawn_interval_for`/`bomb_chance_for` helpers (ease start->floor/cap over
DIFFICULTY_RAMP_SECS). spawn_projectile ramps the next SpawnTimer duration and
bomb chance from elapsed. 3 unit tests cover endpoints + midway. Verified boot.
