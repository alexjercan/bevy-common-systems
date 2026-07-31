# Review: KISS pass: integrity/ + physics/

- TASK: 20260731-172223
- BRANCH: refactor/kiss-integrity-physics

## Round 1

- REVIEWER: out-of-context
- VERDICT: APPROVE

- [x] R1.1 (MINOR) AGENTS.md:83 - the Module Map still attributes impact/blast
  damage to `plugin`, but that half now lives in `src/integrity/damage.rs`.
  Split the bullet: add a `damage` (private) entry and reduce the `plugin`
  bullet to wiring + disable/destroy/prune/cascade.
  - Response: fixed. `AGENTS.md:83-84` now carries both bullets; `damage` is
    marked private and named as the owner of the three damage constants and
    the avian dependency.
- [x] R1.2 (NIT) tasks/20260731-172223/NOTES.md:41 - the drop rationale cites a
  test name that does not exist: `damage_to_zero_destroys_a_lone_node`. The
  actual test is `damage_drives_a_leaf_from_full_health_to_destruction`
  (`src/integrity/plugin.rs:255`). Correct the name.
  - Response: fixed. Confirmed against `git show master:src/integrity/plugin.rs`
    line 421 before editing; the real name is the one the reviewer gives.
- [x] R1.3 (NIT) tasks/20260731-172223/NOTES.md:34 - section headers conflate
  "comments triaged" with "untagged blocks", so the per-section counts do not
  visibly sum to the 45 the DoD names. Restate each header as triaged/untagged.
  - Response: fixed. Recomputed per file with `check-comment-tags.sh` on
    master: blast 1, plugin 22, doom_controller 5, pd_controller 14,
    rigid_body 3 = 45. Headers restated and the sum written out at
    `NOTES.md:105`.
- [x] R1.4 (NIT) src/integrity/plugin.rs:25 - `use super::{components::*,
  damage::*};` glob-imports a private sibling for exactly three observer names.
  Name them explicitly so the wiring file states what it pulls across the seam.
  - Response: fixed. The `damage::` glob is now an explicit three-name list.
    `components::*` stays a glob - it is the module's own component vocabulary,
    matching the convention in the other plugin files.

### In-session verification

The primary re-derived two of the reviewer's load-bearing claims independently
rather than accepting them:

- **Moved code is byte-identical.** Diffed the `add_observer(...)` list in
  `git show master:src/integrity/plugin.rs` against the branch: identical, all
  eight observers plus `derive_integrity_leaves` still registered. The only
  substantive delta the reviewer reported inside the moved bodies was
  `_c2` -> `c2`; confirmed at master `plugin.rs:555-563`, where the binding was
  underscore-prefixed yet used (`collider2: _c2`), so dropping the underscore
  is a correction, not a behaviour change.
- **Per-file untagged counts.** Ran `check-comment-tags.sh` per file against
  master and got 1/22/5/14/3, summing to 45. This is what produced R1.3.

Checks rerun by the primary: `check-comment-tags.sh src/integrity src/physics`
exit 0; bare-HUID grep 0 matches; `check-ascii.sh` exit 0; `cargo fmt --check`
exit 0; `cargo check --all-targets` exit 0 (only the expected
`proc-macro-error2` future-incompat note); `cargo doc --no-deps --features
debug` in-scope warnings 0; `tatr check` exit 0.

### Pending user checks

Not resolvable by review; the user runs these.

- `cargo clippy --all-targets` and `cargo clippy --all-targets --features
  debug`. Standing project rule (`AGENTS.md`, "Build, Verify, Run"): agents do
  not run clippy. Both configs passed earlier in the session, before the final
  `simulate_seconds`, rustdoc and R1.1-R1.4 edits.
- `cargo test`, `cargo test --features debug`, `cargo test --examples`. Blocked
  by `rust-lld` exhausting system RAM while linking test binaries - a separate
  concern with its own diagnosis (see the session's linker-memory
  investigation). `cargo check --all-targets` compiles every target including
  both test modules, so only test EXECUTION is outstanding.
- The two `manual:` DoD proofs (read `NOTES.md`; `git diff master` shows no
  changed `pub` or prelude line). The reviewer confirmed both underlying facts
  independently but correctly declined to tick them.
