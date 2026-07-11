# Review: Stabilize PD for large gain*dt: backward-Euler gain conditioning

- TASK: 20260711-094942
- BRANCH: fix/pd-backward-euler-gains

## Round 1

- VERDICT: REQUEST_CHANGES

The falsified-premise close-out is well evidenced (three experiments, the
decisive one an A/B via cargo path patch) and closing without a controller
change is the right call. One honesty defect:

- [x] R1.1 (MAJOR) tasks/20260711-094942/TASK.md (Outcome) and the squash
  message draft claim the new saturating-budget test "locks into the
  flip-flop on the old order, so it now also discriminates the frame fix
  in the saturated regime". Verified by reverting the composition order in
  place: the test PASSES on the old order too - a pure z-roll with z-only
  body rotation stays in the commuting subspace even when the clamp
  saturates, exactly as the same file's analysis of the OTHER tests says.
  The test is saturation coverage, not a discriminator; fix the Outcome
  text and the test's doc comment accordingly.
  - Response: reworded the TASK.md Outcome and the test doc comment to
    "saturation coverage only", stating the commuting-subspace reason and
    pointing at the both-frames closed form as the discriminator. The
    squash message will carry the corrected wording. Fixed in the round-2
    commit.

## Round 2

- VERDICT: APPROVE

Verified: the "discriminates the frame fix" claim is gone from TASK.md and
the test doc comment now says saturation-coverage-only with the
commuting-subspace reason. Checks green. No new findings.
