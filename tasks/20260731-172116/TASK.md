# Epic: v0.19.x KISS pass over bcs library sources

- STATUS: OPEN
- PRIORITY: 0
- TAGS: goal,chore,kiss
- KIND: EPIC
- FLOW STEP: BACKLOG
- PLAN STATUS: DRAFT

# Epic

KISS pass over the `bevy_common_systems` library sources (`src/`,
`bevy_common_systems_macros/src/`) for the v0.19.x line. Two deliverables per
module cluster:

1. **Comment hygiene.** Keep rustdoc (`//!`, `///`) -- it is the public API
   surface. Keep inline comments that guard a value, explain a non-obvious
   setting, or record a real hazard; when kept, compact them to a tagged form
   (`NOTE:` / `FIXME:` / `BUG:` / `TODO:`) and drop the narration around them.
   Delete comments that restate the code (`// Start with wireframe mode
   enabled.`) or narrate task history ("task <HUID> wanted me to...") -- that
   prose belongs in `tasks/<id>/NOTES.md`, which already holds it.
2. **Structure.** Split a file only where evidence supports it: measured code
   size (lines before `#[cfg(test)]`) plus more than one concern in the file.
   No speculative module churn, no new abstractions, no API changes.

Baseline (2026-07-31): 13509 lines across 49 files; code-before-tests tops out
at `src/mesh/builder.rs` (521) and `src/modding/events.rs` (404). Raw non-doc
comment count: 452, concentrated in `debug/inspector.rs` (29),
`integrity/plugin.rs` (43), `camera/shake.rs` (33), `physics/pd_controller.rs`
(30), `feedback/flash.rs` (28).

Examples (`examples/`), `web/`, `docs/`, and `tasks/` are out of scope; this is
the library only.

## Done Means

- cmd: `cargo fmt --check` clean
- cmd: `cargo clippy --all-targets` and `cargo clippy --all-targets --features debug` clean
- cmd: `cargo test`, `cargo test --features debug`, `cargo test --examples` pass
- cmd: `./scripts/check-ascii.sh` passes
- cmd: `cargo doc --no-deps` builds with no new warnings (rustdoc survived the pass)
- cmd: no non-doc comment in `src/` matches a bare tatr HUID (`grep -rnE '^\s*//([^/!]|$)' src | grep -E '20[0-9]{6}-[0-9]{6}'` returns only lines that also carry `NOTE:`/`FIXME:`/`BUG:`/`TODO:`)
- cmd: every own-line non-doc comment left in the library is tagged (`./scripts/check-comment-tags.sh src bevy_common_systems_macros/src` exits 0). The script landed with 20260731-172208 and is the shared comment proof; end-of-line comments are exempt by design (see the AGENTS.md Conventions bullet)
- cmd: `tatr check --ledger LESSONS.md` clean
- manual: public API unchanged -- no item renamed, removed, or re-exported differently

## Decisions

- Cluster split and baseline measurements: this file.
- Per-cluster keep/drop calls: each child's `tasks/<id>/NOTES.md`.

## Fog

- Whether `src/mesh/builder.rs` genuinely wants a split (primitives vs slicing
  vs Mesh conversion) or is just long-but-cohesive -- the child decides on
  evidence.
- Whether `src/integrity/plugin.rs` should shed its impact/blast damage systems
  into a sibling file.

## Out of Scope

- `examples/`, `web/`, `docs/`, `tasks/` -- library sources only.
- Any behavior, signature, or public-path change. This pass must be a no-op for
  downstream users.
- Rewriting rustdoc prose for style. Only delete rustdoc that is factually
  stale.

## Child Tasks

Derive the working order with `tatr frontier <epic-id>`.

| ID | Prio | Repo | Title | Landed |
| --- | --- | --- | --- | --- |
| 20260731-172208 | 90 | bevy-common-systems | KISS pass: debug/ + lib.rs + completion.rs (sets the comment convention) | x |
| 20260731-172223 | 80 | bevy-common-systems | KISS pass: integrity/ + physics/ | x |
| 20260731-172224 | 70 | bevy-common-systems | KISS pass: mesh/ + meth/ + camera/ | x |
| 20260731-172232 | 60 | bevy-common-systems | KISS pass: modding/ + persist/ + macros subcrate | x |
| 20260731-172233 | 50 | bevy-common-systems | KISS pass: feedback/ tween/ ui/ transform/ + small modules | |

## Manual Acceptance

- Public API unchanged across the whole epic (spot-check `cargo doc` output or
  `src/lib.rs` prelude re-exports before closing).
- From 20260731-172208, both discharged for its cluster and carried here for
  the epic-wide check: public API unchanged (verified mechanically -- comment
  and blank lines stripped from both revisions, only an added `assert_eq!`
  message differed), and `NOTES.md` carrying a per-block keep/compact/drop
  call plus code-before-tests measurements. Each sibling owes the same two.

## Notes for the remaining children

- Run `./scripts/check-comment-tags.sh <your paths>` as the comment proof; do
  not re-derive the rule. The convention is the AGENTS.md Conventions bullet.
- Read an item's own rustdoc before ruling a comment load-bearing: five of the
  21 comments dropped in 20260731-172208 were restating a `///` block within
  ten lines.
- Promote provenance (task HUIDs, regression history) INTO rustdoc rather than
  deleting it; that satisfies the HUID proof without losing the record.
- `src/` carries 12 end-of-line comments, all in these clusters' files. They
  are exempt from the tag rule but still have to earn their keep.
- Pre-existing `clippy::manual_contains` at `src/completion.rs:88` is filed
  separately as 20260731-180747; do not fold it into a comment pass.
