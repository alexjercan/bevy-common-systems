# Decision: comment-tag shape, shared checker, and no file splits

- DATE: 2026-07-31
- STATUS: ACCEPTED
- TASK: 20260731-172208
- TAGS: kiss, convention, debug

## Context

This task is the first child of epic 20260731-172116 and sets the comment
convention the other four clusters follow. The epic names the tag vocabulary
(`NOTE:` / `FIXME:` / `BUG:` / `TODO:`) and a keep/drop rule, but leaves three
things open that every sibling depends on: what shape a kept comment takes,
how the rule is enforced, and whether the in-scope files want splitting.

Measured on base: 74 non-doc comment lines in 37 blocks across scope;
code-before-tests is `autopilot.rs` 320, `screenshot.rs` 270, `inspector.rs`
189, `completion.rs` 147, `harness/mod.rs` 85, `wireframe.rs` 73, `lib.rs` 45,
`debug/mod.rs` 40.

## Decision

- **D1 -- tag shape.** A kept non-doc comment is one tagged BLOCK whose FIRST
  line carries the tag. One line is preferred; wrapping is allowed. "One
  tagged line" in the parent Steps is read as "one tagged comment".
- **D2 -- enforcement.** Ship `scripts/check-comment-tags.sh <path>...`, which
  fails listing every non-doc comment block whose first line lacks a tag, plus
  a one-line `AGENTS.md` Conventions bullet pointing at it. The four sibling
  tasks scope it per-cluster.
- **D3 -- structure.** No file in this cluster is split.

## Alternatives considered

- **D1 (a): every kept comment is exactly one line.** Rejected:
  `src/debug/inspector.rs:142` is a six-line hazard (removing
  `PrimaryEguiContext` alone leaves `EguiContext` +
  `EguiMultipassSchedule`, and bevy_egui panics on the next pass) that cannot
  compress to one line without losing the mechanism. Pushing it to `NOTES.md`
  moves the guard away from the code it guards -- the exact failure the
  epic's keep rule exists to prevent.
- **D2: prose bullet only.** Rejected: not runnable, so four sibling tasks
  would each re-derive the rule by hand.
- **D2: extend `scripts/check-ascii.sh`.** Rejected: different rule, and the
  siblings need a per-cluster path argument that script does not take.
- **D3: split `autopilot.rs` or `screenshot.rs`.** Rejected on the epic's own
  rule, which requires measured size AND more than one concern. Both sit well
  under the epic's flagged outliers (`mesh/builder.rs` 521,
  `modding/events.rs` 404) and each carries a single concern.

## Consequences

- The checker becomes the epic's runnable comment proof, inherited by
  20260731-172223 / 172224 / 172232 / 172233; a later change to the tag
  vocabulary is a one-file edit.
- Prose beyond the guarded mechanism still goes to `NOTES.md`, so kept
  comments stay short without losing hazards.
- D3 is re-confirmed against final measurements in `NOTES.md` before the task
  closes; if a file grows a second concern during the pass, D3 is revisited
  rather than assumed.
