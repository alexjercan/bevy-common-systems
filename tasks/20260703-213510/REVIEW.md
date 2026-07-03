# Review: play-test and tune 08_dropzone flight feel

- TASK: 20260703-213510
- BRANCH: polish/dropzone-tune

## Round 1

- VERDICT: REQUEST_CHANGES

The tuning is well-evidenced (autopilot telemetry: crisp PD, winnable descent,
no tunneling) and the camera/flame reframing is a clear win. The instrumentation
was fully removed and the rest of the diff checks out. One real correctness gap
in the impact-speed fix.

- [ ] R1.1 (MAJOR) examples/08_dropzone.rs:778 `track_approach_speed` - the
  pre-solve capture is only reliable at one fixed substep per render frame. When
  the frame rate drops below the fixed rate, `FixedMain` runs several times
  (`[FixedUpdate(track), FixedPostUpdate(solve)] x N`) before `Update`. If the
  touchdown resolves on a non-final substep, a later substep's
  `track_approach_speed` overwrites `ApproachSpeed` with the post-collision
  (near-zero) velocity before `resolve_landing` reads it - re-introducing the
  crash-scored-as-landing bug under frame stutter. Fix: capture the speed once
  per render frame before the fixed loop runs (move the system to `PreUpdate`),
  so no physics substep can clobber it; update the comment/doc claim accordingly.
  - Response:
- [ ] R1.2 (NIT) examples/08_dropzone.rs - the `ApproachSpeed` doc comment states
  the value "holds the speed the ship was actually travelling at just before
  contact" as an unconditional guarantee; reword to match whatever capture
  strategy R1.1 settles on.
  - Response:

Responses to Round 1:

- R1.1: Fixed. `track_approach_speed` moved from `FixedUpdate` to `PreUpdate`, so
  it captures the ship speed once per render frame before the entire
  fixed-physics loop. No collision substep can overwrite the value with a
  post-impact reading; it predates all of the frame's collisions.
- R1.2: Fixed. The `ApproachSpeed` doc comment, the `track_approach_speed` doc,
  and the design-doc note now describe the `PreUpdate` once-per-frame capture and
  its robustness to multi-substep frames.

## Round 2

- VERDICT: APPROVE

- [x] R1.1 (MAJOR) - resolved: capture moved to `PreUpdate` (examples/08_dropzone.rs,
  `add_systems(PreUpdate, track_approach_speed...)`), verified it predates the
  fixed loop so no substep clobbers it. Build + boot test clean (reaches the
  render loop, no panic).
- [x] R1.2 (NIT) - resolved: docs/comments reworded to match the new strategy.

Checks green: fmt, check-ascii, clippy (default + debug), example builds and
boots. No new findings introduced by the fix.
