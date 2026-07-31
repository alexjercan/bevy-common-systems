# KISS pass: modding/ + persist/ + macros subcrate

- STATUS: OPEN
- PRIORITY: 60
- TAGS: chore,kiss,modding,persist
- KIND: STORY
- FLOW STEP: BACKLOG
- PLAN STATUS: DRAFT
- PARENT: 20260731-172116
- DEPENDS ON: 20260731-172208

Scope: `src/modding/` (`events.rs`, `registry.rs`, `mod.rs`), `src/persist/` (`mod.rs`, `backend.rs`), and the `bevy_common_systems_macros/` subcrate.

`modding/events.rs` (404 lines before tests) and `registry.rs` (320) are the
two biggest non-mesh code bodies, 34 non-doc comments between them. `persist/`
adds 14 more, largely platform narration around the native/wasm backend split.

The macros subcrate is small but its rustdoc is the documented workaround for
the `EventKind` default-`Info`-path footgun (see AGENTS.md gotchas and
`tasks/20260703-095509`). Verify that guidance is still accurate rather than
trimming it.

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
