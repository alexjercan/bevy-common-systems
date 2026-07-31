# KISS pass: feedback/ tween/ ui/ transform/ + small modules

- STATUS: OPEN
- PRIORITY: 50
- TAGS: chore,kiss,ui,tween
- KIND: STORY
- FLOW STEP: BACKLOG
- PLAN STATUS: DRAFT
- PARENT: 20260731-172116
- DEPENDS ON: 20260731-172208

Scope: everything not claimed by the other children -- `src/feedback/`, `src/tween/`, `src/ui/`, `src/transform/`, `src/audio/`, `src/health/`, `src/helpers/`, `src/input/`, `src/scoring/`, `src/time/`.

Most files here are small and clean; the density sits in `feedback/flash.rs`
(28 non-doc comments, 292 before tests), `tween/mod.rs` (24), `screen_flash.rs`
(13), and `ui/touchpad.rs` (10). Expect this child to be mostly deletion with
few or no splits -- say so in `NOTES.md` rather than manufacturing structure.

`ui/status.rs` documents that its closures run in an exclusive system every
frame; that is a performance contract, not fluff.

## Steps

- [ ] Read every file in scope end to end; list each non-doc comment with a keep/compact/drop call in `NOTES.md`.
- [ ] Drop code-restating and task-narration comments.
- [ ] Compact each kept comment to one tagged line (`NOTE:` / `FIXME:` / `BUG:` / `TODO:`), HUID only when it points at a live task record.
- [ ] Audit rustdoc (`//!`, `///`) for stale claims; fix what is wrong, leave style alone.
- [ ] Measure code-before-tests per file; split only where the file carries more than one concern, and record the decision (split or keep) in `NOTES.md`.
- [ ] Run the full verification suite.

## Done Means

- cmd: `cargo fmt --check` clean
- cmd: `cargo clippy --all-targets` clean
- cmd: `cargo clippy --all-targets --features debug` clean
- cmd: `cargo test` and `cargo test --features debug` pass
- cmd: `cargo doc --no-deps` builds, no new warnings
- cmd: `./scripts/check-ascii.sh` passes
- manual: no public item renamed, removed, or moved out of its prelude
