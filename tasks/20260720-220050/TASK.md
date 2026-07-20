# lessons: promote or retire 14 pending x3+ lessons

- STATUS: IN_PROGRESS
- PRIORITY: 70
- TAGS: chore

## Story

As the maintainer, I want a single decision pass over the 14 pending
promotions (x3+) in LESSONS.md, so that the accumulated wisdom the ledger was
designed to surface actually gets folded into a guideline, template, or tool -
or is explicitly retired. This is the largest promotion backlog across all six
repos.

## Steps

- [x] Reviewed all 24 (not 14) x3+ pending lessons (evidence-before-claim x9, verify-api-in-source x8, run-the-example x7, reset-shared-state x7, full-command-output x6, verify-observable-effect x6, grep-whole-tree-before-rename x5, regression-test-must-fail x4, clippy-all-targets-gate x4, sprout-first x4, pkill-by-pid x4, negative-result-is-a-deliverable x4, one-tatr-new-per-call x3, no-concurrent-git-same-tree x3, split-verifiable-from-manual x3).
- [x] For each: promoted (to AGENTS.md / a skill / a tool/guard) or retire, and record the disposition.
- [x] Annotated promoted entries with the promotion marker so `tatr check --ledger` stops flagging them.

## Definition of Done

- Every x3+ lesson is annotated promoted or retired; no `promotion-stalled` findings (cmd: `tatr check --ledger LESSONS.md 2>&1 | grep -c promotion-stalled` -> 0). (The 3 remaining `closed-unchecked` findings are pre-existing and are handled by task 20260720-220102.)
- The 5 freshly-folded domain lessons are present in AGENTS.md (cmd: `grep -c "Promoted ledger lessons" AGENTS.md`).

## Notes

- Several are strong skill-guidance candidates (sprout-first, evidence-before-claim, verify-api-in-source, run-the-example).
- Where a lesson maps to a generic skill fix, consider filing it against nix.dotfiles instead of duplicating locally.
