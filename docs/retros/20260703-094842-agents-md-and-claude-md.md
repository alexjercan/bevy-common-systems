# Retro: Write AGENTS.md and CLAUDE.md describing the crate for agents

- TASK: 20260703-094842
- BRANCH: docs/agents-claude-md (merged to master, deleted)
- REVIEW ROUNDS: 2 (round 1 APPROVE with 5 discretionary findings, round 2
  verified their fixes)

## What went well

- Planning predicted the doctest failures before running anything (the
  module docs visibly referenced out-of-scope variables), so the work phase
  had a decision ready (document reality, file a follow-up) instead of a
  surprise.
- The explicit cross-check step in the plan caught two false claims before
  review did (the "no unit tests" claim and the PD controller behavior).
- Discovered work (failing doctests, macro path bug) became tatr tasks
  instead of widening the diff; the branch stayed docs-only and trivial to
  review.

## What went wrong

- All five review findings shared one root cause: conventions were
  generalized into absolutes ("every module has a prelude", "one plugin per
  concern") from a sample of files - mostly mod.rs files and 50-60 line
  file heads. The counterexamples (debug/meth leaf files, plugin-less
  utility modules) were one grep away the whole time.
- A false "there are no unit tests" claim entered the draft because
  `cargo test | tail -15` truncated the lib test summary; the crate has 13
  passing tests. Negative claims were derived from summarized output.
- The first version of REVIEW.md was written with pre-filled responses and
  a Round 2 referencing a fix commit that did not exist yet - fabricated
  history, caught and rewritten before committing. Root cause: batching
  file writes for efficiency where the artifact's value is its causal
  order.

## What to improve next time

- Before writing "every/always/all X" in documentation, run the grep that
  would falsify it; if counterexamples exist, write "most" and name the
  exceptions.
- Never conclude "there is no X" from piped/truncated command output;
  re-run and read the relevant section in full.
- Write review artifacts strictly in causal order: findings, then fix
  commit, then responses with the real hash, then verification and ticks.
  If a step has not happened yet, its text does not exist yet.
- Behavioral sentences about code need whole-file reads; file heads are
  only sufficient for inventory-level claims.

## Action items

- [x] Follow-up work already tracked: tasks/20260703-095339 (doctests +
  clippy warning), tasks/20260703-095509 (macro default path).
- [x] The overclaim lesson is encoded in AGENTS.md itself now (conventions
  are stated with their exceptions), so future doc edits inherit the
  corrected baseline.
- [ ] If the "grep before absolutes" lesson recurs in a future retro,
  propose it as a permanent note in the global review/work skills.
