# Review: difficulty ramp over time

- TASK: 20260703-140238
- BRANCH: feature/ninja-ramp

## Round 1

- VERDICT: APPROVE

Self-review. Ramp logic is pure (`ramp_t`/`spawn_interval_for`/`bomb_chance_for`)
and unit-tested at endpoints and midway - the CI-testable-core lesson. Elapsed
resets in start_game (per lesson); the SpawnTimer duration is ramped on each
fire and reset in start_game. Checks clean (fmt, clippy both, ascii), 14 example
tests pass, boots without panic.
