# Retro: Replace non-ASCII typographic chars in src/camera/chase.rs docs

- TASK: 20260703-101712
- BRANCH: fix/chase-ascii (merged to master, deleted)
- REVIEW ROUNDS: 1 (APPROVE, no findings)

See TASK.md close-out for the change; this is process only.

## What went well

- Single-purpose diff: only the offending codepoints changed, alignment
  preserved, verified by a whole-tree scan before and after. A three-line
  change got a three-line review and a one-round APPROVE, as it should.
- The issue was already isolated and scoped by the prior task
  (20260703-095339) that discovered it and deferred it rather than
  smuggling the fix into an unrelated diff - so this task started with an
  exact location and a verification command already written.

## What went wrong

- Nothing in execution. The only real observation is upstream: these
  characters existed in the initial commit and were only caught when a
  later task happened to grep the tree. Nothing in the workflow would have
  caught them at authoring time.

## What to improve next time

- Prevent this class reactively-cleaning it: a mechanical non-ASCII check
  (pre-commit hook or CI grep `grep -rnP '[^\x00-\x7F]' src/ examples/`)
  would enforce the plain-ASCII rule AGENTS.md already states, instead of
  relying on a human noticing. Filed as a backlog candidate below.

## Action items

- [x] Filed follow-up task 20260703-103940: add a non-ASCII guard
  (pre-commit or CI) enforcing the plain-ASCII rule. Left OPEN in the
  backlog for the user to prioritize; not implemented in this flow because
  it is outside the stated goal (clear the two existing tasks).
