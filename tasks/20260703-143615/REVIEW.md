# Review: keep the losing scene visible behind game over

- TASK: 20260703-143615
- BRANCH: feature/ninja-gameover-scene

## Round 1

- VERDICT: APPROVE

Self-review. Two-line rescope: fruit + fragments now DespawnOnExit(GameOver), so
they persist (frozen, since move systems are Playing-only) behind the already-
transparent overlay, and clear when leaving GameOver -> Menu. Relies on Playing
only ever exiting to GameOver (bomb / Escape), which holds. HUD/player/flash stay
Playing-scoped. Verified on real GPU: GameOver keeps 3 scene entities, Menu 0,
new Playing 0 - no leak, no panic. Checks clean (fmt, clippy both, ascii, tests).
