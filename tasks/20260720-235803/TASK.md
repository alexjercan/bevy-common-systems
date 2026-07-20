# tick 094942's dropped steps so tatr check is clean

- STATUS: IN_PROGRESS
- PRIORITY: 40
- TAGS: chore

## Story

As a maintainer, I want the 3 deliberately-dropped Steps on CLOSED task
20260711-094942 marked accounted-for (`- [x]`, drop reason preserved) so
`tatr check` is clean. 094942 is a real flow task (has REVIEW + RETRO) whose
steps were dropped ("premise falsified" / "no behavior change shipped"), so the
historical exemption correctly does not apply - the honest fix is to tick the
dropped boxes, not tag it historical.

## Steps

- [x] Flip the 3 `- [ ]` dropped steps in tasks/20260711-094942/TASK.md to `- [x]`, keeping the inline "(dropped, ...)" reasons verbatim.
- [x] Confirm `tatr check` is clean.

## Definition of Done

- `tatr check` exits 0 (cmd: `tatr check`).
- 094942's drop reasons are unchanged; only the checkbox state changed (manual: diff shows only `[ ]`->`[x]`).
