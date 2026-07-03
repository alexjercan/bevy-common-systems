# play-test and tune 08_dropzone flight feel

- STATUS: CLOSED
- PRIORITY: 40
- TAGS: example,polish

Follow-up from `tasks/20260703-165432` (08_dropzone). The example compiles and
passes the full check suite, but the flight constants were tuned by reasoning,
not by actually flying it in a window (the implementing session was headless).
Play-test on a machine with a display and adjust for feel and robustness.

Steps:

- [ ] Run `cargo run --example 08_dropzone` and play several landings and
  crashes. Confirm the PD controller rights the ship crisply without
  oscillation (tune `PD_FREQUENCY` / `PD_DAMPING` / `PD_MAX_TORQUE`).
- [ ] Check `GRAVITY` vs `THRUST_ACCEL` and `FUEL_BURN` give a fun but winnable
  descent (free-fall currently reaches ~15 m/s vs the 4.5 m/s landing limit).
- [ ] Verify the ship rests stably on the bumpy trimesh instead of jittering or
  sliding; if it slides, consider a flatter landing pad, more `AngularDamping`,
  or a friction/restitution tweak.
- [ ] Verify high descent speeds do not tunnel through the thin trimesh
  collider; add `SweptCcd` to the ship if they do.
- [ ] Confirm the menu/result screens frame the planet nicely (the chase camera
  now falls back to `ship_start_pos()`), and the crash fragments read well.
- [ ] Update the constants and, if needed, `docs/2026-07-03-dropzone-example.md`
  with the play-tested values.

See `docs/2026-07-03-dropzone-example.md` (Physics tuning notes / risk) and
`docs/retros/20260703-165432-dropzone-example.md`.
