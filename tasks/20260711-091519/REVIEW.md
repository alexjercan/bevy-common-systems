# Review: PD controller cannot damp a fast off-principal spin

- TASK: 20260711-091519
- BRANCH: fix/pd-fast-spin-damping

## Round 1

- VERDICT: REQUEST_CHANGES

Verified independently: the composition-order fix matches bevy_heavy's
`new_with_local_frame` convention (I_local = L diag L^-1, so
principal-to-world is R * L); the full check suite (fmt, clippy both
configs, test both configs, test --examples, check-ascii) is green on the
branch; the both-frames closed-form test fails on master and passes here.
The only other in-crate PD consumer (examples/08_dropzone) uses a single
aligned body, so its behavior is unchanged by the fix.

- [x] R1.1 (MAJOR) src/physics/pd_controller.rs:398 - the integration body
  is three axis-aligned cuboids on the z-axis, so its inertia tensor is
  diagonal and the principal local frame is IDENTITY: the integration layer
  never executes the code path this branch fixes. The planned step (ticked
  in TASK.md) explicitly required "a NON-identity principal local frame
  (e.g. offset child colliders)". Either add a despin case whose colliders
  are offset off-axis (e.g. children at (0.6, 0.5, z)) with an off-principal
  initial spin, or amend the step and the Outcome to record the deviation
  and its rationale (fidelity to nova's evidence rig). Note for honesty
  either way: a pure damper with the OLD order still drains energy (any
  quaternion sandwich yields an SPD tensor), so the integration test cannot
  discriminate the fix - the closed-form both-frames unit test is the real
  regression guard, and the ticked step's claim should not imply otherwise.
  - Response: added `off_principal_spin_despins_on_a_skewed_body` with
    transverse offsets varying along z and an assertion that the computed
    principal frame is actually non-identity; the test comment and the
    TASK.md Outcome now both state explicitly that the integration layer is
    sanity coverage and the both-frames closed-form test is the
    discriminator. Fixed in the round-2 commit.
- [x] R1.2 (MINOR) src/physics/pd_controller.rs:169 - `world_inertia`
  re-derives the same `rotation * local_frame` composition the fix uses, so
  a shared misreading of the convention would pass both sides. Build the
  oracle from the dependency instead:
  `AngularInertiaTensor::new_with_local_frame(principal, local_frame).rotated(rotation)`
  (bevy_heavy via avian), so the test pins the convention to the library
  that defines it.
  - Response: `world_inertia` now builds the oracle via
    `ComputedAngularInertia::new_with_local_frame(principal, local_frame)
    .rotated(rotation).tensor()`. Fixed in the round-2 commit.
- [x] R1.3 (NIT) src/physics/pd_controller.rs:437 - the 600 / 1800 update
  counts are unexplained; a short "10 s / 30 s of sim at 60 Hz" comment
  saves the next reader the arithmetic.
  - Response: added "10 s / 30 s of sim at 60 Hz" comments to all four
    despin loops. Fixed in the round-2 commit.

## Round 2

- VERDICT: APPROVE

Verified each round-1 response against the new diff: the skewed-body test
exists, asserts its principal frame is non-identity before spinning, and
passes; the closed-form oracle is now dependency-built; the duration
comments are in place. Full suite green (fmt, clippy both configs, test
both configs, test --examples, check-ascii). No new findings.
