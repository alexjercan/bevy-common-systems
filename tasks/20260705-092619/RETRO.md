# Retro: 12_bastion enemies in packs + difficulty ramp

- TASK: 20260705-085335
- BRANCH: feature/bastion-wave-packs (squash-merged), plus a follow-up
  fix/bastion-wave-size-hud
- REVIEW ROUNDS: 1 (APPROVE with 1 MINOR + 1 NIT, both fixed in-round)

Second task of the 12_bastion polish flow. Reworked the wave scheduler from a
one-at-a-time trickle to packs, and steepened the difficulty ramp.

## What went well

- Extracted the scheduling math into pure functions (`pack_size`,
  `packs_in_wave`, `wave_size`) and wrote tests that actually bite. The
  super-linear ramp test is a wide-range linear-projection check (wave 10 must
  overshoot the wave 1->2 slope projected out), which I chose *after* computing
  the values by hand and finding that an adjacent-increment comparison would
  falsely fail -- floored `pack_size` growth makes neighbouring jumps jitter
  (wave 4->5 gains only a pack, not size). Doing the arithmetic before writing
  the assertion saved a red test. This is the "test the formula, not a
  tautology" lesson from the prior bastion retro applied deliberately.
- Verified the observable effect, not a proxy (the follow-up retro's headline
  lesson): a temp log proved wave 1 released two packs of 3 spawning together,
  and the built-binary autopilot completed the full cycle. When the first
  `cargo run` autopilot timed out, I recognised it as shader-warmup eating the
  timeout budget (not a logic hang) and confirmed with a second run of the
  pre-built binary rather than chasing a phantom bug.

## What went wrong

- Shipped a dead-code warning to master. After the rework, `wave_size()` was
  referenced only by the `#[cfg(test)]` module, so a plain non-test example
  build warned "function `wave_size` is never used" -- but I gated verification
  on `cargo clippy --all-targets` and `cargo test --examples`, BOTH of which
  compile the test module and therefore see `wave_size` as used. The warning was
  invisible to every check I ran and only surfaced from the editor diagnostic
  after merge. Root cause: `--all-targets` is the documented compile gate for
  *catching* example errors, but it is the wrong gate for *dead-code that is only
  alive under cfg(test)* -- it is the exact inverse of the "bare cargo build
  hides example errors" gotcha. Fixed in a follow-up branch by using `wave_size`
  in the HUD (an incoming-count readout), which resolves the warning with real
  player value rather than an `#[allow]`.

## What to improve next time

- When a task turns a previously game-used helper into a test-only one, run a
  plain `cargo build --example NN` (no `--all-targets`, no tests) as an extra
  gate before merging. `--all-targets` and `cargo test` both mask
  cfg(test)-only dead code because they compile the tests that keep it alive.
  Add this to the verification checklist for any refactor that removes a
  function's production call sites.
- Prefer resolving a genuine "unused after refactor" by finding the real use
  (or deleting the function) over `#[allow(dead_code)]`; here the HUD use was
  both the fix and a small feature.

## Notes for the next task (juice, 20260705-085338)

- The wave-start camera shake was deliberately left out of this task and is
  owned by the juice task; there is a `NOTE:` marker at the `Sfx::Wave` play
  site in `advance_waves`.
