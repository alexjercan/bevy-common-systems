# Review: spawn + time/cooldown

- TASK: 20260704-134730
- BRANCH: feat/spawn-cooldown

## Round 1

- VERDICT: APPROVE

The sketch-then-commit split is the right one and each half is well argued.

**Dropping `spawn` is correct.** 06's spawner is a plain `Timer` in `Repeating`
mode with `set_duration` for the difficulty ramp -- the raw `Timer` already is
the primitive, and no example uses *temporal* jitter (their spawn randomness is
positional). A `Spawner { interval, jitter }` would be a thin wrapper, so it
lands as a documented recipe in the `time` module doc (with a worked pointer to
`06_fruitninja::spawn_projectile`). Clean negative result.

**Shipping `Cooldown` is justified over a raw `Timer`.** The concrete win is the
starts-ready semantics: a fresh `Cooldown` is `ready()` (fire immediately),
whereas a fresh `Timer` in `Once` mode is *not* finished -- so a weapon built on
one would spawn unable to fire, a real footgun. The `trigger()`/`ready()`/
`started()` API reads for the domain where `reset()`/`finished()` reads
backwards, and it dedups the TWO cooldowns 10_asteroids hand-rolled as bare
`f32` fields. It is a plain value deriving `Component` with no plugin, which is
the right call (a ship carries several cooldowns, so a blanket auto-tick would
not fit) and matches the `scoring::Streak` precedent.

Behaviour-equivalence of the 10 refactor is exact: `Cooldown::new(FIRE)` starts
ready == old `fire_cooldown: 0.0`; `Cooldown::started(INVULN)` starts on cooldown
== old `invuln: INVULN_TIME`; `.ready()` == `<= 0.0`; `.trigger()` == `= DURATION`;
`.tick(dt)` == the manual `(x - dt).max(0.0)`. No `f32` leftovers.

Verification is end-to-end, not just a boot: the autopilot held Space every frame
and 10 fired ~13 rate-limited shots over ~3s of Playing (`FIRE_COOLDOWN = 0.16`
caps it near 6/s; an ungated fire would be ~180), Menu -> Playing -> GameOver,
no panic. That directly exercises the `ready()`/`trigger()`/`tick()` gate.

Tests are meaningful: 5 `Cooldown` unit tests (starts-ready, trigger, tick-and-
clamp, `started`, `trigger_for` clamping) plus the recipe doctest. Naming does
not collide with `bevy::prelude` (clean `clippy --all-targets` with both globs).

Verified in the worktree: `cargo fmt --check`, `cargo clippy --all-targets`
(default and `--features debug`), `cargo test` (85 unit + 43 doctests),
`cargo test --examples` and `scripts/check-ascii.sh` all pass.

No findings.
