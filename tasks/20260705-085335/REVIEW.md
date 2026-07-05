# Review: Bastion enemies in packs + difficulty ramp

- TASK: 20260705-085335
- BRANCH: feature/bastion-wave-packs

## Round 1

- VERDICT: APPROVE

Reviewed `git diff master...HEAD`. The pack scheduler is correct and delivers the
Goal:

- `advance_waves` opens a wave with `pack_size`/`packs_left`, releases a full
  pack when `pack_timer` elapses (first pack immediate via `pack_timer = 0.0`),
  then `PACK_GAP` between packs and `WAVE_GAP` (after field-clear) between waves.
  No stall risk: `pack_size(n) >= 3` and `packs_in_wave(n) >= 2` always, so a
  wave never has zero packs/size.
- Difficulty ramps on multiple axes (pack size, pack count, hp, speed); total
  enemies go super-linear (6/30/88 at waves 1/5/10).
- Pure helpers extracted and both new tests are MEANINGFUL: `pack_size(1) ==
  PACK_SIZE_BASE`, growth asserts, and the super-linear projection test would
  fail for a constant-increment schedule. The old tautology-adjacent
  `wave_size_grows` is replaced.
- Wave-start shake correctly deferred to the juice task (documented in-code), so
  no double-add.

Checks re-run in the worktree: clippy `--all-targets` clean, `fmt --check` clean,
`check-ascii` clean, `cargo test --examples` green. Observable-effect verified
(temp log showed two packs of 3 spawning together); built-binary autopilot run
completed the full cycle with no panic.

Findings (both non-blocking):

- [x] R1.1 (MINOR) examples/12_bastion.rs:1849 - the `wave_size` doc says the
  total "ramps super-linearly (each wave's jump is at least as big as the
  previous one)". The parenthetical is false: floored `pack_size` growth makes
  adjacent jumps jitter (e.g. wave 4->5 gains only a pack, not size, so its jump
  is smaller than wave 1->2). This is exactly the monotonicity the ramp test
  deliberately avoids asserting, and the repo has a standing lesson against
  shipping doc claims the code does not honour. Fix the doc to match reality.
  - Response:
- [x] R1.2 (NIT) examples/12_bastion.rs:1843 - `packs_in_wave(n)` computes
  `(n - 1)` on a `usize`; it is only ever called with `n >= 1`, but a stray
  `packs_in_wave(0)` would panic on underflow in debug. Consider
  `n.saturating_sub(1)` for defensiveness. Take it or leave it.
  - Response: Done -- switched to `n.saturating_sub(1)`.
