# KISS pass: mesh/ + meth/ + camera/

- STATUS: CLOSED
- PRIORITY: 70
- TAGS: chore, kiss, mesh, camera
- KIND: STORY
- FLOW STEP: DONE
- PLAN STATUS: APPROVED
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

- [x] Read every file in scope end to end; list each non-doc comment with a keep/compact/drop call in `NOTES.md`.
- [x] Drop code-restating and task-narration comments.
- [x] Compact each kept comment to one tagged line (`NOTE:` / `FIXME:` / `BUG:` / `TODO:`), HUID only when it points at a live task record.
- [x] Audit rustdoc (`//!`, `///`) for stale claims; fix what is wrong, leave style alone.
- [x] Measure code-before-tests per file; split only where the file carries more than one concern, and record the decision (split or keep) in `NOTES.md`.
- [x] Run the full verification suite.

## Definition of Done

- Every kept non-doc comment in scope is a tagged block; base has 57 untagged (cmd: `./scripts/check-comment-tags.sh src/mesh src/meth src/camera` exits 0).
- No non-doc comment in scope carries a bare tatr HUID (cmd: `grep -rnE '^\s*//([^/!]|$)' src/mesh src/meth src/camera | grep -E '20[0-9]{6}-[0-9]{6}' | grep -vE 'NOTE:|FIXME:|BUG:|TODO:'` prints nothing).
- Rustdoc in scope is warning-free (cmd: `nix develop --command cargo doc --no-deps --features debug 2>&1 | grep -cE '^\s+--> src/(mesh|meth|camera)/'` -> 0).
- `NOTES.md` records a keep/compact/drop call for all 57 comment blocks plus the per-file code-before-tests numbers behind the split-or-keep decision for `mesh/builder.rs` (manual: read `tasks/20260731-172224/NOTES.md`).
- Task artifacts and ledger lint clean (cmd: `tatr check --ledger LESSONS.md`).
- Formatting clean (cmd: `nix develop --command cargo fmt --check`).
- Lints clean in both feature configurations (cmd: `nix develop --command cargo clippy --all-targets` and `nix develop --command cargo clippy --all-targets --features debug`).
- Tests pass in both feature configurations and for examples (cmd: `nix develop --command cargo test`, `... cargo test --features debug`, `... cargo test --examples`).
- Plain-ASCII rule holds (cmd: `./scripts/check-ascii.sh`).
- Public API unchanged: no item renamed, removed, or moved out of its prelude; `bevy_common_systems::mesh::prelude::TriangleMeshBuilder` still resolves (manual: `git diff master -- src/mesh src/meth src/camera` shows no `pub` signature or prelude re-export line changed).

## Close-out

**What.** 57 untagged non-doc comment blocks across `src/mesh/`, `src/meth/`
and `src/camera/` -> 0; bare tatr HUIDs were already 0 and stay 0.
`mesh/builder.rs` split (618 -> 495 lines) by moving the triangle-vs-plane
geometry kernel into a new private `src/mesh/slice.rs`. Two stale rustdoc
defects fixed: `mesh/explode.rs`'s module header was `///` on the `use`
statement (so it documented the import and the module rendered with no
description at all), and `meth/mod.rs` still described `tween` as "the
crate's future `tween` easing" after that module shipped.

**Why the split.** `builder.rs` was the crate's largest code body (521 lines
before tests) and carried two things with disjoint dependency sets: the
`TriangleMeshBuilder` API (bevy mesh + assets, `noise`, `crate::meth`) and the
plane-slice kernel `edge_plane_intersection` / `TriangleSliceResult` /
`triangle_slice` (pure `Triangle3d`-vs-plane math, `bevy::prelude` only).
`slice()` is the kernel's only caller. The kernel is also the only part of the
file that is total by construction -- every entry point must return a finite
result for degenerate or parallel input, because `explode` runs it on
arbitrary game meshes -- and that contract now has a module doc stating it.
Public API is byte-identical: the new `slice` MODULE is private (the
`TriangleMeshBuilder::slice` method is untouched), the moved items were private
and are now `pub(super)`, and `mesh::prelude::TriangleMeshBuilder` is
untouched.

**Alternatives.** Keeping `builder.rs` whole (rejected: measured size plus two
concerns is exactly the epic's split criterion); splitting further into
primitives / mutation / `Mesh` conversion (rejected as YAGNI -- all inherent
methods on one type over one shared `triangles` invariant, so scattering them
buys no dependency separation); making `slice` public (rejected, the task
forbids public API changes). `camera/shake.rs` was the other size candidate
(313 code lines) and was measured and KEPT: one concern, the bulk being a
headless-app test rig that belongs beside what it exercises.

**Difficulties.** 20260731-172223's `split-along-the-test-seam` lesson did not
apply here -- `builder.rs` had a single test module, so the "two test modules
sharing no helper" signal was absent and the split had to rest on the
dependency-set argument alone. Recorded as counter-evidence in `NOTES.md`
rather than skipped. The recurring question in this cluster was where test
intent belongs: comments explaining what a test proves were promoted to `///`
doc comments on the test fn (exempt by design, and the intent describes the
test, not a line inside it), while comments guarding a magic value inside a
body (the `60`-frame decay loop in `shake.rs`) stayed as tagged `NOTE:`
blocks. `meth/lerp.rs` lost all six of its comments as restatement but gained
one: `powi(7)` was an unexplained magic exponent.

**Evidence.** `check-comment-tags.sh src/mesh src/meth src/camera` exit 0
(57 -> 0); bare-HUID grep 0 matches; `check-ascii.sh` exit 0; `cargo fmt
--check` exit 0; `cargo clippy --all-targets` exit 0 and `--features debug`
exit 0 (only the expected `proc-macro-error2` future-incompat note);
`cargo doc --no-deps --features debug` in-scope warnings 0 (the 6 remaining
are pre-existing, in the untouched `helpers/`, `input/` and `modding/`);
`cargo test` 147 + 59, `--features debug` 154 + 66, `--examples` all ok --
identical to the master baseline, confirming the 3 tests that moved to
`slice.rs` all still run; `tatr check --ledger LESSONS.md` exit 0;
`git diff master -- src/mesh src/meth src/camera` shows no changed `pub`
signature or prelude line.

**Reflection.** The two rustdoc defects are the argument for reading a module
header as code rather than skimming it: `///` where `//!` was meant is
invisible to `cargo doc` (no warning -- the doc simply attaches to the next
item) and cost `mesh::explode` its description for the life of the module.
Swept the whole library for the same shape afterwards (`head -1` of every
`.rs` under `src/` and the macros subcrate): `mesh/explode.rs` was the only
one, so no follow-up task is owed.
