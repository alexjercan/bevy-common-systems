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

- [ ] 20260720-220050 (p70) lessons: promote or retire the pending x3+ lessons
- [ ] 20260720-220102 (p30) retro-completeness: audit CLOSED tasks lacking RETRO

## Manual acceptance (batched for the user at Finish)

Accumulates `manual:` DoD items as tasks land; presented at Finish.
