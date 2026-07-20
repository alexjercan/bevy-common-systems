# Goal: resolve the pending-promotion ledger backlog and retro-completeness in bevy

- DATE: 20260720
- UMBRELLA TASK: 20260720-232058
- LANDING SCOPE: squash-merge each task to local master; do NOT push (user's call).

## Goal

bevy-common-systems has the largest pending-promotion backlog of the six repos
(~24 x3+ lessons sitting undecided) and ~65% retro coverage. This run resolves
both: every x3+ pending lesson is dispositioned (promoted with a concrete home -
global AGENTS.md rule, a flow skill, or bevy's own AGENTS.md conventions - or
retired), and every CLOSED task lacking a RETRO is either given one or marked
`historical` so strict check is clean.

## Done means

1. Every x3+ pending lesson is annotated promoted or retired; ledger lints clean (cmd: `tatr check --ledger LESSONS.md`).
2. Domain lessons that were promoted into bevy's AGENTS.md are actually present there (manual: reviewer spot-checks AGENTS.md against the promoted entries).
3. Every CLOSED task has a RETRO.md or a `historical` tag; strict check is clean (cmd: `tatr check -S`, using the exemption-aware tatr binary).

Overall: `tatr check`, `tatr check --ledger LESSONS.md`, and `tatr check -S` all clean; `cargo clippy --all-targets` green (no code touched, but confirm the tree still builds).

## Tasks

- [x] 20260720-220050 (p70) lessons: promote or retire the pending x3+ lessons
      landed 28660c2; 1 review round (APPROVE, no findings). 24 lessons promoted
      (not 14); 5 domain lessons folded into AGENTS.md; Pending promotions empty.
      Discovered 3 pre-existing closed-unchecked findings -> folded into 220102.
- [x] 20260720-220102 (p30) retro-completeness: audit CLOSED tasks lacking RETRO
      landed 74081af; 1 review round (APPROVE, 1 NIT: incidental TAGS whitespace).
      73 pre-flow tasks tagged historical; no fabricated retros; closed-missing = 0.
      3 closed-unchecked left verbatim, deferred to tatr 20260720-233308.

## Manual acceptance (batched for the user at Finish)

- (pending) 20260720-220050: skim the AGENTS.md "Promoted ledger lessons" block
  and the LESSONS.md PROMOTED annotations - confirm the promotions land where you
  want them.
- (pending) 20260720-000752: a recent task marked historical for lack of a
  captured retro - decide whether to backfill a real retro if you recall the
  context, or leave it historical.

## Done-definition status

All 3 criteria met. `cargo clippy --all-targets` exits 0 (no Rust touched);
`tatr check --ledger` has 0 promotion-stalled; `tatr check -S` has 0
closed-missing-review/retro. Residue: 3 pre-existing `closed-unchecked` findings
(20260704-102342 superseded, 20260705-140043, 20260711-094942 dropped steps) are
left VERBATIM per the history-immutability policy and deferred to tatr task
20260720-233308 (extend the historical exemption to closed-unchecked). This
umbrella (goal-tagged) is exempt from -S's own record rules.
