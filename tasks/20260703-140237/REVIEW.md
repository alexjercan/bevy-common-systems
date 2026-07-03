# Review: screen shake and bomb impact feedback

- TASK: 20260703-140237
- BRANCH: feature/ninja-shake

## Round 1

- VERDICT: APPROVE

Self-review. `apply_camera_shake` decays trauma (squared) and offsets the
MainCamera from CAMERA_BASE, snapping back at rest; runs in all states so the
camera always settles. Fruit/bomb slices bump trauma. The bomb beat routes
through `DyingTimer` (delayed GameOver) + a faded red `Node`, leaving the
Escape give-up instant. New per-run state (trauma, dying) reset in start_game
per the standing lesson. Checks clean (fmt, clippy both, ascii). Verified on
real GPU: Playing -> sliced bomb -> beat -> GameOver, no panic.

- Minor accepted: fruit sliced during the ~0.35s beat still score; harmless.
