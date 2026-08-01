# Retro: Release 0.19.6

- TASK: 20260801-112854
- BRANCH: chore/release-0.19.6
- REVIEW ROUNDS: 2

## What went well

- Deriving the scope from the DIFF before drafting the entry. `v0.19.5..HEAD`
  is 80-odd commits whose subjects read like features; the `pub` grep (7
  lines, all `pub(super)` in the two NEW private modules or comment text) and
  the non-comment src diff (46 lines, all tests plus a `manual_contains` fix
  and a `let` shadowing) turned "no public API change" from a hedge into a
  verified lead sentence. Drafting from subjects would have shipped a
  features-flavoured changelog for a release that has none.
- Escalating rather than guessing on the ledger. `tatr check --ledger` was
  already red on `master`, on a `record-numbers-from-a-rerun` entry pointing
  at task `20260801-102152`, which the user created and then deleted the same
  day. Three spellings were probed against the linter (no annotation ->
  `promotion-awaiting-decision`; a prose word -> `bad-disposition`;
  `DEFER <date>` -> `DEFER needs 'at x<count>'`), all of which confirmed the
  entry needs a DISPOSITION, and `~/.claude/skills/lessons/ledger.md` says
  only the user picks one. `LESSONS.md` was reverted to byte-identical with
  `master` and the question handed back.
- The out-of-context review paid for itself on a diff with no code in it: it
  independently re-derived the no-API-change claim before accepting it, and
  all three findings were real.

## What went wrong

- The "tag exists and both are pushed" DoD item was ticked `[x]` while the
  work sat on an unpushed branch with no such tag anywhere (review R1.1). A
  branch cannot satisfy a land-time proof. Had it shipped, the release record
  would have permanently claimed a push that had not happened -- and the
  `[0.19.6]` compare link in CHANGELOG.md is genuinely dead until the tag is
  pushed, so the item is load-bearing, not bookkeeping.
- The Close-out enumerated the 46-line non-comment diff from the reading pass
  instead of re-deriving it from the 46 lines, claiming "two assertion
  messages" when the set holds one -- the other three are in the split files
  that same sentence explicitly excludes -- and omitting the two `mod`
  declarations (review R1.2). Fourth occurrence of a lesson already sitting at
  x3 in Pending promotions, written inside a task that had read it.
- The CHANGELOG RAM bullet credited only the `nix develop` half of a two-half
  fix, omitting `[profile.dev.package."*"] debug = false` (review R1.3) -- the
  half that shrinks each binary, which the flake's own comment points at.

## What to improve next time

- Put the two survey commands (`pub` grep, non-comment diff) in a release
  task's Steps, not just its Context. They are what makes the entry honest and
  are cheap enough to be routine rather than discretionary.
- A DoD loop over every released version's link ref, as here, is the only
  thing that catches a rotted markdown reference block -- nothing in the
  toolchain resolves those targets, which is how `[unreleased]` sat comparing
  from `v0.19.1` across four releases.
- Breadth: single-purpose diff (4 files plus records), no scope creep beyond
  the link-ref repair, which is squarely inside "update the changelog".
- Context: no pressure observed -- no compaction, no checkpoint, no resume.

## Action items

- Ledger: bump `record-numbers-from-a-rerun` to x4 with the fourth occurrence.
- Ledger: new `survey-the-diff-before-the-changelog` (x1).
- Ledger: new `markdown-link-refs-rot-unverified` (x1).
- Ledger: new `a-branch-cannot-tick-a-land-time-proof` (x1).
- USER DECISION OUTSTANDING: `record-numbers-from-a-rerun` needs a disposition
  (PROMOTE with a live task / DEFER at a count / RETIRE / ABSORBED) before
  `tatr check --ledger` can go green. Recorded via `tatr ledger`, not by hand.
