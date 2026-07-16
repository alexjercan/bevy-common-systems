# Retros folder

Retrospectives live NEXT TO THEIR TASK as `tasks/<id>/RETRO.md` (spikes as
`SPIKE.md`, reviews as `REVIEW.md`, design/fix records as `NOTES.md`). This
folder keeps only two things:

- [LESSONS.md](LESSONS.md) - the distilled lessons ledger, appended by
  /compound. Read it before starting work.
- Records below that have no surviving task folder to live in.

## Task-less records

Design/fix notes that were never a single task (cross-cutting dev knowledge).
Kept here because there is no `tasks/<id>/` to move them into:

- [20260703-web-showcase-gotchas.md](20260703-web-showcase-gotchas.md) -
  gotchas hit while standing up the wasm showcase site.
- [2026-07-03-test-memory.md](2026-07-03-test-memory.md) - taming `cargo test`
  peak memory (dev profile debuginfo).
- [2026-07-05-defaultplugins-doctest-headless-fix.md](2026-07-05-defaultplugins-doctest-headless-fix.md) -
  DefaultPlugins doctests panicking in a headless (CI) run.
- [2026-07-16-release-and-license-gate.md](2026-07-16-release-and-license-gate.md) -
  versioned releases and the third-party license gate.
