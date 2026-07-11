# Retro: backward-Euler PD conditioning - premise falsified

- TASK: 20260711-094942
- BRANCH: fix/pd-backward-euler-gains (squashed to master as d9e13e1)
- REVIEW ROUNDS: 2 (R1: 1 MAJOR on an overclaimed test; R2: APPROVE)

## What went well

- The plan's first step was "write the failing repro" and the repro
  refused to fail - the cheapest possible way to kill a wrong theory.
  Three graded experiments (planned repro, transplanted exact state,
  cargo-path-patch A/B against the fixed dependency) each cost minutes
  and the last one was decisive.
- Closing a task with "no code change, premise falsified" was treated as
  a legitimate outcome with a full evidence trail, not a failure to be
  papered over with speculative code.

## What went wrong

- The task existed at all because the previous cycle's diagnosis
  (nova-side trace) was read against the UNFIXED dependency: the
  period-2 flip-flop was a faithful description of the terminal state
  but not of its cause. Root cause of the wasted plan: designing fix #2
  before A/B-ing fix #1 - the path patch that settled it in five minutes
  was available the whole time.
- R1.1: the close-out claimed the new saturation test "discriminates the
  frame fix" without running it against the old order; the review probe
  (one sed revert + one test run) showed it does not. Same overclaim
  shape as the previous cycle's R1.1 - a test asserted to cover
  something it does not.

## What to improve next time

- When a dependency fix has just landed and a downstream symptom
  persists in a trace gathered BEFORE that fix, re-gather the trace
  against the fixed dependency (cargo `[patch]` with a path) before
  theorizing new mechanisms, let alone planning new tasks.
- Any sentence of the form "this test fails on the old code / covers X"
  goes with actually flipping the code to the old state and running it -
  the check costs one revert and one test run. Second occurrence of the
  overclaimed-coverage lesson in two cycles; if it appears again it
  belongs in the review skill checklist.

## Action items

- [x] R1.1 fixed in-round.
- [ ] nova task 20260709-125640: finish downstream (rev bump, tighten the
      release-spin guard to < 0.5, remove the diagnostic test).
