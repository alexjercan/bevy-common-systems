# PD controller cannot damp a fast off-principal spin

- STATUS: CLOSED
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

- [x] Add closed-form unit tests for `compute_pd_torque` in
      `src/physics/pd_controller.rs`: for the pure-damper case
      (`from_rotation == to_rotation`, angular velocity below clamp
      saturation) the output must equal `-kd * I_world * omega`, where
      `I_world = (from_rotation * inertia_local_frame) * diag(principal) *
      (from_rotation * inertia_local_frame)^-1`. Cover (a) non-identity
      `inertia_local_frame` with identity body rotation, (b) non-identity
      body rotation with identity local frame, (c) both non-identity, and
      (d) a saturated case asserting the clamp preserves direction.
- [x] Add a headless avian integration test (in-module, full `App` with
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
- [x] Diagnose with those tests in hand and fix `compute_pd_torque`.
      Leading candidate: the frame composition order at line 136
      (`inertia_local_frame * from_rotation` - world-from-principal should
      be `from_rotation * inertia_local_frame`). Note: even the wrong
      order yields a symmetric positive-definite tensor, so a pure damper
      should still drain energy in theory - if the repro shows sustained
      spin, look past the order at the clamp interaction, axis extraction,
      or torque application, and document the actual mechanism in a code
      comment on the fix.
- [x] Document the inertia-sandwich math (what frame each factor lives in)
      in a comment block in `compute_pd_torque`, so the next reader can
      check the order by inspection.
- [x] Run the full bcs check suite (fmt --check, clippy --all-targets with
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

## Outcome (2026-07-11)

What changed:

- Fixed the inertia frame composition in `compute_pd_torque`
  (src/physics/pd_controller.rs): principal-to-world is
  `from_rotation * inertia_local_frame`; the code had the product reversed.
  Verified against bevy_heavy's `new_with_local_frame` convention
  (I_local = L diag L^-1). Documented the sandwich in a comment.
- Added closed-form pure-damper unit tests (torque == -kd * I_world * omega)
  for local-frame-only, rotation-only and both-frames cases, plus a
  clamp-preserves-direction test. Only the both-frames case distinguishes the
  two orders (with either frame identity they coincide) - it failed before
  the fix and passes after.
- Added four avian integration despin tests mirroring nova's wiring (input
  sync before PDControllerSystems::Sync, Forces::apply_torque after, ship-like
  symmetric-top body of three unit cuboids, PD 4.0/4.0/40.0): tracking command
  at 1.5 rad/s, frozen command at 1.5 rad/s, frozen command at 0.7 rad/s,
  and (review round 1) a skewed body with a verified NON-identity principal
  frame spinning off-principal - the only integration case that runs the
  corrected composition. Honesty note: a pure damper drains energy under
  either composition order (any quaternion sandwich yields an SPD tensor),
  so the integration tests are end-to-end sanity checks, not the
  discriminating regression test - that is the both-frames closed-form
  unit test, which is also the only test whose oracle comes from the
  dependency (ComputedAngularInertia::new_with_local_frame().rotated()).

Key finding: the integration repro does NOT reproduce nova's corkscrew - all
three despin scenarios pass even on the pre-fix code. The sustained-spin bug
reported in nova task 20260709-125640 is therefore in how nova feeds or
applies the PD (its command shaping or torque application path), not in this
crate's math. The order bug fixed here is real but only affects bodies whose
principal frame differs from the body frame AND whose rotation is off the
principal axes; nova's aligned test ship never saw it.

Difficulties:

- Test harness: avian needs AssetPlugin + MeshPlugin (its collider cache reads
  AssetEvent<Mesh> even for primitive colliders) and `app.finish()` (its
  diagnostics resources are initialized in Plugin::finish). Both hit as
  runtime system-param validation failures with unnamed systems; the `debug`
  feature (`bevy/track_location`) names them.

Reflection: writing the closed-form tests BEFORE the fix was the right call -
the single-frame cases passing on buggy code proved the old test coverage
could never have caught this, and the repro-first discipline stopped a
plausible-but-wrong "this fixes nova" claim: the honest result is "bcs math
bug fixed, nova bug lives elsewhere".
