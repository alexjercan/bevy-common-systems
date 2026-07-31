# KISS pass: mesh/ + meth/ + camera/

- STATUS: OPEN
- PRIORITY: 70
- TAGS: chore,kiss,mesh,camera
- KIND: STORY
- FLOW STEP: BACKLOG
- PLAN STATUS: DRAFT
- PARENT: 20260731-172116
- DEPENDS ON: 20260731-172208

Scope: `src/mesh/` (`builder.rs`, `explode.rs`, `mod.rs`), `src/meth/` (`lerp.rs`, `sphere.rs`, `mod.rs`), `src/camera/` (`shake.rs`, `chase.rs`, `wasd.rs`, `post.rs`, `project.rs`, `skybox.rs`, `mod.rs`).

`mesh/builder.rs` is the crate's biggest code body (521 lines before tests) and
the strongest split candidate: primitive construction, subdivision, noise
displacement, plane slicing, and `Mesh` conversion are five separable concerns
behind one type. Decide on evidence; if it splits, the public path
`bevy_common_systems::mesh::prelude::TriangleMeshBuilder` must not move.

`camera/shake.rs` has 33 non-doc comments. Its module doc records the
accumulating-shake bug and the Restore/Apply ordering contract -- that is
convention-mandated API documentation, keep it. The set-ordering doc comments
are likewise load-bearing per AGENTS.md.

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
