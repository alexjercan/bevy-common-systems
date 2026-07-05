# breach -- main-menu and mobile usability polish

- STATUS: OPEN
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

- Spike: docs/spikes/20260705-132024-breach-fun-pass.md
- Reuse `ui/menu`, `ui/touchpad`, `input/state`; do not rebuild the state machine.
- Copy Bevy 0.19 UI idioms from an existing example (font_size/TextLayout/
  border_radius/AmbientLight); improvising the visual layer bites repeatedly.
- For a responsive layout that must hold at phone width, use percentage widths,
  not fixed px + flex_wrap (reactor mobile-touch lesson). Verify phone width with
  `BCS_SHOT=390x844 ... --features debug` if `$DISPLAY` is set.
- Verify: `cargo clippy --all-targets`, headless run, then run for real; if
  `$DISPLAY` is set, screenshot the menu at phone width to confirm nothing renders
  below the fold.
