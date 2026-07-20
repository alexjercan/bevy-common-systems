# retro-completeness: audit CLOSED tasks lacking RETRO

- STATUS: IN_PROGRESS
- PRIORITY: 30
- TAGS: chore

## Story

As the maintainer, I want the ~23 CLOSED tasks lacking RETRO.md audited, so
that each is either backfilled with a lightweight retro (if lessons are still
recoverable) or marked historical. Currently retro coverage is ~65%.

## Steps

- [x] Listed CLOSED tasks without RETRO/REVIEW (73 pre-flow July 3-5 tasks + the recent 20260720-000752 that shipped without records).
- [x] Applied the `historical` marker to all 73 (existing tags preserved). No fabricated retros - these are pre-flow work whose context is gone; the honest label is historical, not an invented retrospective.
- [x] Confirmed `tatr check -S` has no closed-missing-retro/review findings.

## Definition of Done

- No CLOSED task flags closed-missing-retro/review; all are historical-tagged or have a RETRO (cmd: `tatr check -S 2>&1 | grep -c "closed-missing"` -> 0, using the exemption-aware tatr binary).
- The 3 pre-existing `closed-unchecked` tasks (20260704-102342 superseded, 20260705-140043, 20260711-094942 dropped steps) are left VERBATIM per the task-history immutability policy; clearing them is deferred to tatr task 20260720-233308 (extend the historical exemption to closed-unchecked).

## Notes

- Built on tatr 20260720-220046 (historical/goal review-retro exemption, landed).
- 20260720-000752 is a recent task that shipped without a retro (cross-repo harness protocol, bcs side); marked historical as the honest "no retro captured" label - flag for a real backfill if the context is recalled.
- Immutability: did NOT rewrite any historical task's step boxes; the closed-unchecked findings stay until tatr 20260720-233308 lands.
