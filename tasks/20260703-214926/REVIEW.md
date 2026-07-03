# Review: 07_orbit orb-pickup streak juice

- TASK: 20260703-214926
- BRANCH: feature/07-orbit-polish

## Round 1

- VERDICT: REQUEST_CHANGES

Verified: `cargo fmt --check`, `cargo clippy --all-targets`,
`cargo test --example 07_orbit` (13 pass, was 11), `./scripts/check-ascii.sh`
all green in the worktree. The score-model change (Score now banks orb points,
not a raw count) is consistent across every use site (`start_game`,
`resolve_collisions`, `update_hud`, `record_high_score`, `spawn_game_over`) and
`score_value`'s new semantics are unit-tested. combo.wav reuse and the
web-copy-dir claim check out. Two findings, both about the banner popup.

- [ ] R1.1 (MINOR) examples/07_orbit.rs:997,1064 - `spawn_streak_banner` is
  called on every pickup at streak x2+, and every banner is spawned at the same
  fixed `top: Val::Px(130.0)`. During a fast chain (pickups < the 0.8 s popup
  lifetime apart, which is exactly when a streak is hot) the "STREAK x2",
  "STREAK x3", ... popups overprint each other at the same y, which reads as
  messy rather than punchy. Suggested change: keep at most one banner alive -
  mark it with a dedicated component (e.g. `StreakBanner`) and despawn any
  existing one before spawning the new one, so the banner just updates to the
  current streak count in place.
  - Response: Fixed. Added a `StreakBanner` marker component; before spawning a
    new banner `resolve_collisions` despawns any live one, so at most one banner
    exists and it updates to the current streak count in place. Re-ran the full
    suite (fmt/clippy/13 tests/ascii), still green.
- [ ] R1.2 (NIT) examples/07_orbit.rs:1088-1092 - the banner centers via a
  full-width node plus `TextLayout { justify: Justify::Center }`. That is the
  idiomatic approach, but it is unverified headlessly; confirm on a play-test
  that the banner actually sits centered (fall back to `06_fruitninja`'s
  compute-x-from-window-width approach if it does not).
  - Response: Left as-is (NIT). `Justify::Center` on a `width: 100%` text node
    is the standard Bevy way to center a single line across the screen; flagged
    for visual confirmation on the play-test task, not worth pre-emptively
    swapping to manual x-math. Now that there is a single banner, if it needs a
    tweak it is a one-line change.

## Round 2

- VERDICT: APPROVE

- [x] R1.1 resolved: verified only one `StreakBanner` can exist (despawn-before-
  spawn in `resolve_collisions`), banner updates in place. Suite green.
- R1.2 left open as a NIT (visual confirmation deferred to the play-test task);
  non-blocking per the review policy.

No new findings introduced by the round-1 fix. The diff delivers the task Goal:
streak scoring, rising-pitch pickup + reused combo chime, floating "+N" popups
and a single in-place "STREAK xN" banner, all with unit-tested pure helpers.
