# Review: breach -- points + combos

- VERDICT: APPROVE
- ROUNDS: 1

## Verified

- `cargo fmt --check`, `cargo clippy --all-targets` (clean, only the transitive
  proc-macro-error2 future-incompat note), `cargo test --example 14_breach`
  (8 pass), `scripts/check-ascii.sh` (clean).
- Headless `BCS_AUTOPILOT=1 --features debug` run: full Menu->Playing->GameOver
  cycle, no panic, no despawn-race errors; persisted best jumped to 280 points
  (combo-scaled), proving kills accrue streak-multiplied points end to end.
- Real windowed run reached the render loop (X11 window opened).

## Findings

- No correctness issues found.
- Checked the two failure modes the retros warn about:
  - Popup lifecycle: `PopupPlugin` self-adds `TweenPlugin`, so "+N"/banner popups
    fade and despawn (no leak), and are scoped `DespawnOnExit(Playing)`.
  - Testability: the death observer stays UI-free (fills `KillFeed` + bumps the
    `Streak`); the streak-scaled scoring and lapse are unit-tested off the ECS
    (`chained_kills_multiply_by_the_streak`, `the_streak_lapses_after_its_window`),
    not trusted to the autopilot (which force-transitions on a timer).
- Score changed from tuple to struct `{ points, kills }`; grep confirms no stale
  `score.0` / `Score(` references remain.

## Nits (non-blocking)

- The kill "+N", combo banner and live readout occupy nearby central regions; a
  dense multi-kill could visually crowd. Cosmetic, acceptable for an example.
