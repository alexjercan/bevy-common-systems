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
- 29 task folders that had been pruned were recreated as CLOSED archive-stub
  `TASK.md`s (`TAGS: archive`) so their moved record has a home and `tatr ls`
  does not break on a folder without a `TASK.md`.
- 4 genuinely task-less records (cross-cutting dev knowledge with no single
  task) stay under `docs/retros/` as the task-less bucket.

`docs/` now holds only the reference docs (`dev-harness.md`,
`wasm-web-builds.md`), plus `docs/retros/` with the new `LESSONS.md` ledger,
a `README.md`, and the 4 task-less records. Added a `docs/README.md` index
and documented the "Where records go" convention in `AGENTS.md`.

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
