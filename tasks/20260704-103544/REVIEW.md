# Review: 08_dropzone Tier-A fun pass

- TASK: 20260704-103544
- BRANCH: feature/08-dropzone-fun

## Round 1

- VERDICT: REQUEST_CHANGES

The Tier-A implementation itself is clean and correct: an independent skeptical
pass found no blockers, verified the landed-ship lifecycle rework (drop
`DespawnOnExit` from spawn, re-add on crash, `despawn_ships` on `OnExit(Result)`),
the great-circle pad distance, the scoring clamps, the camera-shake ordering,
and that the frozen Static hull does not drift. The changes requested below are
(a) the user's added scope during review - make each run fresh and navigable -
and (b) a handful of MINOR/NIT polish items.

### User-added scope (acceptance criteria)

- [x] R1.1 (MAJOR) examples/08_dropzone.rs:490-524,468 - the landing pad is a
  single fixed direction spawned once in `setup`. Randomize it each run: pick a
  random azimuth and a polar angle in a reachable band around the +Y spawn pole
  (e.g. 0.18..0.5 rad, well clear of the antipode `from_rotation_arc`
  singularity), spawn the beacon per-run, and update the `LandingPad` resource in
  `start_run`. Keep it visible on the result screen (parked ship on the pad) and
  clean it up on leaving Result alongside the ship.
- [x] R1.2 (MAJOR) examples/08_dropzone.rs:798-816,1083-1092 - fuel cans are 3
  fixed positions per run. Randomize their positions each run, and maintain
  roughly 3 on the map by spawning replacements over time (never zero, never a
  swarm) - model on `07_orbit`'s maintain-objects system with a spawn timer and a
  target count. `fuel_can_positions` becomes a random-position generator.
- [x] R1.3 (MAJOR) examples/08_dropzone.rs - there is only a numeric "pad Nm" HUD
  readout; add a direction indicator toward the pad. Prefer a diegetic guide: a
  world-space arrow that hovers by the ship and points along the ground track to
  the pad, updated each frame in `Playing`.

### Polish (independent review)

- [x] R1.4 (MINOR) examples/08_dropzone.rs:778-816 - `CameraShake` is never reset
  between runs. A crash sets trauma 0.75 (decay 1.6/s), so mashing SPACE on the
  result screen carries residual shake into the next run's first frames. Add
  `shake.trauma = 0.0;` to `start_run`.
- [x] R1.5 (MINOR) examples/08_dropzone.rs:1118 - the fuel cap
  (`(fuel.0 + FUEL_CAN_AMOUNT).min(START_FUEL)`) was a called-out decision but is
  untested. Extract a pure `add_fuel(cur, amount) -> f32` helper and assert it
  caps at `START_FUEL`.
- [x] R1.6 (NIT) examples/08_dropzone.rs:~1520 - test
  `landing_score_bullseye_beats_a_far_scrappy_landing`: the "far" case is a
  perfect flight (full fuel, zero speed/tilt), just far and at par, so it is not
  "scrappy". Rename to `..._beats_a_far_slow_landing`.
- [x] R1.7 (NIT) examples/08_dropzone.rs:1232-1238 - dust spawns at
  `transform.translation` (hull centre) rather than the contact patch under the
  hull. Offset the spawn point down toward the feet (e.g. `at - up * 0.8`).
- [x] R1.8 (NIT) examples/08_dropzone.rs:~1052-1063 - `ship_start_pos` and
  `drive_chase_camera` doc comments still say the camera falls back to the
  vantage because "there is no ship (Menu / Result)"; after the visible-landing
  change a soft landing keeps the ship through Result. Update the comments.
- [x] R1.9 (NIT) examples/08_dropzone.rs:1195 - `#[allow(clippy::too_many_arguments)]`
  on `resolve_landing` is redundant (the lint is allowed crate-wide; `setup`/
  `start_run` carry many args without it). Remove for consistency.

## Round 2

- VERDICT: APPROVE

Verified every Round 1 finding against the new diff (commit adds pad
randomization, fuel spawner, guide arrow, and the polish fixes):

- R1.1 RESOLVED - pad rolled per run in `start_run` via `random_cap_dir` in
  `[PAD_ANGLE_MIN, PAD_ANGLE_MAX]`, beacon placed flush from `PlanetNoise`,
  `Pad` marker cleaned by `cleanup_run_scene` on `OnExit(Result)`. New test
  `random_pad_dir_stays_in_the_reachable_band` asserts the invariant.
- R1.2 RESOLVED - `random_fuel_can_pos` + `maintain_fuel_cans` keep
  `FUEL_CAN_TARGET` cans, refilling `FUEL_CAN_SPAWN_INTERVAL` after a collect
  (timer primed while full, so no instant/unbounded refill). Test
  `random_fuel_cans_are_reachable_and_above_the_surface`. Full-cycle run held
  the count at 3 for 662 frames.
- R1.3 RESOLVED - `GuideArrow` + `update_guide_arrow` points along the ground
  track to the pad each frame; present every frame in the run.
- R1.4 RESOLVED - `start_run` sets `shake.trauma = 0.0`.
- R1.5 RESOLVED - `add_fuel` helper used in `collect_fuel_cans`; test
  `add_fuel_caps_at_the_starting_tank`.
- R1.6 RESOLVED - renamed `landing_score_bullseye_beats_a_far_slow_landing`.
- R1.7 RESOLVED - dust spawns at `transform.translation - up * 0.8`.
- R1.8 RESOLVED - `ship_start_pos` / `drive_chase_camera` comments updated for
  the visible-landing behaviour.
- R1.9 RESOLVED - redundant `#[allow(clippy::too_many_arguments)]` removed (none
  added to the grown `start_run` either; lint is allowed crate-wide).

Checks re-run clean: fmt, clippy --all-targets, 6 unit tests pass, check-ascii,
example boots to the render loop, web showcase rebuilds. No new issues
introduced. Approving.
