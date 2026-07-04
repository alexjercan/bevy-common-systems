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

- [ ] R1.1 (MAJOR) examples/08_dropzone.rs:490-524,468 - the landing pad is a
  single fixed direction spawned once in `setup`. Randomize it each run: pick a
  random azimuth and a polar angle in a reachable band around the +Y spawn pole
  (e.g. 0.18..0.5 rad, well clear of the antipode `from_rotation_arc`
  singularity), spawn the beacon per-run, and update the `LandingPad` resource in
  `start_run`. Keep it visible on the result screen (parked ship on the pad) and
  clean it up on leaving Result alongside the ship.
- [ ] R1.2 (MAJOR) examples/08_dropzone.rs:798-816,1083-1092 - fuel cans are 3
  fixed positions per run. Randomize their positions each run, and maintain
  roughly 3 on the map by spawning replacements over time (never zero, never a
  swarm) - model on `07_orbit`'s maintain-objects system with a spawn timer and a
  target count. `fuel_can_positions` becomes a random-position generator.
- [ ] R1.3 (MAJOR) examples/08_dropzone.rs - there is only a numeric "pad Nm" HUD
  readout; add a direction indicator toward the pad. Prefer a diegetic guide: a
  world-space arrow that hovers by the ship and points along the ground track to
  the pad, updated each frame in `Playing`.

### Polish (independent review)

- [ ] R1.4 (MINOR) examples/08_dropzone.rs:778-816 - `CameraShake` is never reset
  between runs. A crash sets trauma 0.75 (decay 1.6/s), so mashing SPACE on the
  result screen carries residual shake into the next run's first frames. Add
  `shake.trauma = 0.0;` to `start_run`.
- [ ] R1.5 (MINOR) examples/08_dropzone.rs:1118 - the fuel cap
  (`(fuel.0 + FUEL_CAN_AMOUNT).min(START_FUEL)`) was a called-out decision but is
  untested. Extract a pure `add_fuel(cur, amount) -> f32` helper and assert it
  caps at `START_FUEL`.
- [ ] R1.6 (NIT) examples/08_dropzone.rs:~1520 - test
  `landing_score_bullseye_beats_a_far_scrappy_landing`: the "far" case is a
  perfect flight (full fuel, zero speed/tilt), just far and at par, so it is not
  "scrappy". Rename to `..._beats_a_far_slow_landing`.
- [ ] R1.7 (NIT) examples/08_dropzone.rs:1232-1238 - dust spawns at
  `transform.translation` (hull centre) rather than the contact patch under the
  hull. Offset the spawn point down toward the feet (e.g. `at - up * 0.8`).
- [ ] R1.8 (NIT) examples/08_dropzone.rs:~1052-1063 - `ship_start_pos` and
  `drive_chase_camera` doc comments still say the camera falls back to the
  vantage because "there is no ship (Menu / Result)"; after the visible-landing
  change a soft landing keeps the ship through Result. Update the comments.
- [ ] R1.9 (NIT) examples/08_dropzone.rs:1195 - `#[allow(clippy::too_many_arguments)]`
  on `resolve_landing` is redundant (the lint is allowed crate-wide; `setup`/
  `start_run` carry many args without it). Remove for consistency.
