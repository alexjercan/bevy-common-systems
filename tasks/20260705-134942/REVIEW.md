# Review: Expand README.md into a proper crate README

- TASK: 20260705-134942
- BRANCH: docs/readme-expand

## Round 1

- VERDICT: REQUEST_CHANGES

- [x] R1.1 (MAJOR) README.md:113-115 - the `scoring` module description invents
  type names. It reads "a `Score` + `Combo`/streak-decay resource and a generic
  `HighScore<T>` best-score resource", but `src/scoring/` has NO `Score` and NO
  `Combo` type. The public types are `Streak` (`src/scoring/streak.rs:64`, a
  counter that grows on each hit and decays when the player goes quiet) and
  `HighScore<T>` (`src/scoring/high_score.rs:46`). "Combo" appears only inside
  streak.rs doc comments as the game-local pattern `Streak` replaces. This
  directly violates the task's headline acceptance criterion ("Every claim must
  be cross-checked against the actual code ... nothing invented"), and it is in
  the public-facing deliverable. Fix: describe it as the real API, e.g. "a
  `Streak` counter that grows on hits and decays when the player goes quiet,
  and a generic `HighScore<T>` best-score resource with a 'new best' edge."
  - Response: Fixed (README.md:107-109). Rewrote the scoring line to name the
    real types only -- `Streak` (grows on each hit, decays when quiet) and
    `HighScore<T>` (with the "new best" edge) -- wording taken from
    src/scoring/mod.rs's own doc. No `Score`/`Combo` names remain.

### Verified accurate (no action)

Spot-checked against the code and confirmed correct, for the record:
- Versions: "Bevy 0.19 and avian3d 0.7" match `Cargo.toml` (bevy 0.19.0,
  avian3d 0.7). Feature flags `debug`/`dev` match `[features]`.
- License line "MIT" matches `LICENSE`.
- Quickstart snippet: `HealthPlugin`, `Health::new(100.0)`, and the
  `HealthApplyDamage { entity, source, amount }` shape all match
  `src/health/mod.rs` (adapted from its own tested doc example).
- Module map matches `src/lib.rs`: audio, camera (chase/post/skybox/wasd/
  shake/project), debug, feedback, health, helpers, input, material, mesh,
  meth, modding, persist, physics (pd_controller/doom_controller), time,
  transform, tween, ui (status/animate/menu/popup/touchpad). modding prelude
  exports match; persist native-file/wasm-localStorage claim matches
  `src/persist/backend.rs`; `glowing_material` exists (`src/material.rs:39`).
- Examples gallery: all 14 files exist (`01`..`14`); `05_explode` is indeed
  triggered by `MouseButton::Left` (`examples/05_explode.rs:126`) as stated.
- README.md is pure ASCII; dependency git URL matches `git remote`.

## Round 2

- VERDICT: APPROVE

- R1.1 verified fixed (README.md:107-109): scoring now names only the real
  types, `Streak` and `HighScore<T>`; no `Score`/`Combo` names remain anywhere
  in the README. No new findings introduced by the change. The README is
  accurate, complete against the task Goal, and plain ASCII. Approved.
