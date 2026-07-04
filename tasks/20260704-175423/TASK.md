# HighScore<T> generic best-score resource + record + New best (Wave 2)

- STATUS: OPEN
- PRIORITY: 50
- TAGS: spike,feature

> Spike: docs/spikes/20260704-175058-dev-harness-and-app-scaffolding.md (read
> first). Wave 2 -- clean leaf harvest.

## Goal

All six games (06-11) hand-roll an in-memory best-score resource plus a
`record_high_score` system and the identical "New best!" branch (06:351,984;
07:297; 09:511; 10:303; 11:297). The value type varies (`usize`/`f64`/`f32`) so
the API must be generic: a `HighScore<T: PartialOrd>` resource (Reflect-friendly)
+ a `record_high_score` helper that updates it on game-over and reports whether
it was a new best.

Distinct from the open `persist` task (tasks/20260704-134700): `persist` would
*save* this resource across launches; this task is the resource and its update
rule. They compose -- persisting a `HighScore<T>` is the intended pairing.

Prove it by refactoring a couple of games onto it. This task is stepless on
purpose (spike output); run /plan to break it into steps before /work.
