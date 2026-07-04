# Retro: CI: run example unit tests (cargo test --examples)

- TASK: 20260703-175735 (CLOSED)
- BRANCH: ci/example-tests (squash-merged to master as 0f5d2ca)
- REVIEW ROUNDS: 2 (R1 APPROVE with one MINOR, addressed; R2 APPROVE)

See `tasks/20260703-175735/TASK.md` and `tasks/20260703-175735/REVIEW.md` for
what changed and why. This retro is about how the working went.

## What went well

- The task's own premise was validated the moment it was implemented. Turning on
  `cargo test --examples` immediately failed on a rotted, flaky assertion in
  `10_asteroids` -- exactly the "silent rot" the task predicted a coverage gap
  would hide. Fixing a CI-visibility gap paid for itself on the first run. The
  lesson generalises: when you close a gap that was hiding failures, expect the
  first honest run to surface real ones, and budget for fixing them as part of
  the same task rather than treating them as out of scope.
- Instrumenting beat guessing on the "impossible" failure. My first mental model
  (triangle inequality: shard speed >= |SPLIT_SPEED - drift| = 2.5, so 1.85 is
  impossible) was wrong. Instead of arguing with the number, I added a temporary
  `eprintln!` of every shard velocity and ran it several times. That revealed the
  real mechanism -- `spawn_asteroid` zeroes the burst's z component on the planar
  arena, so an out-of-plane burst leaves only a small planar residual -- which no
  amount of re-reading the code would have made obvious. Print the actual values
  when the math "proves" a failure can't happen; the model is what's wrong.
- Scoped the change to what was needed. Verified up front that no example test is
  `#[cfg(feature = "debug")]`, so the `--features debug` variant the task offered
  as optional would add zero coverage. Skipped it instead of adding a redundant
  ~1-minute CI step, and said so explicitly in the step comment and TASK.md.

## What went wrong

- The rotted assertion was a repeat of the crate's most persistent pattern: a
  test asserting a physical property the code never guaranteed. The original
  `10_asteroids` test claimed each shard "keeps at least the parent's drift",
  written aspirationally from the design intent ("inherits drift + burst")
  without deriving what the code actually produces once `LockedAxes` /
  `with_z(0.0)` drop the out-of-plane burst. This is the same "comment claims
  behaviour X, code doesn't do X" root cause already logged in the
  modding-json-registry and 11_overload retros -- now surfaced a third time,
  in a physics assertion rather than a filter/action call.
- My own first fix (R1.1) under-shot in the mirror-image way: the corrected
  assertion (`|vel - drift| <= SPLIT_SPEED`) was true but passed even if the
  burst term were dropped entirely, so its comment ("plus an outward burst")
  again out-ran what it proved. Caught in review and fixed with a burst-liveness
  assertion. Small, but it shows the honesty rule cuts both ways: an assertion
  that is too weak to fail on the behaviour its comment describes is the same
  defect as one that is simply wrong.

## What to improve next time

- When writing (or repairing) a test that asserts a numeric/physical invariant,
  derive the bound from the code path that actually runs -- including the
  clamps, axis locks and component drops downstream of the formula -- not from
  the design intent in the doc comment. `parent_vel + burst` is not the velocity
  that lands on the entity if `spawn_asteroid` then zeroes a component.
- After correcting an assertion, ask "would this still fail if the behaviour my
  comment describes were removed?" If not, the assertion is too weak; add the
  case that pins the behaviour (here: at least one shard must differ from the
  bare parent drift).

## Action items

- [x] `cargo test --examples` added to CI and AGENTS.md "Build, Verify, Run";
  the 58 example tests (previously compiled-but-never-run) now execute on every
  push.
- [x] Fixed and strengthened the `10_asteroids` split test; verified stable over
  repeated runs.
- No new AGENTS.md rule proposed: the test-honesty convention this reinforces
  is already documented (added after the modding-json-registry retro). This is a
  third data point that the rule is right, not evidence a new rule is needed.
- No follow-up code task: the change is self-contained and merged, and no other
  example test failed once the suite ran.
