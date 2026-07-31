# Retro: KISS pass: mesh/ + meth/ + camera/

- TASK: 20260731-172224
- BRANCH: refactor/kiss-mesh-meth-camera
- REVIEW ROUNDS: 1

## What went well

- The split call was made from measurements taken before any edit
  (code-before-tests per file, dependency sets per item), so `builder.rs` was
  split and `shake.rs` - the other big file - was demonstrably left alone.
  Recording the KEEP decisions with their numbers is what makes "no
  speculative churn" checkable instead of asserted.
- Two real rustdoc defects surfaced from reading module headers as code rather
  than skimming them: `mesh/explode.rs` used `///` on its `use` statement, so
  the module had no doc at all and `cargo doc` never warned; `meth/mod.rs`
  still called the shipped `tween` module future work. Both were invisible to
  every check in the suite.
- The out-of-context reviewer earned its keep on a diff that was mostly
  comment deletion: it found the `///`-inside-`#[cfg(test)]` loophole, which
  nobody inside the implementing context would have questioned, and two
  arithmetic slips in the record.

## What went wrong

- Two numbers in NOTES.md were wrong (R1.1, R1.2): `shake.rs` labelled 20
  comment blocks against a 15-row table, and "3 of 7 tests moved" against 6.
  Root cause: both were written from memory of the reading pass instead of
  re-running the command that produced them. TASK.md quotes a raw comment-LINE
  count (33 for `shake.rs`) while the checker counts BLOCKS, and the two units
  were silently mixed. The per-file baseline is now written into NOTES.md, so
  the total is reconstructible.
- Eleven comments were promoted to `///` on test fns without recording that as
  a decision (R1.3). The route is legitimate and had 13 precedents in landed
  code, but it satisfies `check-comment-tags.sh` without the text becoming
  documentation - rustdoc does not render `#[cfg(test)]` items - so an
  unstated version of it is indistinguishable from gaming the checker.
- `cargo fmt` reflowed the review fix (R1.5 narrowed a signature's visibility,
  fmt then collapsed it back to one line), so `fmt --check` failed after the
  fix round. Same shape as the sibling task's fmt-relocates-comments trap:
  fmt runs last, always.

## What to improve next time

- Any count that reaches a record comes from re-running its command at write
  time, with the unit named. "57 blocks" and "33 comments" are different
  measurements of the same file, and reading them as the same number is what
  produced both slips.
- When a pass picks a route that a project checker cannot see, write the
  decision down in the same commit. The reviewer will otherwise have to ask,
  and the next cluster will re-decide it.

## Action items

- Folded into AGENTS.md this task: the `///`-on-test-fn vs tagged-`NOTE:`
  split, with an explicit warning that the `///` form is not an escape hatch
  for untagged body comments. The two remaining epic clusters
  (20260731-172232, 20260731-172233) inherit it.
- Ledger: `record-numbers-from-a-rerun`, and a bump of the existing
  `fmt-relocates-eol-comments` lesson to cover fmt reflowing a fix.

## Diagnosis

- **Breadth.** The diff is 10 files but shallow: 57 comment edits, one file
  split, two doc fixes. It matches the plan; the cluster was sized by the
  epic and did not grow.
- **Churn.** One review round, no rework of code behaviour - five of six
  findings were record accuracy or visibility tidiness. The one that would
  have been prevented at plan time is R1.3: the plan never asked "which of
  these comments become rustdoc, and is that route legible to the checker?",
  which is the cold-reader rationale test applied to a convention rather than
  to a design.
- **Context.** No pressure observed: no compaction warning, no checkpoint, no
  handoff. The heavy verification ran behind a shared build lock (a second
  session on the same box), which cost wall-clock but not context; the
  `cargo test` runs were handed to the user and returned green.
