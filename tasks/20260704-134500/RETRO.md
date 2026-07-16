# Retro: camera/shake CameraShake trauma module

- TASK: 20260704-134500
- BRANCH: worktree-camera-shake (squash-merged to master as e4fdec3)
- REVIEW ROUNDS: 2 (APPROVE round 1 with findings, APPROVE round 2 after fixes)

See TASK.md and `tasks/20260704-134500/NOTES.md` for what changed and
why; this retro is about how the cycle went.

## What went well

- Read all four example hand-rolled shake copies (06/07/08/10) before designing
  instead of copying one. That surfaced the key insight the spike only hinted
  at: each example dodged the drift bug *differently* (06 wrote `= base +
  offset` from a const, 07/08 relied on the chase camera overwriting the base,
  10 let a fit system own the base). Seeing all four made it obvious no
  single-strategy copy would compose, which is what produced the Restore/Apply
  two-phase design rather than a fourth ad-hoc variant.
- Applied the standing "test the hard half" lesson up front: the anti-drift
  property is the module's entire reason to exist, so it got a direct ECS
  regression test (re-center after repeated kicks), not just pure-math unit
  tests. That test is now the guard against the exact bug the asteroids retro
  recorded.
- Used an independent reviewer subagent for fresh eyes rather than self-marking
  the diff.

## What went wrong

- R1.4 (MAJOR, self-caught): ordered `Restore` only `.before(
  ChaseCameraSystems::Sync)` and `Apply` `.after` it, assuming that transitively
  ordered Restore before Apply. It does not when the chase plugin is absent (06
  and every static-camera game): an empty `Sync` set provides no ordering edge,
  leaving two `Transform`-writing systems unordered. Root cause: treated
  "order against set X" as "order before/after everything, including the gap at
  X", when it actually means "order relative to X's *members*" -- vacuous when X
  is empty. The initial anti-drift test passed anyway, by the executor's
  insertion-order ambiguity resolution, which *masked* the gap. Both the initial
  test and the independent reviewer accepted "tests pass" as proof; the bug was
  only found by manually tracing the empty-set case.
- The module's headline "composes with a moving base (chase / custom driver)"
  claim had no test (R1.2). Root cause: the one refactored example (06) is a
  static camera, so the tested path and the proven-by-refactor path were both
  the *easy* (static) case; the harder composition case the module advertises
  went unexercised until the reviewer flagged it. A related honesty gap only
  surfaced while writing the follow-up task: because both Restore and Apply live
  in `PostUpdate`, the "any driver" claim really means "any driver that runs
  between the two sets" -- a base writer in `Update` (like 10's `fit_camera`)
  runs before Restore and would break. Fine for v1 (chase + static), but the
  doc slightly over-claims; captured as a decision point in the port task.

## What to improve next time

- In Bevy, when correctness depends on system A running before system B, encode
  that with a direct edge (`configure_sets(A.before(B))` or `chain()`), never by
  ordering both against a third set that may be empty. Ordering against a
  SystemSet only orders you against its current members.
- Do not trust a green test for a property that depends on system *ordering*:
  ambiguous ordering is resolved deterministically per build, so a passing test
  can hide an unpinned order. Pin the order, then the test guards it.
- When a module claims to support a composition (here: a moving base driver)
  that none of the refactored examples exercise, add a test that stands up a
  dummy of that collaborator. The refactor is a test of the case it uses, not of
  the cases it merely advertises.

## Action items

- [x] Added AGENTS.md gotcha: ordering against a possibly-empty SystemSet gives
  no guarantee; pin inter-system order explicitly.
- [ ] tatr 20260704-144509: port 07/08/10 onto `camera/shake` and delete their
  local copies (also resolves the Update-vs-PostUpdate driver caveat for 10).
  Deferred intentionally so this task stayed scoped to the module plus one
  proof-of-use refactor.
