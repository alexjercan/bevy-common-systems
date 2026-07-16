# bevy_common_systems docs

Reference documentation lives here. Task-scoped records (spikes, reviews,
retros, design notes) live next to their task under `tasks/<id>/` - see
"Where records go" below.

## Reference

- [dev-harness.md](dev-harness.md) - the headless dev/test harness used to
  drive and screenshot the example games without a human at the keyboard.
- [wasm-web-builds.md](wasm-web-builds.md) - building the examples to
  WebAssembly and the `web/` showcase site (trunk, audio unlock, deploy).
- [retros/LESSONS.md](retros/LESSONS.md) - the distilled lessons ledger.
  Read it before starting work; /compound appends to it.

The crate's own orientation (module map, conventions, build commands) is in
the repo-root [AGENTS.md](../AGENTS.md).

## Where records go

Everything tied to one task lives in that task's folder, so an `ls` of
`tasks/<id>/` shows the whole story:

- `tasks/<id>/TASK.md` - the task itself (tatr).
- `tasks/<id>/SPIKE.md` - research that scoped the task (/spike).
- `tasks/<id>/REVIEW.md` - review rounds and verdicts (/review).
- `tasks/<id>/RETRO.md` - the retrospective (/compound).
- `tasks/<id>/NOTES.md` - the design/fix record for the shipped change.

Do not create per-task record files under `docs/`. The only records kept
here are in [retros/](retros/README.md): the `LESSONS.md` ledger plus a few
old records whose task folder no longer exists.

## After a meaningful change

Record, per [AGENTS.md](../AGENTS.md): what changed and why (alternatives,
tradeoffs), difficulties and how they were diagnosed, and what to do
differently next time. Update the relevant reference doc, or write the task's
`RETRO.md`/`NOTES.md`; new recurring lessons go to the `LESSONS.md` ledger.
Plain ASCII punctuation only.
