# Review: 07_orbit hazard-hit impact feedback

- TASK: 20260703-214927
- BRANCH: feature/07-orbit-polish

## Round 1

- VERDICT: REQUEST_CHANGES

Verified in the worktree: `cargo fmt --check`, `cargo clippy --all-targets`
(and `--features debug`), `cargo test --example 07_orbit` (14 pass), and
`./scripts/check-ascii.sh` all green. The core design is right: the shake system
is ordered `.after(ChaseCameraSystems::Sync)` and adds its offset (the chase
sync sets `translation` absolutely each frame, confirmed in
`src/camera/chase.rs:234`, so the additive jitter cannot accumulate), trauma is
reset in `start_game`, and the pure decay/clamp helpers are unit-tested. Two
findings.

- [ ] R1.1 (MINOR) examples/07_orbit.rs:1089-1096 - the hazard-hit branch also
  resets the streak (`streak.count = 0`), which is a sensible gameplay coupling
  (a hit costs your momentum) but is undocumented: the module `//!` streak
  description and the T1 review say nothing about a hit breaking the streak, so
  a reader would not expect it. Suggested change: add one clause to the module
  doc's streak sentence noting a hazard hit breaks the streak, so the behavior
  is honest and discoverable.
  - Response: Fixed. Added a clause to the module `//!` streak sentence: "Taking
    a hazard hit breaks the streak, so a clean run is worth chasing." Re-ran
    fmt/tests/ascii, green.
- [ ] R1.2 (NIT) examples/07_orbit.rs:1050-1063 - the damage overlay is
  `DespawnOnExit(Playing)` and `fade_damage_flash` is gated to `Playing`, so a
  *fatal* hit's flash is effectively invisible (the state flips to GameOver
  before the next fade tick). Non-fatal hits flash correctly, and the game-over
  screen has its own red, so this is acceptable; noting it so it is a conscious
  choice, not an oversight. No change required.
  - Response: Acknowledged, left as-is by design. The fatal hit already hands
    off to the game-over screen's own red; a non-fatal hit (the common case)
    flashes fully.

## Round 2

- VERDICT: APPROVE

- [x] R1.1 resolved: module doc now states a hazard hit breaks the streak.
  Verified in the diff; suite green.
- R1.2 left as a NIT by design (fatal-hit flash defers to the game-over screen);
  non-blocking.

No new findings. The diff delivers the Goal: a decaying camera shake layered
after the chase-camera sync (additive, no drift, proven by reading the sync) and
a red damage flash on hazard hits, with unit-tested trauma helpers.
