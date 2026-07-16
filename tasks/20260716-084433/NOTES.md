# docs: records live next to their task

- DATE: 2026-07-16
- TASK: 20260716-084433

## What changed

Mirrored the nova-protocol docs restructure (their commit 514fb577) here.
Every record tied to a task now lives in that task's folder instead of loose
under `docs/`:

- `docs/retros/<id>-*.md` -> `tasks/<id>/RETRO.md` (100 files).
- `docs/spikes/<id>-*.md` -> `tasks/<id>/SPIKE.md` (7 files); `docs/spikes/`
  removed.
- Top-level dated design notes `docs/2026-MM-DD-*.md` -> `tasks/<id>/NOTES.md`
  (31 files), resolving each file's task id from its header (or by slug/body
  where the header was absent).
- 33 task folders that had been pruned (or were task-less records) were
  recreated as CLOSED archive-stub `TASK.md`s (`TAGS: archive`) so every
  record has a home and `tatr ls` does not break on a folder without a
  `TASK.md`. This includes the 4 formerly task-less records (test-memory,
  web-showcase-gotchas, defaultplugins-doctest, release-license-gate), each
  given a synthetic archive id and its record as `NOTES.md`.

`docs/` now holds only the reference docs (`dev-harness.md`,
`wasm-web-builds.md`), the `LESSONS.md` ledger at the docs root, `docs/plans/`
(README only, no plans yet), and a `docs/README.md` index. `docs/retros/` and
`docs/spikes/` were dissolved entirely - matching nova-protocol's end state.
The "Where records go" convention is documented in `AGENTS.md`.

All ~200 references to the old `docs/` paths across `AGENTS.md`, task files
and source comments were rewritten to the new locations via a rename-map pass.

## Why

An `ls tasks/<id>/` now tells the whole story of a task (spike, task, review,
retro, design note) in one place, and `docs/` stops accumulating loose
per-task files. This matches the sister nova-protocol repo so agents carry
one convention between them.

## Difficulties

- The top-level devlogs used a different date format (`2026-07-03`) than the
  tatr ids (`20260703-HHMMSS`) and several had no task-id header, so mapping
  them to a task needed reading each file's body / matching by slug.
- A handful of records referenced pruned task ids; those became archive stubs.
- Reference rewriting had to be literal (not regex) and longest-path-first to
  avoid partial-path collisions.
