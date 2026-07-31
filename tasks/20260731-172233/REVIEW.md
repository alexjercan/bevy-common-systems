# Review: KISS pass: feedback/ tween/ ui/ transform/ + small modules

- TASK: 20260731-172233
- BRANCH: refactor/kiss-feedback-tween-ui-transform

## Round 1

- REVIEWER: out-of-context
- VERDICT: REQUEST_CHANGES

- [x] R1.1 (MAJOR) src/material.rs:58 - `src/material.rs` is claimed by no
  epic child, and this task's Story scopes it in ("everything not claimed by
  the other children") even though the DoD enumerates only the ten
  directories. Being the last child, nobody else will take it, so the epic's
  own gate `./scripts/check-comment-tags.sh src
  bevy_common_systems_macros/src` still exits 1 after this lands. Tag the
  block in place as a `NOTE:` (it is a real hazard -- an `unlit` emissive
  material skips the lighting pass and does not bloom) and add
  `src/material.rs` to the DoD's checker path list.
  - Response: Fixed. `src/material.rs:58` compacted and tagged as a `NOTE:` naming the real hazard (an `unlit` emissive skips the lighting pass, so it does not bloom). Both DoD checker proofs now name the file, and the epic-wide invocation `./scripts/check-comment-tags.sh src bevy_common_systems_macros/src` was added alongside so a scoping gap cannot recur -- it now exits 0. Recorded in NOTES.md under its own heading, including why the file belonged to this child.
- [x] R1.2 (MINOR) tasks/20260731-172233/NOTES.md:231 - the split table records
  `ui/status.rs` as `326 | 326`, and TASK.md / NOTES.md repeat "326" three more
  times, but the committed file is 328 lines (no test module, so total ==
  code). The number was taken before the final module-doc edit and never
  re-run -- the exact failure `record-numbers-from-a-rerun` guards. Change all
  four occurrences to 328.
  - Response: Fixed. `wc -l src/ui/status.rs` re-run -> 328; all four occurrences updated (TASK.md close-out, and the NOTES.md split table, prose total and section heading). Agreed on the cause: the figure predated the module-doc correction and was never re-derived.
- [x] R1.3 (MINOR) tasks/20260731-172233/NOTES.md:36 - "23 files changed, +104 /
  -151 lines"; `git diff master...HEAD --shortstat -- src` reports 106
  insertions. Change `+104` to `+106`.
  - Response: Fixed, and superseded by a re-derivation rather than a patch of the old number: `git diff --cached --shortstat master -- src` after the round-1 fixes reports 24 files, 109 insertions, 153 deletions, which is what NOTES.md now records. The pre-review figures are kept alongside so the change is traceable.
- [x] R1.4 (NIT) src/ui/health_display.rs:107 - the three dropped body comments
  each justified a magic input (`0.4`, `2.29`, `2.3`) and all three were folded
  into the `///` on `living_sliver_ceils_to_one_percent`. AGENTS.md keeps VALUE
  guards in the body as a tagged `NOTE:`; this is the one place in the diff
  where that split was not applied (contrast `flash.rs:501` and
  `streak.rs:194`, which did keep theirs). Restore a body `NOTE:` naming what
  `2.29` vs `2.3` straddles and trim the `///` to the intent sentence.
  - Response: Fixed, and accepted as the substantive point it is. A body `NOTE:` now states that 0.4 of 230 is the 0.17% sliver and that 2.29 vs 2.30 straddles the 1% boundary; the `///` keeps only the intent sentence. NOTES.md records this as the one file where the intent/value-guard split was not applied during the pass -- the exact misuse the cluster was guarding against.
- [x] R1.5 (NIT) tasks/20260731-172233/NOTES.md:90 - "The other eight (246, 255,
  261, 271, 291, 313, 317, 321, 325, 329 minus the two promoted)" lists ten
  line numbers and asks the reader to subtract, but the two promotions are
  `///` docs on fns rather than any listed line, so the arithmetic does not
  resolve. State plainly that all ten body blocks were removed and two facts
  were folded into the `///` docs.
  - Response: Fixed. The paragraph now states plainly that all ten blocks (line numbers listed) left the bodies as assert restatements, and that the two `///` docs are new text on the fns rather than any of those lines relocated.

