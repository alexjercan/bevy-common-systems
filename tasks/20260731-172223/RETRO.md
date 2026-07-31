# Retro: KISS pass: integrity/ + physics/

- TASK: 20260731-172223
- BRANCH: refactor/kiss-integrity-physics
- REVIEW ROUNDS: 1

## What went well

- **The split-or-keep call was decided by evidence already in the file.**
  `integrity/plugin.rs` had two test modules, `mod tests` (avian-free cascade)
  and `mod physics_tests` (avian-driven damage), sharing no helper. That seam
  was the author's own, found and not acted on. Cutting along it made the move
  mechanical and byte-identical; the reviewer verified that independently.
- **Measuring code-before-tests rather than total lines** stopped two false
  splits. `pd_controller.rs` is 564 lines but only 150 of code - the bulk is an
  avian integration rig that belongs beside what it exercises.
- **Two real code defects fell out of the comment pass** without being sought:
  a duplicated `axis.normalize_or_zero()` in the torque path, and five magic
  loop counts that the `simulate_seconds` helper deleted along with the five
  comments labelling them. Reading every comment forces reading every line.
- **Round 1 was cheap and caught real things.** All four findings were doc- or
  record-level (MINOR/NIT, no blockers), and three of them - a nonexistent test
  name cited in NOTES.md, per-file counts that did not sum to 45, a glob import
  of a private sibling - are exactly the class an implementer cannot see.

## What went wrong

- **`cargo fmt` silently defeated the end-of-line-comment exemption.** The
  comment convention exempts end-of-line comments from the tag rule, so
  `for _ in 0..600 { // 10 s of sim at 60 Hz` looked like a clean compaction of
  five duplicated comments. rustfmt relocated each one into the loop body,
  converting five exempt labels into five untagged BLOCKS and re-failing
  `check-comment-tags.sh` after it had already passed. The decision seemed
  sound because the exemption is real - what was missed is that it only holds
  where rustfmt will not reflow the line.
- **Section counts in NOTES.md conflated two different numbers.** The headers
  said "N blocks" while the tables mixed already-tagged comments (triaged) with
  untagged ones (what the DoD counts), so the per-file numbers did not sum to
  the 45 the DoD names. Nobody could have checked the total from the record.
- **A DoD proof was blocked by an out-of-scope defect.** A pre-existing
  `manual_contains` lint in `src/completion.rs` would have failed the
  lints-clean proof for a reason unrelated to this task.
- **Two proof classes could not be run at all.** Clippy became a standing
  session prohibition mid-task, and `cargo test` is blocked by `rust-lld`
  exhausting system RAM while linking test binaries. `cargo check --all-targets`
  covers compilation of every target, so only test EXECUTION is outstanding -
  but that gap is real and is recorded as a pending user check rather than
  papered over.

## What to improve next time

- **Treat an end-of-line comment as safe only after a simple statement.** After
  an opening brace, a `,` in a multi-line call, or anything rustfmt may reflow,
  it will be relocated and become a block. When the urge is to label a magic
  number at the end of a line inside a construct, extract a named helper or
  binding instead - which is what fixed this and was better anyway.
- **Re-run the formatter BEFORE the convention checker, not after.** The order
  `fmt` -> `check-comment-tags` would have caught this in one step; running the
  checker first produced a green that fmt then invalidated.
- **State the counting basis in a record that carries counts.** "N triaged, M
  untagged" costs four words and makes the record auditable against the DoD.
- **Fix a proof-blocking defect outside scope, and say so in the close-out.**
  Leaving it makes a DoD item permanently unmeetable; fixing it silently is
  scope creep. One line in the record resolves both.

## Action items

- Follow-up task for the `rust-lld` link-memory regression: the
  `split-debuginfo`/`line-tables-only` mitigation from 20260703-000003 was
  measured at 6 examples and 12 doctests; the tree now has 15 and 60, and
  rustdoc's doctest harness never receives `[profile.dev]` at all. Filed
  separately - it blocks `cargo test` for every task, not just this one.
- No follow-up needed for the split itself; `integrity/damage.rs` is private
  and the public API is unchanged.
