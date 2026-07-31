# KISS pass: integrity/ + physics/

- STATUS: OPEN
- PRIORITY: 80
- TAGS: chore, kiss, integrity, physics
- KIND: STORY
- FLOW STEP: PLANNED
- PLAN STATUS: APPROVED
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

## Definition of Done

- Every kept non-doc comment in scope is a tagged block; base has 45 untagged (cmd: `./scripts/check-comment-tags.sh src/integrity src/physics` exits 0).
- No non-doc comment in scope carries a bare tatr HUID; base has 1 at `pd_controller.rs:327` (cmd: `grep -rnE '^\s*//([^/!]|$)' src/integrity src/physics | grep -E '20[0-9]{6}-[0-9]{6}' | grep -vE 'NOTE:|FIXME:|BUG:|TODO:'` prints nothing).
- Rustdoc in scope is warning-free (cmd: `nix develop --command cargo doc --no-deps --features debug 2>&1 | grep -cE '^\s+--> src/(integrity|physics)/'` -> 0).
- Formatting clean (cmd: `nix develop --command cargo fmt --check`).
- Lints clean in both feature configurations (cmd: `nix develop --command cargo clippy --all-targets` and `nix develop --command cargo clippy --all-targets --features debug`).
- Tests pass in both feature configurations and for examples (cmd: `nix develop --command cargo test`, `... cargo test --features debug`, `... cargo test --examples`).
- Plain-ASCII rule holds (cmd: `./scripts/check-ascii.sh`).
- Task artifacts and ledger lint clean (cmd: `tatr check --ledger LESSONS.md`).
- `NOTES.md` records a keep/compact/drop call for all 45 comment blocks plus the per-file code-before-tests numbers behind the split-or-keep decision for `integrity/plugin.rs` (manual: read `tasks/20260731-172223/NOTES.md`).
- Public API unchanged: no item renamed, removed, or moved out of its prelude (manual: `git diff master -- src/integrity src/physics` shows no `pub` signature or prelude re-export line changed).
