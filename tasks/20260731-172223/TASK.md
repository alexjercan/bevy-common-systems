# KISS pass: integrity/ + physics/

- STATUS: OPEN
- PRIORITY: 80
- TAGS: chore,kiss,integrity,physics
- KIND: STORY
- FLOW STEP: BACKLOG
- PLAN STATUS: DRAFT
- PARENT: 20260731-172116
- DEPENDS ON: 20260731-172208

Scope: `src/integrity/` (`plugin.rs`, `blast.rs`, `components.rs`, `mod.rs`) and `src/physics/` (`pd_controller.rs`, `doom_controller.rs`, `rigid_body.rs`, `mod.rs`).

`integrity/plugin.rs` is the largest plugin body in the crate (679 lines, 317
before `#[cfg(test)]`) and holds 43 non-doc comments -- the most of any file.
It also runs four distinct concerns in one file: impact damage, blast overlap
damage, disable-at-zero, and destroy/prune cascade. Judge on evidence whether
the damage-source systems belong in a sibling file.

`pd_controller.rs` carries 30 comments including derivation prose and a nova
task HUID at line 327; keep the physics reasoning that a reader cannot recover
from the code, drop the provenance narration.

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
