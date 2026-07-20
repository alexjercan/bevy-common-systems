# Retro: Adopt flow v2 (bevy-common-systems)

- TASK: 20260720-171843
- BRANCH: chore/flow-v2-adoption (landed as 0ae11b0 via sprout land)
- REVIEW ROUNDS: 1 (out-of-context APPROVE, 1 NIT taken)

## What went well

- The ledger restructure moved 24 entries byte-identically - verified by
  the reviewer via sorted-line comparison, the strongest possible check
  against silent history edits.
- The reviewer independently reproduced master's 11 bad-severity findings
  before accepting the 11/11 fix claim, and corrected the work report's
  tick count (17 claimed, 15 in the diff) - counts belong to the diff, not
  the narrative.
- Residue reasoning held under adversarial reading: superseded, unevidenced
  and falsified-premise boxes all stayed honest.

## What went wrong

- The work agent's own report miscounted its ticks; harmless here because
  the reviewer counted, but narratives drift - close-outs should cite the
  diff numbers.

## Action items

- [x] Residue (3 tasks, 9 boxes) forwarded to the umbrella GOAL.md.
