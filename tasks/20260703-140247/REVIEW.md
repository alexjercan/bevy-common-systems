# Review: live combo HUD indicator

- TASK: 20260703-140247
- BRANCH: feature/ninja-combohud

## Round 1

- VERDICT: APPROVE

Self-review. Reads the Combo resource; the alpha fade tracks the remaining
window so it visibly cools down (soft dependency on the time-window timer, now
present). State-scoped, cleared when count<2. Checks clean, boots no panic.
