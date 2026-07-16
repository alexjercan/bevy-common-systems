# 11_overload mobile touch controls

Date: 2026-07-04
Task: `tasks/20260704-130314`

## What this adds

`examples/11_overload.rs` is now playable on a phone in the wasm showcase. The
game was keyboard-only (press 1/2/3/4 to vent the four gauges) and its menu and
meltdown screens only read mouse + keys, so a phone could neither start a run
nor vent a gauge. Touch is added as an ADDITIVE input source, mirroring the
`08_dropzone` virtual-pad pattern; the keyboard path is behaviourally unchanged.

- **On-screen vent pad.** A bottom strip of four labelled buttons (`1 HEAT`,
  `2 PRES`, `3 FLUX`, `4 CHRG`), one per gauge, tinted a distinct hue. Tapping a
  gauge's column vents it, exactly like pressing its number key. The pad is
  hidden until the first touch and then shown for the rest of the session; the
  keyboard legend it replaces is hidden at the same time.
- **Touch menu / meltdown navigation.** `menu_start` and `gameover_dismiss` now
  also accept `Touches::any_just_pressed()`, so a tap starts a run and dismisses
  the meltdown screen. winit-on-web delivers taps as `Touch` events and does not
  synthesize `MouseButton::Left`, so without this a phone could never enter or
  leave a run. (Esc-to-give-up stays keyboard-only.)
- **Web canvas.** `web/games/11_overload/index.html` gains `touch-action: none`
  on `#game-canvas` so touches feed the game instead of scrolling / pinch-zooming
  the page (the viewport already had `user-scalable=no`).

## Key decisions

### Window-fraction hit-test, not Bevy UI `Interaction`

The four vent buttons are purely visual. The touch hit-test is a pure function,
`vent_button_at(point, window) -> Option<usize>`, that maps a touch point to a
gauge column: the live area is the bottom `VENT_ZONE_H_FRAC` (16%) of the window
height, split into `GAUGE_COUNT` equal full-width columns. `spawn_vent_pad`
lays the buttons out over the same fractions (a flexbox row of equal children
filling a bottom strip of the same height), so the visuals line up with the hit
zones by construction.

This deliberately avoids Bevy UI's `Interaction` component, which would have
been the first use of it in the crate. Two reasons:

1. `Interaction` hit-testing goes through `ComputedNode` + `GlobalTransform`,
   whose coordinate/DPI conventions are exactly the "verify the Bevy UI layer
   against source, do not improvise" trap the AGENTS.md gotcha warns about.
   `08_dropzone` already proved that raw `Touches` positions share the window's
   logical pixel space (its zone split is `touch.start_position().x <
   window.width() * FRAC`), so the fraction model is known-good in this Bevy
   version.
2. A pure `vent_button_at` is unit-testable without an ECS world or a window,
   matching the crate's testing convention (pure logic gets in-module tests, ECS
   wiring is exercised by running the example).

### Additive writer, keyboard untouched

`vent_input` (keyboard) and `touch_vent_input` (touch) are separate systems that
both call the same `apply_vent` and the same `trigger_vent_sfx`. The shared SFX
helper was factored out of the old inline block in `vent_input`; its behaviour is
identical (same volume, same "pitch up when the gauge is still in trouble"
speed), so the two input sources sound the same and there is one gauge model with
two sources. The keyboard path's behaviour is unchanged.

### Frame-derived just-pressed avoids the held-finger leak

`touch_vent_input` reads `touches.iter_just_pressed()`, not held touches. The tap
that starts a run (or dismisses the meltdown screen) is a `just_pressed` edge in
the *menu*/*gameover* frame; by the time the state machine is in `Playing` and
the vent pad exists, that same finger -- even if still held -- is `pressed`, not
`just_pressed`, so it never leaks a spurious vent. This is the same
frame-derived-state lesson `08_dropzone` recorded (deriving from the live input
set each frame beats latching event ids).

### Reveal on first touch (runtime detection)

A `TouchSeen` resource flips true the first frame `any_just_pressed` fires;
`update_touch_pad` (running only in `Playing`) then shows the pad and hides the
keyboard legend. So a PC/mouse session never shows the pad and a phone reveals it
the instant a thumb lands -- no `#[cfg(target_arch = "wasm32")]` (which would show
on desktop browsers too) and no JS `navigator.maxTouchPoints` probe. Same
rationale as dropzone.

## Testing / verification

- `vent_button_at` has a unit test covering each column mapping, a miss above the
  strip, edge clamping into the first/last column, and off-window / degenerate
  windows returning `None` instead of panicking. (`cargo test --example
  11_overload`: 10 passed.)
- `cargo fmt --check`, `cargo clippy --example 11_overload`,
  `scripts/check-ascii.sh` all clean.
- Ran headless (`DISPLAY=:0`) with a temporary `OVERLOAD_SMOKE` autopilot that
  drove Menu -> Playing -> GameOver: the render loop was reached and the touch
  systems (`spawn_vent_pad`, `update_touch_pad`, `touch_vent_input`) ran every
  Playing frame with empty touch input without a panic or query conflict. The
  harness was removed before commit.
- NOT verified: real finger gestures (no touch-injection tool in this
  environment). The hit-test math is unit-tested and the layout mirrors it, but
  the strip height (`VENT_ZONE_H_FRAC`) and button feel want a pass on a real
  phone / browser touch-emulator, like the dropzone `*_PX` constants do.
