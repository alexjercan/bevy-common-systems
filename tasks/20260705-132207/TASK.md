# breach -- main-menu and mobile usability polish

- STATUS: CLOSED
- PRIORITY: 40
- TAGS: spike,breach,example,mobile

## Goal

Light polish on `14_breach`'s already-working main menu and mobile support (both
already shipped -- this is refinement, not new construction). Menu: add a controls
hint (WASD/mouse/click, or the touch equivalent) and a clearer start affordance,
keeping the existing pulsing title + best-score readout. Mobile: a touch
usability pass -- button/stick sizing and placement, and consider light aim-assist
for the swarm since "an FPS is the hardest genre for touch" (the example is
candidly desktop-first).

## Notes

- Spike: tasks/20260705-132024/SPIKE.md
- Reuse `ui/menu`, `ui/touchpad`, `input/state`; do not rebuild the state machine.
- Copy Bevy 0.19 UI idioms from an existing example (font_size/TextLayout/
  border_radius/AmbientLight); improvising the visual layer bites repeatedly.
- For a responsive layout that must hold at phone width, use percentage widths,
  not fixed px + flex_wrap (reactor mobile-touch lesson). Verify phone width with
  `BCS_SHOT=390x844 ... --features debug` if `$DISPLAY` is set.
- Verify: `cargo clippy --all-targets`, headless run, then run for real; if
  `$DISPLAY` is set, screenshot the menu at phone width to confirm nothing renders
  below the fold.

## Steps

- [x] **Menu polish.** The desktop controls hint + pulsing title + best-score readout
  already existed; added a touch controls hint line and made the begin prompt
  tap-aware ("Tap or press any key to begin").
- [x] **Touch aim-assist (the real mobile win).** `touch_aim_assist` (pre-Drive): while
  firing on touch, nudge the view yaw toward the nearest enemy already within `AIM_CONE`,
  capped at `AIM_ASSIST_RATE`; Drive then adds the player's own look on top. Touch-only
  (a mouse never sets `TouchInput.fire`), so desktop aim is untouched. Shortest-arc
  `step_angle_toward` is pure + unit-tested (cap + /-pi wraparound).
- [x] **Verify.** `cargo fmt`, `cargo clippy --all-targets`, `cargo test --example
  14_breach`, ascii, headless `BCS_AUTOPILOT`. Fire-button sizing left as-is (already
  usable; changing it risks desyncing the visual button from the `read_touch` fire zone).
  Header + AGENTS note updated.
