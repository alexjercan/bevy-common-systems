# Retro: scoring/streak decaying combo counter

- TASK: 20260704-161518
- BRANCH: feat/scoring (squash-merged to master as 31ba654)
- REVIEW ROUNDS: 1 (APPROVE, no findings)

First Wave B item of the input/projection harvest -- and the first
"sketch-then-commit, a negative result is fine" task, which changed the shape of
the work: the deliverable was a *decision* backed by code, not just code.

## What went well

- Did the sketch honestly and let it split the deliverable. Reading both copies
  first (06 `Combo`, 07 `Streak`) surfaced that the mechanic is really two
  things: a genuinely re-derived decay state machine (the `tick -> ended-edge`
  both games hand-rolled with the same count==0/timer footgun) and a trivial
  `Score` counter with no logic. Shipping the first and dropping the second --
  rather than reflexively building everything the task named -- is exactly the
  "does this beat a raw primitive?" discipline the task asked for. The reviewer
  endorsed the split.
- Handled the two games' divergence with composition, not a bloated struct. 07
  uses `Streak` as its resource directly; 06 keeps its game-specific `points`
  tally by *embedding* a `Streak` in its own `Combo`. That kept the game rule
  out of the shared type (the module owns bookkeeping only) without forcing a
  dead `points` field on 07.
- Caught the one behaviour-adjacent change myself before review: the golden
  `extend_to` guards `count > 0` where the old `timer.max()` was unconditional.
  Traced that the old write was dead when inactive, so it is observably
  identical. Semantics-first tracing (last cycle's lesson) paid off again.

## What went wrong

- Nothing of substance; clean single-round APPROVE. The only judgement call was
  test placement: two 06 tests that only exercised the decay were dropped rather
  than converted, because that behaviour now lives (and is tested) in the
  module. Worth stating explicitly in the review so it does not read as
  weakening coverage -- which I did.

## What to improve next time

- For sketch-then-commit tasks, decide the ship/drop split from the evidence
  *before* writing the module, and record the reasoning where it lasts (module
  docs + commit + retro), so the negative-result half is a documented decision
  rather than a silent omission. That is what made this one reviewable in a
  single pass.
- When behaviour moves from an example into a module, say so in the review's
  test section (relocated, not deleted). A dropped example test looks like lost
  coverage until you point at its new home.

## Action items

- [x] `scoring::Streak` shipped; `Score` deliberately dropped (documented
  negative result), 06 and 07 refactored onto it.
- [ ] Wave B continues: radial gravity (tatr 20260704-161522) and `progress`
  (tatr 20260704-161526), both also sketch-then-commit and each a candidate to
  downgrade to a doc. Note: a higher-priority devtools spike
  (`tasks/20260704-175058/SPIKE.md`, tatr 20260704-175421 etc, priority 80+) has
  since landed and outranks the rest of Wave B -- flag to the user which to run
  next.
