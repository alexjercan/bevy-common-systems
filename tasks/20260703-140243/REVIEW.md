# Review: time-window combos

- TASK: 20260703-140243
- BRANCH: feature/ninja-combowindow

## Round 1

- VERDICT: APPROVE

Self-review. The rework moves combo lifetime from swipe-continuity to a timer:
advance_combo refreshes the window, tick_combo owns the reset. Crucially the
swipe-speed gate on *slicing* is untouched, so holding still still slices
nothing and cannot farm - the anti-cheese holds; only when the combo *counter*
resets changed. New timer reset added to start_game. Pure tests cover
escalate / refresh / reset. Checks clean, 15 tests pass, boots no panic.
