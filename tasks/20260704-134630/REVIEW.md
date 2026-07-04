# Review: tween - narrow duration-based Tween<T>

- TASK: 20260704-134630
- BRANCH: feat/tween

## Round 1

- VERDICT: APPROVE

A well-designed, genuinely narrow foundation. The three planning decisions are
all resolved soundly and stated in the code/commit:
- **Hand-roll over `bevy_tweening`**: correct for a "fully copy-pastable,
  zero-such-deps" crate on bevy 0.19; a mature external dep would be a poor fit.
- **Component-tween, Output-style** (like `transform/*`): `Tween<T>` owns
  timing/easing/completion and exposes `value()`; the game applies it. This
  sidesteps the lens machinery and the `Tween<Vec3>`-scale-vs-translation target
  collision that a "built-in adapters" design would hit, while matching the
  crate's own established output pattern. The cleanest available shape.
- **Home in top-level `tween/` not `meth`**: right call despite the `meth` tag --
  it ships a plugin + component + marker (runtime behaviour), which the
  convention puts in its own module, not pure-`meth`.

Correctness:
- Good catch on the zero-duration bug: a tween with `duration == 0` is
  `finished()` from the start, so the old `if finished { continue }` guard would
  have skipped it forever (never firing its policy), contradicting the doc. The
  `completed` flag fixes it and is what the three ECS policy tests exercise.
- 06 `SlicePop` refactor is behaviour-preserving: `Tween<Vec3>(base, base*1.45,
  SLICE_POP_TIME, Linear)` reproduces the old `base*(1 + 0.45*progress)` linear
  ramp exactly, and the `On<Add, TweenFinished>` observer restores `tween.start`
  (= base) and bursts, matching the old restore-then-explode. Under either
  observer/`apply_slice_pop` ordering the fruit bursts at base scale, because the
  observer removes `SlicePop`+`Tween<Vec3>` so `apply_slice_pop` no longer
  matches it. `apply_slice_pop.after(TweenSystems::Advance)` is a real edge
  (the set has four unconditional `advance_tween::<T>` members), not the
  empty-set ordering trap.

Tests are meaningful: 6 pure (fraction, value, easing-bends-off-linear,
componentwise Vec3, clamp, zero-duration) plus 3 that drive the plugin headlessly
and assert each completion policy's ECS outcome (marker inserted, tween
removed/kept, entity despawned). Notably the ECS path IS testable here (only
`MinimalPlugins` needed), so unlike the enhanced-input bridge this got real
committed coverage. Verified 06 runs the full autopilot cycle
(`Menu -> Playing -> GameOver, no panic`).

Verified in the worktree: `cargo fmt --check`, `cargo clippy --all-targets`
(default and `--features debug`), `cargo test` (76 unit + 38 doctests) and
`scripts/check-ascii.sh` all pass.

- [ ] R1.1 (MINOR) src/tween/mod.rs - the task's motivation is that `ui/popup`
  (rise/fade) and `feedback` (flash decay) "should both consume this, so it is a
  foundation, not a leaf", and the module doc lists "a popup fade" as an example.
  But only 06's `SlicePop` was refactored; popup/flash are not yet consumers, so
  the foundation claim is asserted, not demonstrated. The acceptance bar ("prove
  by refactoring one example") IS met, so this is not blocking -- but file a
  follow-up to refactor `ui/popup`'s fade (its clean `base_alpha -> 0` A->B, the
  rise stays velocity-based) onto `Tween<f32>`, to make the foundation real.
  - Response: Filed as follow-up tatr 20260704-201801 (route popup fade and
    feedback decay onto `Tween`), per the flow "new work becomes a new task"
    rule. The acceptance bar (prove on one example) is met by 06; this realizes
    the foundation claim next.
- [x] R1.2 (NIT) src/tween/mod.rs:120 - `Tween<T>` does not derive `Reflect`
  (the crate generally reflects components), because it holds an `EaseFunction`
  and is generic. Acceptable given those constraints; `TweenFinished` and
  `TweenOnComplete` are reflected and registered. Leave as-is unless a reason to
  inspect live tweens appears.
  - Response: Left as-is by design. `EaseFunction` reflection plus the generic
    bounds make a `Tween<T>` `Reflect` derive not worth the churn; the marker and
    policy enum carry the inspectable state.
