# Review: golden bonus fruit

- TASK: 20260703-140244
- BRANCH: feature/ninja-golden

## Round 1

- VERDICT: APPROVE

Self-review. Golden = flat +5 and extends the combo window (timer.max, so it
only ever lengthens, never shortens an active combo), and does not advance the
combo count - a clean "bonus + extra time" reading. Reuses SlicePop and the
material-inheritance so it bursts gold. Built on the time-window combo (dep
met). Checks clean; verified golden spawn + slice: +5, timer 2.5, no panic.
