# Retro: Fruit ninja touchscreen support via bevy_enhanced_input

- TASK: 20260703-173128
- BRANCH: feature/fruitninja-touch (merged to master, f1da966)
- REVIEW ROUNDS: 1 (APPROVE first round)

See TASK.md close-out and tasks/20260703-173128/NOTES.md for the
what/why of the change; this retro is only about how the working went.

## What went well

- Read the dependency source before designing. `bevy_enhanced_input` 0.26 has
  no native touch binding; grepping its `Binding` enum and finding the
  `CustomInputs` / `Binding::Custom` extension point (with its own trackpad
  example) up front meant the design was right the first time instead of after
  a failed attempt to bind touch directly.
- Verified the one risky assumption at review time against the crate's own
  state-transition table (events.rs), not from memory: a conditionless bool
  binding goes `None -> Fired` (START|FIRE) on press and `Fired -> None`
  (COMPLETE) on release. That is exactly what the Start/Complete observers rely
  on, so `just_pressed`/`pressed` are provably correct without a runtime test.
- Heeded the two standing gotchas from prior retros: verified the web build
  through the real entry point (`web/scripts/build-games.sh`), and checked
  build pass/fail via redirect + `$?`, never a piped `| tail`. Both paid off:
  the wasm build genuinely compiled and packaged.
- One clean review round. The diff delivered the goal, checks were green, and
  the only findings were a MINOR (verification gap) and a NIT (multi-touch),
  neither blocking. No rework.

## What went wrong

- CI blind spot discovered, not caused: plain `cargo test` does not run
  `examples/` in-file tests, so the 3 new tests (and 16 pre-existing ones in
  06_fruitninja) never run in CI. Root cause is a pre-existing CI config gap;
  it only surfaced because I explicitly ran `cargo test --example` to confirm
  my new tests, rather than trusting the green `cargo test`. Lesson: when you
  add tests, run the exact command that will execute them, not just the suite
  that is "supposed to".
- No interactive playtest possible: this cycle ran headless, so the ECS input
  wiring was verified only by compile + source-reading, never by an actual
  click or swipe. For an input feature that is a real gap; mitigated by the
  state-machine verification and by handing the playtest to the user (R1.1),
  but worth naming.
- Stale orientation doc cost a small detour: AGENTS.md says enhanced_input
  0.25; Cargo.toml is 0.26. Caught immediately by checking Cargo.toml before
  trusting the doc, but it is a reminder that the module map lags the manifest.
- Tooling friction: `tatr new` uses second-resolution timestamp IDs, so two
  `new` calls in the same second collided and the second silently overwrote the
  first. Lost one follow-up task until I noticed the duplicate ID. Space out
  `tatr new` calls, or verify each returns a distinct ID.

## What to improve next time

- For any input/interaction feature verified headless, state the
  interactive-playtest gap explicitly in the review and hand a concrete
  click-through checklist to the user (done here).
- Trust the manifest over the orientation doc for versions; check Cargo.toml
  first.
- When creating multiple tatr tasks in one go, confirm distinct IDs (or pause a
  second between `tatr new` calls).

## Action items

- [ ] tatr 20260703-175735: add `cargo test --examples` to CI so example unit
  tests actually run (follow-up).
- [ ] tatr 20260703-175719: bump enhanced_input 0.25 -> 0.26 in AGENTS.md
  (follow-up).
- [ ] Possible AGENTS.md workflow note: `tatr new` IDs are second-resolution;
  creating several in a row can collide. Left as a retro observation for now
  rather than a doc edit, pending whether it recurs.
