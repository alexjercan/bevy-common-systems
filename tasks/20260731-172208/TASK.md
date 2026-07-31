# KISS pass: debug/ + lib.rs + completion.rs (sets the comment convention)

- STATUS: OPEN
- PRIORITY: 90
- TAGS: chore,kiss,debug
- KIND: STORY
- FLOW STEP: BACKLOG
- PLAN STATUS: DRAFT
- PARENT: 20260731-172116

Scope: `src/debug/` (`inspector.rs`, `wireframe.rs`, `harness/{mod,autopilot,screenshot}.rs`, `mod.rs`), plus `src/lib.rs` and `src/completion.rs`.

Densest fluff in the crate: `inspector.rs` has 29 non-doc comments, several pure
restatement (`// Start with debug mode enabled.`, `// Physics debug plugins.`),
alongside genuinely load-bearing ones (the egui-cluster removal hazard around
line 142, the primary-context retargeting rationale) that must survive as
tagged `NOTE:` lines. `wireframe.rs` is four restating comments over 73 lines.
The harness files carry 34 between them.

Note the `debug` feature gate: everything here needs `--features debug` to
compile, so verify both feature configurations.

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
