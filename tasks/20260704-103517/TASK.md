# 08_dropzone: mobile virtual-pad touch controls

- STATUS: OPEN
- PRIORITY: 3
- TAGS: feature,dropzone,mobile

## Goal

Third follow-up from the fun+mobile spike (`tasks/20260704-102022`, read Part 2).
Make `examples/08_dropzone.rs` playable on a phone in the wasm showcase by adding
an on-screen "virtual pad": a touch scheme for thrust and lean. Desktop keyboard
controls stay exactly as-is; touch is an ADDITIONAL writer of `ShipInput`, so the
physics / PD / scoring path is untouched.

Scope = the spike's recommended default (option 1) only. Tilt/accelerometer
(option 2) and on-screen A/D/W/S buttons (option 3) are explicitly NOT in scope
here -- they are possible later opt-in/fallback modes, noted at the bottom.

## Why this scheme (from the spike Part 2)

Our lean is an absolute TARGET ATTITUDE (`ShipInput.lean_pitch/lean_roll`, max
`MAX_LEAN` 0.45, self-levels via `LEAN_DECAY` on release), not a rate. So the
touch input must express an absolute, self-centering magnitude
(deflection-to-position), which also matches research showing position-control
beats rate-control for tilt steering. A floating virtual stick / origin-drag is
literally a 2-axis absolute controller and maps 1:1 onto the existing input.

## Steps

- [ ] Touch lean -- right-side floating stick / origin-drag. On touch-down in the
      steer zone, capture the origin; each frame map the finger offset
      `(dx, dy)` from that origin to `(lean_roll, lean_pitch)` target, magnitude
      clamped to a max radius mapped to `MAX_LEAN`, with a small dead zone at the
      center. Lift finger -> target returns to level (share the existing
      "released = level" convention with the keyboard path). Use a
      floating/draggable origin (re-centers on touch, slides at the extents) so
      the thumb never runs out of room during the flare.
- [ ] Touch thrust -- left-side hold zone. A dedicated thrust region (e.g. left
      third, or a labeled bottom-left pedal): touch/hold in it sets
      `input.thrust = true`. Boolean, trivial.
- [ ] Touch routing (no gesture conflict). Track touches by pointer id via Bevy's
      `Touches` (works under wasm). Route each touch by the zone it STARTED in
      and never re-evaluate a moving touch against zone boundaries, so a lean
      drag that crosses into the thrust zone does not misfire thrust, and vice
      versa. Both thumbs act simultaneously (thrust + steer at once, as a lander
      needs).
- [ ] HUD. Draw the virtual pad: a ghost stick ring at the steer origin (drawn
      even for the drag variant, for discoverability + feedback) and a visible
      thrust zone/pedal. Keep it unobtrusive; it is harmless on desktop so it can
      always show, or gate it on a touch device being present (decide + note).
- [ ] Web/showcase. Ensure the `web/` build exposes a mobile-friendly viewport
      (meta viewport, canvas sizing) so the example is actually usable on a
      phone. Verify through the real entry point (`npm run build`), not a hand-run
      of trunk (AGENTS.md gotcha); fresh worktrees need `npm ci` in `web/` first.
- [ ] Verify: `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test`,
      `./scripts/check-ascii.sh`, then RUN the example on desktop and confirm the
      render loop plus that keyboard STILL works and the touch HUD renders. If a
      touch device / emulation is available, sanity-check thrust+lean via touch.
      Document the scheme and decisions in `docs/`.

## Notes

- Implementation should be minimal: a new touch-input system that writes the same
  `ShipInput` `read_input` writes today, plus HUD nodes. Do NOT change the
  physics, PD controller, gravity/thrust application, or scoring.
- Out of scope (possible future opt-in modes, per spike): device
  tilt/accelerometer (needs an iOS-safe permission button behind a user gesture,
  HTTPS, Android auto-grants; must always keep a touch fallback), and on-screen
  A/D/W/S buttons as an accessibility fallback. All three future modes can share
  this task's two-thumb layout and "released = level" convention.
- Faithful to AGENTS.md: additive, physics path untouched, example is the
  integration test (run it), wasm-friendly, plain ASCII.
- Independent of the fun/hazard tasks (`20260704-103544`, `20260704-103553`);
  can be done in any order relative to them.
