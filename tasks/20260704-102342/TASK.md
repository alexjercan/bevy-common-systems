# 08_dropzone: landing target + fuel pickups + touch controls

- STATUS: OPEN
- PRIORITY: 3
- TAGS: feature,dropzone

## Goal

The "recommended first slice" from the fun+mobile spike (`tasks/20260704-102022`,
read it first). Make `examples/08_dropzone.rs` a real game and playable on a
phone in one cohesive cycle, without touching the physics/PD/state machine:

1. A glowing landing target on the surface with a positional score term (land
   close = more points), reusing the `07_orbit` marker look.
2. Fuel-can pickups on the descent line: fly through one to add `Fuel` and get a
   "+FUEL" popup + `pickup` sound. Place them slightly off the efficient line so
   grabbing them is a real risk/reward routing choice.
3. Touch controls (spike Part 2, option 1): a left-side thrust hold-zone and a
   right-side floating-stick / origin-drag lean, deflection-to-position mapping,
   as an ADDITIONAL writer of the existing `ShipInput` (keyboard path stays).
   Route each touch by the zone it started in, tracked by pointer id via Bevy's
   `Touches`. Draw a ghost stick ring for feedback; small dead zone; clamp
   deflection to `MAX_LEAN`.

## Steps

- [ ] A1: spawn a landing target entity; add a `proximity_bonus` term to
      `landing_score` (or its caller) based on touchdown distance to the target.
      Show the target on the HUD / with a beacon.
- [ ] A2: spawn fuel-can pickups; detect fly-through (avian sensor or distance
      check); add fuel, cap at `START_FUEL` or allow overfill (decide + note);
      "+FUEL" popup + `pickup` sound.
- [ ] Touch input system writing `ShipInput` (thrust zone + floating-stick
      lean), sharing the "released = level" convention with the keyboard path.
      Draw the touch HUD (ring + thrust zone).
- [ ] Optional light juice: pickup popup, camera punch on touchdown.
- [ ] Verify: `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test`,
      `./scripts/check-ascii.sh`, then actually RUN the example (desktop) and
      confirm it reaches the render loop and the new mechanics work. If a display
      is available, sanity-check the touch HUD renders. Rebuild the web/wasm
      showcase entry via `npm run build` and note mobile viewport behaviour.

## Notes

- Keep it faithful to AGENTS.md: additive, physics path untouched, example is
  the integration test (run it, do not just build it), stay wasm-friendly.
- Bigger follow-ons (obstacles B1, wind B4, multi-leg refuel B2,
  tilt/accelerometer opt-in mode) are separate tasks; do not scope-creep this
  slice into them.
- Distance-for-points from the original request is realized as A1
  (close-to-a-hard-target), which the spike argues is better than rewarding raw
  distance (that would reward flying off to nowhere).