### In-session verification

The primary re-derived every load-bearing claim in this round rather than
accepting it:

- R1.1: `./scripts/check-comment-tags.sh src bevy_common_systems_macros/src`
  exits 1 with exactly one residual block, `src/material.rs:58`. Confirmed no
  sibling child claims the file: the epic's five children are 172208
  (debug/lib/completion), 172223 (integrity/physics), 172224
  (mesh/meth/camera), 172232 (modding/persist/macros) and this one. The one
  `tasks/` grep hit for "material" elsewhere is the word "materially" in an
  unrelated task (20260731-210413).
- R1.2: `wc -l src/ui/status.rs` -> 328 on the committed tree.
- R1.3: `git diff master...HEAD --shortstat -- src` -> 106 insertions, 151
  deletions.

The reviewer's baseline re-derivation (`git archive master` into a clean tree
-> exit 1, 106 blocks, per-file breakdown matching line for line) reproduces
the number this task recorded, independently of the implementing session.

Plain observations, not findings:

- No test was weakened: `#[test]` count is 155 on both master and the branch,
  and the multiset of `assert*!` macros is byte-identical between the two
  trees.
- Public API unchanged: `git diff -U0 master...HEAD -- src` touches no line
  containing `pub` or `prelude`.
- Process signal: the Story's scope sentence ("everything not claimed by the
  other children") and its own enumerated directory list disagree, and the
  disagreement survived planning into the DoD. A scope phrased as a complement
  needs the complement computed once against the tree, not restated as a list.

## Round 2

- REVIEWER: out-of-context
- VERDICT: APPROVE

All five round-1 findings CONFIRMED fixed by the round-1 reviewer against the
tree, not against the fix commit's message; their boxes above are ticked on
that confirmation. Two of its checks went past what round 1 asked for:

- R1.1's justification was re-derived independently -- the four sibling
  children's Scope lines were grepped and none claims `material.rs`, so this
  task's scope sentence really does take it.
- R1.4's restored guard was checked under real `f32` rather than decimal:
  `0.4 / 230` is 0.1739%, `2.29` is 0.99565% (ceil branch) and `2.30` is
  exactly 1.0 (round branch), so "2.29 vs 2.30 straddles the boundary where
  ceiling hands over to rounding" is literally true. The trimmed `///` lost no
  fact.

Two new NITs, both in the task record, both fixed before this round was
written:

- [x] R2.1 (NIT) tasks/20260731-172233/TASK.md:68 - the R1.1 edit inlined 163
  characters into a paragraph hand-wrapped at ~76 and dropped a relative
  pronoun. Re-wrap and insert "that".
  - Response: Fixed; re-wrapped to the surrounding width with the pronoun
    restored.
- [x] R2.2 (NIT) tasks/20260731-172233/TASK.md:96 - the Evidence paragraph
  recorded only the ten-directory checker run, though the fix commit added two
  further gates to the DoD. Append the epic-wide checker and `tatr check
  --ledger`.
  - Response: Fixed; Evidence now records both checker invocations with their
    base-state exit codes, plus `tatr check --ledger LESSONS.md`.

### In-session verification

The primary re-ran the full suite itself after the fix commit, exit codes
captured to files rather than through a pipe: `cargo fmt --check` 0, `clippy
--all-targets` 0, `clippy --all-targets --features debug` 0, `cargo test` 0,
`cargo test --features debug` 0, `cargo test --examples` 0, `cargo doc
--no-deps --features debug` 0 with `grep -c '^warning: .*lib doc'` printing 0,
`check-ascii.sh` 0, both checker invocations 0. The reviewer's independent
rerun agrees on every one, and reports no regression: `#[test]` count is still
155 and the `assert*!` multiset still byte-identical to master's.

Pending user checks (`manual:` proofs, not blocking):

- `ui/status.rs`'s exclusive-system performance contract is verified rather
  than trimmed on sight -- read `NOTES.md`.
- `NOTES.md` accounts for every baseline block and states a split call for each
  file over 200 code lines.
- Public API unchanged: `git diff master -- <scope>` shows no `pub` signature
  or prelude re-export line changed.
