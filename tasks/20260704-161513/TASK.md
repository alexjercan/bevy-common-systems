# ui/touchpad: reveal-on-first-touch + hit-test primitives (Wave A)

- STATUS: CLOSED
- PRIORITY: 28
- TAGS: spike,feature,ui,input

> Spike: docs/spikes/20260704-161210-input-and-projection-harvest.md (read first). Wave A -- the most-documented duplication in the
> repo (4 touch docs, 3 touch retros). Ship PRIMITIVES, not a fixed pad.

## Goal

Promote the reveal-on-first-touch on-screen control pad PRIMITIVES that three
games hand-roll (`08_dropzone`, `09_reactor`, `11_overload`) into a `ui/touchpad`
module -- deliberately NOT an opinionated pad widget, to stay on the right side
of the "no framework machinery" charter line:

- A `TouchSeen` resource that flips true on the first `Touches::any_just_pressed`
  and a helper to toggle a tagged HUD root's `Visibility` (runtime touch
  detection, the pattern every touch doc argues for over `#[cfg(wasm32)]` and
  `navigator.maxTouchPoints`).
- Pure, unit-testable window-fraction hit-test helpers:
  `button_grid_at(point, window, cols, rows, zone) -> Option<usize>`
  generalizing `vent_button_at` (`examples/11_overload.rs:822`, tested `:1162`),
  and the deflection->stick-offset map from `touch_lean`
  (`examples/08_dropzone.rs:1424`, tested `:2596`).

BAKE IN the frame-derivation lesson both touch retros record: derive from the
live `just_pressed`/pressed set each frame, never latch a touch id, or ports
reintroduce the held-finger leak. NEEDS-A-DECISION at planning (spike open
question): bare gate+hit-test only, vs also a ready-made bottom-strip button-row
builder mirroring `status_bar_item` (more useful, more opinionated). Prove by
refactoring 11_overload's vent pad onto the primitives.
