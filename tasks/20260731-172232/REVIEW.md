# Review: KISS pass: modding/ + persist/ + macros subcrate

- TASK: 20260731-172232
- BRANCH: refactor/kiss-modding-persist-macros

## Round 1

- REVIEWER: out-of-context
- VERDICT: APPROVE

No BLOCKER or MAJOR. The comment pass is sound: the 24-block baseline is exact
(line numbers included), every dropped comment clears the "restates the code"
bar, and the one substantive change beyond hygiene -- the corrected AGENTS.md
`EventKind` gotcha plus its regression test -- is true and actually guards what
the records claim. Two MINORs are record/convention accuracy; three NITs are
tidiness.

- [x] R1.1 (MINOR) tasks/20260731-172232/NOTES.md:26 - the code-before-tests
  table gives `persist/mod.rs` as `total 200 / code 200`, but that file has a
  test module: `#[cfg(all(test, not(target_arch = "wasm32")))]` at master line
  149 (branch line 147). Code before tests is ~148, not 200. The other five
  rows re-derive exactly (`events.rs` 543/404, `registry.rs` 494/320,
  `mod.rs` 9/9, `backend.rs` 160/80, macros `lib.rs` 43/43) -- this row was
  measured with a `#[cfg(test)]` grep that the `cfg(all(...))` form escapes.
  Change: correct the row to the real number. The KEEP call is unaffected (it
  gets smaller, not larger), so this is record accuracy only.

