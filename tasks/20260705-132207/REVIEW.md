# Review: breach -- menu + mobile polish

- VERDICT: APPROVE
- ROUNDS: 1

## Verified

- `cargo fmt --check`, `cargo clippy --all-targets` (clean), `cargo test --example
  14_breach` (18 pass), `scripts/check-ascii.sh` (clean).
- Headless `BCS_AUTOPILOT`: full cycle, no panic. `touch_aim_assist` early-returns when
  `touch.fire` is false (the autopilot fires via keyboard, not touch), so it does not
  interfere with the autopilot's own direct-yaw aiming.

## Findings / checks

- Scope was honest: the menu already had a controls hint, pulsing title and best-score
  readout, so this added a touch hint line + a tap-aware begin prompt, and the real
  mobile win -- firing aim-assist.
- `touch_aim_assist` is touch-only by construction (a mouse never sets
  `TouchInput.fire`), so desktop aim is provably untouched. It runs BEFORE `Drive`
  (which does `yaw -= look * sens`), so the player's own look still applies on top; it
  only assists within a frontal `AIM_CONE` so it never yanks the camera off an enemy the
  player is not looking at.
- The shortest-arc `step_angle_toward` is pure and unit-tested, including the +/-pi
  wraparound (the classic bug for angle interpolation) and the cap.

## Deliberately left as-is

- Fire-button / stick sizing: the visual FIRE button and the `read_touch` fire zone are
  two separately-tuned constants; resizing one without the other desyncs tap detection
  from the button, and both are already usable. Left unchanged to avoid introducing that
  class of bug for marginal gain -- noted rather than silently skipped.

## Nits

- Aim-assist itself can only be felt on a real touch device; verified structurally
  (safe, gated, pure math tested), not by a headless touch simulation.
