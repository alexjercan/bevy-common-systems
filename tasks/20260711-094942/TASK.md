# Stabilize PD for large gain*dt: backward-Euler gain conditioning

- STATUS: OPEN
- PRIORITY: 95
- TAGS: bug,physics,pd

## Goal

`compute_pd_torque`'s discrete update is unstable when gain * dt is large:
with nova's shipped tuning (frequency 4, damping 4 -> kd = 72) at 64 Hz,
kd * dt > 1, and when the clamp saturates, one tick's torque impulse
(max_torque * dt / I) can exceed twice the spin it opposes - the body
overshoots through zero to a mirrored state every tick and locks into a
period-2 bang-bang limit cycle that never decays. Diagnosed with a tick
trace in nova task 20260709-125640: post-release the spin sits at a
constant 1.414 rad/s while flipping sign EVERY tick, output saturated at
max_torque and exactly opposing the previous tick's spin, frozen 0.4 rad
attitude error keeping the demand pinned at the clamp. Deliver a PD that
is stable for any frequency / damping_ratio / timestep combination.

## Steps

- [ ] Add a failing integration repro to src/physics/pd_controller.rs
      tests: the existing ship-like body with `max_torque: 100.0` (nova's
      test-rig torque budget; 100 * dt / I_roll = 3.1 > 2 * 1.5 so the
      overshoot condition holds), spin 1.5 rad/s about the long axis,
      command frozen at the release attitude; assert despin below
      0.1 rad/s within 30 s of sim. Must FAIL on current code with a
      sustained ~1.4 rad/s limit cycle.
- [ ] Make `compute_pd_torque` dt-aware and condition the gains with the
      backward-Euler (implicit) form used for stable rigid-body PDs:
      `g = 1 / (1 + kd * dt + kp * dt * dt)`, `kp' = kp * g`,
      `kd' = (kd + kp * dt) * g`, then `raw = axis * (kp' * angle) -
      angular_velocity * kd'`. Pass dt from `update_controller_root_torque`
      via `Res<Time>` (the system runs in FixedUpdate, so this is the fixed
      timestep). Document the conditioning and why (discrete stability for
      any gains) in the comment block.
- [ ] Update the closed-form pure-damper oracles to the conditioned gain:
      expected torque is `-(kd + kp * dt) * g * I_world * omega`.
- [ ] Run the full bcs check suite (fmt --check, clippy --all-targets with
      and without --features debug, cargo test, cargo test --features
      debug, cargo test --examples, scripts/check-ascii.sh).
- [ ] Behavior of the only in-crate consumer: boot 08_dropzone headless
      under the autopilot harness (BCS_AUTOPILOT=1 cargo run --example
      08_dropzone --features debug, under timeout) and confirm the cycle
      completes without panic - the conditioned gains soften the response
      slightly and the lander must still fly.

## Notes

- Follows 20260711-091519 (frame order fix, landed as 13e33e5).
- Downstream: nova-protocol task 20260709-125640 bumps its pinned rev after
  this lands; its regression guard tightens from spin < 2.0 to < 0.5.
- Why the release case only: mid-maneuver the attitude error dominates and
  saturation-driven bang-bang IS the desired slew behavior; the cycle only
  locks when the error is small-and-frozen while the spin is fast. nova's
  in-game corkscrew at max_torque 40 sustains ~0.6 rad/s, matching
  40 * dt / I = 1.25 = 2 * 0.625.
- The clamp itself stays: with conditioned gains the demand near release
  stays under the clamp, so saturation overshoot does not arise there.
- Reference: the "stable backward PD" rigid-body controller formulation
  (implicit-Euler gain conditioning), standard in game physics writing.