- [x] R1.2 (MINOR) src/persist/backend.rs:112 - the `///` promoted onto
  `unsafe_keys_are_rejected` says only *why the test is pure* ("so it never
  races the env-based two-app test on `BCS_PERSIST_DIR`") and never says what
  the test proves. That is a design guard, not test intent, so it is the one
  place in this diff where an untagged body comment moved into the checker's
  blind spot -- exactly the pattern the AGENTS.md sub-bullet added by
  20260731-172224 R1.3 draws a line against. The sibling hazard about the same
  env var (src/persist/mod.rs:164) correctly stayed a tagged `NOTE:` block.
  Change: either lead the `///` with the intent ("`is_safe_key` rejects
  traversal and separator keys; `load`/`save` both gate on it") and keep the
  purity guard as a `NOTE:` in the body, or drop the guard back into the body
  as a `NOTE:` block.

- [x] R1.3 (NIT) src/modding/events.rs:416 -
  `use crate::prelude::EventKind as _;` inside the new test is dead: the module
  already has `use super::*`, which brings the `EventKind` trait defined in
  this very file into scope (that is what makes `OnQuiet::name()` resolve), and
  an `as _` alias cannot supply the *name* `EventKind` that the
  `requires_unit_payload<E: EventKind<Info = ()>>` bound at line 427 needs --
  so the bound is already relying on the glob, not on this line. Change: delete
  the import.

- [x] R1.4 (NIT) src/modding/events.rs:301 - the de-linked sentence
  ("The dispatcher (the private `queue_system`) used to scan *every* handler in
  the world") now runs ~89 columns against the ~78-column wrap of the
  paragraph it sits in. Change: re-wrap the paragraph.

- [x] R1.5 (NIT) AGENTS.md:237 - the rewritten `EventKind` gotcha is four
  sentences on one very long line in a list whose neighbours are one or two.
  The content is right and worth keeping; the historical half ("the old warning
  ... described a defect fixed since") could compress to a clause. Change:
  optional trim to two sentences plus the test pointer.

### Verified independently (not taken from the record)

- Baseline: reconstructed all six in-scope files from `master` into a temp tree
  and ran `check-comment-tags.sh` -- **24** untagged blocks, split
  `events.rs` 8 / `registry.rs` 8 / `persist/mod.rs` 5 / `persist/backend.rs` 2
  / macros `lib.rs` 1, and every line number in NOTES.md's four tables matches
  the checker output one-for-one (79, 256, 388, 408, 481, 497, 509, 538; 345,
  357, 394, 401, 413, 422, 460, 486; 94, 145, 169, 174, 187; 99, 115; 12).
  Every one of the 24 has a call recorded. On the branch the same command exits
  0.
- Dropped comments audited against the "guards a value / non-obvious setting /
  hazard" bar. All five drops clear it: `events.rs:388` is restated by the
  `EventHandlerIndex` rustdoc twenty lines up and by `index.handlers(...)`
  itself; `registry.rs:394` restates three asserts whose `min: 3` literal is
  eight lines above in the same JSON; `persist/mod.rs:174,187` are carried by
  the assertion messages, which do read "default on a clean store" and
  "restored across launches" as claimed; `backend.rs:99` restates a
  save-then-assert pair. No dropped comment pointed outside its file.
- The new test guards what the records say. `bevy_common_systems_macros/src/lib.rs:18`
  really is `let mut event_info = quote! { () };` and the name default really is
  `to_string().to_lowercase()`, so the corrected AGENTS.md bullet states current
  behaviour. `requires_unit_payload::<OnQuiet>()` pins `Info = ()` at compile
  time, and the attribute-less `#[derive(EventKind)]` on `OnQuiet` is what
  reproduces the original defect (a compile failure) -- the claim that this is a
  compile-time rather than runtime guard is correct.
- The rewritten `NOTE:` at src/modding/events.rs:255 is accurate: the dispatch
  chain below carries `.run_if(not(is_queue_empty).or_else(resource_changed))`
  while `maintain_handler_index` does not, and it is pinned with a direct
  `.before(queue_system::<W>)` rather than through a set -- which is the
  AGENTS.md empty-set rule honoured, not just cited.
- `cargo doc --no-deps --features debug`: in-scope warnings **0**; the 4 that
  remain are `helpers/mod.rs`, `helpers/pointer.rs` x2, `input/mod.rs` -- all
  cluster 20260731-172233's, as the DoD says.
- `cargo fmt --check` exit 0; `cargo clippy --all-targets` exit 0 (the single
  warning is the expected `proc-macro-error2` future-incompat note);
  `cargo test --lib` **148 passed**, and `events.rs` goes 3 -> 4 `#[test]`, so
  the 147 master baseline follows; a clean rebuild of the lib test target emits
  no warnings (so the new test introduces none);
  `check-comment-tags.sh src/modding src/persist bevy_common_systems_macros/src`
  exit 0; `check-ascii.sh` exit 0; the bare-HUID grep prints nothing;
  `tatr check --ledger LESSONS.md` exit 0.
- Public API unchanged: the whole `src/` + macros diff is comments, rustdoc
  prose and one added `#[cfg(test)]` fn. No `pub` signature, no prelude line.
- Not re-run (cost, and the diff cannot plausibly affect them): `cargo test
  --features debug`, `cargo test --examples`, `cargo clippy --all-targets
  --features debug`. Nothing in the diff is feature-gated and no example is
  touched; the close-out's numbers for those are accepted rather than verified.

### Design

The all-KEEP split call is defensible on the evidence given, and survives the
counterfactual. The dependency-set test is the right one and it genuinely fails
for `events.rs`: `EventHandlerIndex` stores `EventHandler<W>` and is generic
over `W: EventWorld`, and `queue_system` reads `GameEvent` while calling both
trait objects, so any cut leaves each half importing what it imported before.
Building this from scratch today, `events.rs` would still be one file -- 404
lines of one bus. `registry.rs` at 320 is likewise one pipeline. Sizing files
by concern rather than by line count is the epic's stated rule and it was
applied, not waived.

### Pending user checks

The two `manual:` DoD items were both discharged by the reviewer above (NOTES.md
carries a call for all 24 blocks; public API unchanged), so nothing blocks. One
housekeeping note: `tasks/20260731-172232/TASK.md` has an uncommitted
`FLOW STEP: WORKING -> REVIEWING` edit in the worktree -- expected mid-review,
just fold it into the review-round commit.

### Responses (round 1)

- R1.1 fixed. Re-derived: `git show master:src/persist/mod.rs` is 200 lines with
  `#[cfg(all(test, not(target_arch = "wasm32")))]` at line 149, so 148 code
  lines. NOTES.md's table now reads 148 and carries a parenthetical naming the
  `cfg(all(...))` form the first grep escaped. The KEEP call is unchanged; 148
  was never near a split threshold.
- R1.2 fixed. `src/persist/backend.rs` now leads the `///` with what the test
  proves (`is_safe_key` is the containment boundary both `load` and `save` gate
  on) and the purity guard moved back into the body as a tagged `NOTE:`, matching
  how the same hazard is recorded at `src/persist/mod.rs`.
- R1.3 fixed. Confirmed dead before deleting: the tests module's `use super::*`
  supplies the `EventKind` name, and an `as _` alias cannot supply the name the
  `requires_unit_payload<E: EventKind<Info = ()>>` bound needs -- the bound was
  already resolving through the glob. `cargo test --lib` still 148 passed after
  the deletion, so the trait was in scope by another path as claimed.
- R1.4 fixed. Paragraph re-wrapped to the surrounding ~78 columns.
- R1.5 fixed. The historical half compressed to a clause; the bullet is now two
  sentences plus the test pointer.

Re-verified after the fixes: `cargo fmt --check` 0, `check-comment-tags.sh
src/modding src/persist bevy_common_systems_macros/src` 0, `check-ascii.sh` 0,
`cargo clippy --all-targets` 0 (only the expected `proc-macro-error2` note),
`cargo test --lib` 148 passed. All five findings were NIT or MINOR and none
touched behaviour, so the `--features debug` and `--examples` suites verified
before the round stand.
