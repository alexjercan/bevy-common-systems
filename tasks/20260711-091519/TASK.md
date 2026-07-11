# PD controller cannot damp a fast off-principal spin

- STATUS: OPEN
- PRIORITY: 95
- TAGS: bug,physics,pd

## Goal

A rigid body released with a fast residual spin (~1.5 rad/s, e.g. roll about
a ship's nose) is never despun by `compute_pd_torque` even when the command
tracks the body's attitude exactly (pure damper, P contribution zero). Slower
spins (~0.7 rad/s) damp fine. Downstream evidence lives in nova-protocol task
20260709-125640. Deliver a PD whose damping term provably removes rotational
energy at any spin rate, with tests that fail on the current code.

## Steps

- [ ] Add closed-form unit tests for `compute_pd_torque` in
      `src/physics/pd_controller.rs`: for the pure-damper case
      (`from_rotation == to_rotation`, angular velocity below clamp
      saturation) the output must equal `-kd * I_world * omega`, where
      `I_world = (from_rotation * inertia_local_frame) * diag(principal) *
      (from_rotation * inertia_local_frame)^-1`. Cover (a) non-identity
      `inertia_local_frame` with identity body rotation, (b) non-identity
      body rotation with identity local frame, (c) both non-identity, and
      (d) a saturated case asserting the clamp preserves direction.
- [ ] Add a headless avian integration test (in-module, full `App` with
      avian's physics plugins, like nova's flight tests): a dynamic body
      whose collider layout gives a symmetric-top inertia with a
      NON-identity principal local frame (e.g. offset child colliders),
      spun at 1.5 rad/s about its long axis, PD input synced to the body's
      `Rotation` every tick before the PD system, output applied via avian
      `Forces::apply_torque` exactly as nova does. Assert the spin decays
      below 0.1 rad/s. Add a 0.7 rad/s control case. This reproduces the
      bug at the crate boundary; if it does NOT reproduce, the bug is in
      how nova applies the torque - record that finding in the nova task
      and stop here.
- [ ] Diagnose with those tests in hand and fix `compute_pd_torque`.
      Leading candidate: the frame composition order at line 136
      (`inertia_local_frame * from_rotation` - world-from-principal should
      be `from_rotation * inertia_local_frame`). Note: even the wrong
      order yields a symmetric positive-definite tensor, so a pure damper
      should still drain energy in theory - if the repro shows sustained
      spin, look past the order at the clamp interaction, axis extraction,
      or torque application, and document the actual mechanism in a code
      comment on the fix.
- [ ] Document the inertia-sandwich math (what frame each factor lives in)
      in a comment block in `compute_pd_torque`, so the next reader can
      check the order by inspection.
- [ ] Run the full bcs check suite (fmt --check, clippy --all-targets with
      and without --features debug, cargo test, cargo test --features
      debug, cargo test --examples, scripts/check-ascii.sh).

## Notes

- Consumer evidence (nova-protocol tasks/20260709-125640): released hull at
  1.5 rad/s spins forever at constant rate; 0.7 rad/s damps; a direct
  `Forces::apply_torque(-spin * 40)` despin attempt in nova made the spin
  GROW (1.5 -> 3.2 rad/s), so keep application-side sign/frame surprises on
  the suspect list, not just the PD math.
- Relevant files: src/physics/pd_controller.rs (all of it), nova's consumer
  seam: nova-protocol crates/nova_gameplay/src/sections/controller_section.rs
  (sync_controller_section_forces).
- nova-protocol will bump its pinned git rev after this lands (its task
  20260709-125640 depends on this one).
