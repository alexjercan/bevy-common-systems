# retro-completeness: audit CLOSED tasks lacking RETRO

- STATUS: OPEN
- PRIORITY: 30
- TAGS: chore

## Story

As the maintainer, I want the ~23 CLOSED tasks lacking RETRO.md audited, so
that each is either backfilled with a lightweight retro (if lessons are still
recoverable) or marked historical. Currently retro coverage is ~65%.

## Steps

- [ ] List CLOSED tasks without RETRO.md.
- [ ] For each: if lessons are still recoverable, write a brief retro; otherwise apply the historical marker (per tatr task #5's mechanism).
- [ ] Confirm `tatr check -S` clean.

## Definition of Done

- Every CLOSED task either has a RETRO.md or a historical marker (cmd: `tatr check -S`).

## Notes

- Depends on tatr task #5 (historical/no-retro recognition).
- Also review the 4 tasks lacking both RETRO and REVIEW (SUPERSEDED/deferred) as archive stubs.
