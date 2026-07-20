# Retro: promote the pending x3+ lessons

## What went well

- Most of the backlog was already institutionalized - the disposition was to
  VERIFY and annotate, not re-invent: global ~/AGENTS.md already carried the
  agent-hygiene process lessons (full-command-output, pkill-by-pid,
  evidence-before-claim), the flow skills carried others (regression-test ->
  review, sprout-first -> work, verify-api-in-source -> plan), and bevy's own
  AGENTS.md already documented most Bevy-domain lessons. Only 5 genuinely-missing
  domain lessons needed folding into AGENTS.md.
- Verified each "already covered" claim by grepping AGENTS.md/global AGENTS.md
  before annotating, so no lesson was marked PROMOTED with a home it lacks. The
  out-of-context reviewer re-verified every annotation (including the skill files)
  and found no false promotions.
- Found the estimate was off (24 pending, not 14) and handled all 24.

## What went wrong

- `tatr check --ledger` is not fully clean: 3 pre-existing `closed-unchecked`
  findings remain on unrelated CLOSED tasks (SUPERSEDED / dropped-step work).
  Those are not ledger-promotion issues; scoped to task 20260720-220102 and the
  DoD proof was narrowed to `promotion-stalled` count = 0.

## What to improve next time

- "Promote the pending lessons" is mostly an audit: check where each lesson is
  already enforced before writing new prose. Duplicating an already-documented
  rule bloats AGENTS.md; annotate the ledger to point at the existing home and
  only fold the genuinely-missing few.

## Action items

- [x] 24 lessons promoted; 5 folded into AGENTS.md; Pending promotions empty; landed 28660c2.
- Carry to 220102: also clear the 3 closed-unchecked findings while doing retro-completeness.
