# Review: progress difficulty-ramp helper (sketch-then-decide)

- TASK: 20260704-161526
- BRANCH: feat/progress-recipe

## Round 1

- VERDICT: APPROVE

Correct sketch-then-decide outcome, and the task explicitly pre-authorised it
("may downgrade to a doc"; "only build a module if a Level-timer helper proves
substantial and reused"). The sketch's reasoning holds up: of the six games'
ramps, only two are time-based, each is a one-liner (clamp + lerp, optionally an
`EaseFunction`), and the `Level` timer -- while used by two games -- is
implemented divergently (07 recomputes `level_for(elapsed)` and compares; 11
uses a `next_level_at` threshold accumulator) and is too small to earn a module.
Not a copy, not substantial -> a doc, exactly as the task steered.

Doc accuracy checked against the code:
- continuous ramp matches `06_fruitninja` `ramp_t` (`:439`) + `spawn_interval_for`;
- `level_at` matches `07_orbit` `level_for` (`:577`), and the level-up edge
  matches its `advance_level` (`:1087`) compare-and-act;
- the two idioms called out as game-specific are real: `10_asteroids` Wave
  (per-clear) and `09_reactor` `tier_for_score` (`:581`, log-scaled).

Both idioms carry compiling, asserting doctests (ramp normalization + lerp
endpoints; `level_at` boundaries and the level-up edge), so the recipe is tested,
not just prose -- satisfying the task's "unit-test the direction math" spirit for
a doc outcome. Home is right: `meth` (the task's tag), beside `lerp`. The
`EaseFunction`/future-`tween` note is accurate as forward-looking.

Verified in the worktree: `cargo fmt --check`, `cargo clippy --all-targets`,
`cargo test` (67 unit + 36 doctests, the 2 new `meth` ones included) and
`scripts/check-ascii.sh` all pass. Diff is a single file, +66 lines, docs only.

No findings. This closes Wave B of the input/projection harvest (scoring shipped,
radial gravity + progress documented as recipes).
