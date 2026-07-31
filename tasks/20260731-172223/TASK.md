# KISS pass: integrity/ + physics/

- STATUS: CLOSED
- PRIORITY: 80
- TAGS: chore, kiss, integrity, physics
- KIND: STORY
- FLOW STEP: DONE
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

- [x] Read every file in scope end to end; list each non-doc comment with a keep/compact/drop call in `NOTES.md`.
- [x] Drop code-restating and task-narration comments.
- [x] Compact each kept comment to one tagged line (`NOTE:` / `FIXME:` / `BUG:` / `TODO:`), HUID only when it points at a live task record.
- [x] Audit rustdoc (`//!`, `///`) for stale claims; fix what is wrong, leave style alone.
- [x] Measure code-before-tests per file; split only where the file carries more than one concern, and record the decision (split or keep) in `NOTES.md`.
- [x] Run the full verification suite.

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

## Close-out

**What.** 45 untagged non-doc comment blocks across `src/integrity/` and
`src/physics/` -> 0; 1 bare cross-repo tatr HUID -> 0. `integrity/plugin.rs`
split (679 -> 304 lines) into wiring+cascade, with the collision-to-damage
half moved to a new private `src/integrity/damage.rs`. Two stale rustdoc
defects fixed: `physics/mod.rs` listed only `pd_controller` (missing
`doom_controller` and `rigid_body`), and `doom_controller.rs` carried a dead
intra-doc link to `examples/14_breach`.

**Why the split.** `plugin.rs` ran two concerns with disjoint dependency
sets - collisions-to-damage (needs avian, owns three tuning constants, ~155
lines) and disable/destroy/prune/cascade (pure ECS, no avian, ~103 lines).
The file's own two test modules, `mod tests` and `mod physics_tests`, had
already drawn that seam and shared no helper; the split follows it exactly.
Public API is byte-identical: `damage` is private, the moved observers were
already private and are now `pub(super)`, and `IntegrityPlugin` still
registers all eight observers.

**Alternatives.** Keeping `plugin.rs` whole (rejected: largest plugin body in
the crate, and the evidence pointed the other way); splitting `damage.rs`
further into impact vs blast (rejected as YAGNI - shared constants, one
funnel into `HealthApplyDamage`); making `damage` public (rejected - would
grow the public API the task forbids changing). `pd_controller.rs` and
`doom_controller.rs` were measured and KEPT: 150 and 201 code lines
respectively, one concern each, the bulk being test rigs that belong next to
what they exercise.

**Difficulties.** `cargo fmt` relocated five end-of-line comments into loop
bodies, converting exempt labels into untagged blocks and re-failing
`check-comment-tags.sh`; fixed properly by extracting a `simulate_seconds`
test helper that removed both the comments and five magic loop counts. A
pre-existing `manual_contains` clippy warning in the out-of-scope
`src/completion.rs` would have made the lints proof unmeetable, so it was
fixed as a one-liner. Linking a private module from public rustdoc warns, so
`plugin.rs` names `damage` in prose rather than as a link. One real code
duplication surfaced: `axis.normalize_or_zero()` was called twice in
`pd_controller`'s torque path.

**Evidence.** `check-comment-tags.sh src/integrity src/physics` exit 0;
bare-HUID grep 0 matches; `check-ascii.sh` exit 0; `cargo fmt --check`
exit 0; `cargo check --all-targets` exit 0 (only the expected
`proc-macro-error2` future-incompat note); `cargo doc --no-deps --features
debug` in-scope warnings 4 -> 0; `git diff master -- src/integrity
src/physics` shows no changed `pub` signature or prelude line.

**Deferred proofs, since closed.** The clippy and `cargo test` proofs were
outstanding at review time: clippy sat behind a session prohibition (since
lifted -- it links nothing) and `cargo test` behind `rust-lld` exhausting
system RAM, fixed as its own task 20260731-210044 (`CARGO_BUILD_JOBS`/
`RUST_TEST_THREADS` capped in `flake.nix`). All were run on this branch
before landing, after merging current master in:

- `cargo clippy --all-targets` exit 0, `--features debug` exit 0; only the
  expected `proc-macro-error2` future-incompat note in each.
- `cargo test` 147 + 59, `--features debug` 154 + 66, `--examples` all pass
  (run on the branch before the final master merge, which touched only a
  `flake.nix` comment and task records).
- `cargo doc --no-deps --features debug` exit 0; 6 warnings, all in untouched
  files (`helpers/`, `input/`, `modding/`), 0 in scope -- none new.
- `cargo fmt --check` exit 0; `./scripts/check-ascii.sh` exit 0;
  `./scripts/check-comment-tags.sh src/integrity src/physics` exit 0;
  bare-HUID grep 0 matches; `tatr check --ledger LESSONS.md` exit 0.

The whole-tree `check-comment-tags.sh src bevy_common_systems_macros/src`
still reports 188 untagged blocks -- all in the epic's three remaining
clusters, which is the epic's gate, not this task's.

**Reflection.** The split-or-keep call is much cheaper to make correctly when
you look at the test modules first: two test modules that share no helper are
a seam the author already found and did not act on. Measuring code-before-
tests rather than total lines was what kept `pd_controller.rs` (564 lines,
150 of code) from being split for no reason.
