# Review: 12_bastion defend-the-core tower defense

- TASK: 20260704-220736
- BRANCH: feature/12-bastion

## Round 1

- VERDICT: APPROVE

The diff delivers the Goal: a working tower defense that genuinely exercises all
three target modules -- `camera/project` (`pointer_on_plane` in `placement_point`,
`world_to_screen` in `on_enemy_killed`), `transform/point_rotation` (orbit rig),
and `transform/smooth_look_rotation` (turret slew, applied from the plugin's
output). Check suite is green (clippy both configs, fmt, 5 tests, ascii), the
wasm build succeeds, and the autopilot run + a live screenshot confirmed the loop
(place -> kill -> reward -> Core damage). Verified the highest-risk correctness
question -- the Core revives on a second run because `HealthZeroMarker` is the
only "destroyed" gate and `start_game` removes it and resets `current`.

No BLOCKER or MAJOR findings. Two MINORs worth fixing and two NITs below; the
MINORs are being addressed before merge (flow is driving), the NITs are left to
discretion.

- [x] R1.1 (MINOR) examples/12_bastion.rs:~260 (`orbit_camera` registration) -
  `orbit_camera` runs `run_if(not(in_state(GameState::GameOver)))` and writes the
  rig's `PointRotationInput` from per-frame deltas. Because it stops running in
  GameOver, the last nonzero input is left in place and `PointRotationPlugin`
  keeps integrating it every `PostUpdate`, so the camera spins continuously
  behind the game-over screen. Fix: run `orbit_camera` in all states (drop the
  `run_if`), so with no input the written delta is zero and the view holds; the
  frozen battlefield stays orbit-able on the game-over screen, which is fine.
  - Response: fixed -- `orbit_camera` now runs unconditionally (dropped the
    `run_if`); the `.after(orbit_camera)` edge on the Playing systems still holds.
    Re-verified: autopilot cycle clean, no panic.
- [x] R1.2 (MINOR) examples/12_bastion.rs (`upgrade_selected` + tests) - the
  `upgrade_cost_scales_with_level` test asserts `x*1 == x` and `x*3 == x*3`,
  which are tautologies that test nothing (exactly the "aspirational test"
  anti-pattern the repo's own retros call out). The upgrade-cost formula
  (`upgrade_cost * level`) lives inline in `upgrade_selected` and is untested.
  Extract `fn upgrade_cost(spec_idx: usize, level: u32) -> u32`, call it from
  `upgrade_selected`, and assert real values (e.g. Gun L1->2 = 30, L2->3 = 60).
  - Response: fixed -- added `upgrade_cost(spec_idx, level)` (used by
    `upgrade_selected`) and `wave_size(n)` (used by `advance_waves`), and rewrote
    both tests to assert real values against them. 5 tests still pass.
- [ ] R1.3 (NIT) examples/12_bastion.rs (turret mesh) - the turret is a
  `Cuboid::new(1.4, 0.35, 0.35)` centered on its pivot, so the barrel visually
  extends equally both ways rather than pointing forward. Offset the mesh +X
  (a child transform, or model it as a `TriangleMeshBuilder::new_cone`) so it
  reads as a forward barrel. Cosmetic only.
  - Response:
- [ ] R1.4 (NIT) examples/12_bastion.rs (`menu_click`) - starting on any
  `pointer.just_pressed` means a drag meant to orbit the menu also starts the
  run. Matches the other examples' "tap to play", so acceptable; could gate on
  `released_tap` if menu-orbit is wanted. Leave-it.
  - Response:

### Round 1 resolution

Both MINORs (R1.1, R1.2) addressed and verified by the reviewer; NITs R1.3/R1.4
left to discretion (cosmetic / matches other examples). Verdict stands: APPROVE.
