# 08_dropzone mobile virtual-pad touch controls

Date: 2026-07-04
Task: `tasks/20260704-103517` (from spike `tasks/20260704-102022`, Part 2 option 1)

## What this adds

`examples/08_dropzone.rs` is now playable on a phone: an on-screen virtual pad
maps touch to the same steering the keyboard drives, so the wasm showcase build
is usable without a keyboard. Desktop keyboard controls are unchanged; touch is
an ADDITIONAL writer of `ShipInput`, and the physics / PD controller / scoring
are untouched.

- **Left half (`THRUST_ZONE_FRAC` = 40% width) = thrust.** Hold anywhere in the
  left zone to fire the main thruster. Boolean, exactly like Space/Up.
- **Right side = a floating steer stick.** Touch down anywhere on the right and
  that point becomes the stick origin; dragging deflects the lean. Deflection
  maps to an absolute lean target (`touch_lean`): magnitude scales linearly from
  zero at the dead zone (`STEER_DEAD_PX`) to `MAX_LEAN` at the radius
  (`STEER_RADIUS_PX`), clamped past it. Drag right rolls right (like `D`), drag
  down pitches back (like `S`). Lift the finger and the lean self-levels via the
  existing `LEAN_DECAY` path. The origin slides to follow the thumb once it
  reaches the radius, so you never run out of stick during a landing flare.

## Why this scheme (from the spike)

Our lean is an absolute TARGET ATTITUDE, not a rate, so the touch input is
deflection-to-position (stick offset -> target angle), which the spike's
research argued beats rate control for tilt steering and maps 1:1 onto
`ShipInput.lean_pitch/lean_roll`. Tilt/accelerometer and on-screen A/D/W/S
buttons were explicitly out of scope (possible later opt-in modes); all three
can share this task's two-thumb layout and "released = level" convention.

## Key decisions

### Additive writer, not a rewrite

`read_input` now reads a `TouchControl` resource alongside the keyboard:
`thrust = key_thrust || touch.thrust`, and the lean target is the touch stick's
value while a steer touch is held, else the keyboard's. Both share the one
smoothing/self-levelling path, so there is a single steering model with two
input sources and the keyboard behaviour is byte-for-byte unchanged when no
touch is active.

### Routing by starting zone, derived fresh each frame

`update_touch_control` distils raw `Touches` into `TouchControl` every frame from
the *currently pressed* touches, keyed by where each one STARTED (`start_position`
vs the zone split), not by latching a single pointer id. Thrust is "any pressed
touch that started in the left zone"; the steer stick keeps its tracked touch
while it stays pressed and right-zone, otherwise adopts another right-zone touch
(re-centring the floating origin). A moving touch is never re-classified, so a
lean drag that wanders left never misfires thrust and vice versa; and because the
state is frame-derived, a second finger in a zone and a finger held across a run
restart both keep working (an earlier id-latched version cut input in those
cases). Both thumbs act at once, as a lander needs.

### Touch navigation of menu / result

The flight controls are only half the "playable on a phone" goal: `menu_input`
and `result_input` also accept `Touches::any_just_pressed()` so a tap starts and
retries a run. winit-on-web delivers taps as `Touch` events and does not
synthesize `MouseButton::Left`, so without this a phone could never enter or
restart. (Esc-to-menu stays keyboard-only; a tap is reserved for the common
retry.)

### HUD revealed on first touch (mobile vs PC)

The virtual pad (a faint left thrust panel, a steer ring, and a knob that
follows the finger) is hidden until the first touch is seen, then shown for the
rest of the session. A `TouchSeen` resource flips true on `any_just_pressed`, and
the HUD root (`TouchHud`) toggles its `Visibility` accordingly. So a PC/keyboard
session never shows the pad (no touch ever fires), while a phone reveals it the
instant a thumb lands. This is deliberately runtime touch-detection rather than
`#[cfg(target_arch = "wasm32")]` (which would show on desktop browsers too) or a
JS `navigator.maxTouchPoints` probe (wasm-only, false-positives on touch
laptops); reveal-on-first-touch is platform-agnostic and correct for hybrids.
The ring is a UI circle (`Node { border_radius: BorderRadius::MAX }` plus
`BorderColor`); note `BorderRadius` is a `Node` field in Bevy 0.19, not a
standalone component.

### Web viewport

`web/games/08_dropzone/index.html` already had a mobile viewport
(`width=device-width, initial-scale=1, user-scalable=no`). Added
`touch-action: none` to the canvas so touch drags feed the game instead of
scrolling / pinch-zooming the page.

## Testing / verification

- `touch_lean` (the pure offset -> lean mapping) has a unit test covering the
  dead zone, full-deflection sign per axis, clamping past the radius, and the
  `MAX_LEAN` ceiling on any offset. (`cargo test --example 08_dropzone`: 7
  passed.)
- `cargo fmt --check`, `cargo clippy --all-targets`, `scripts/check-ascii.sh`
  all clean.
- Ran the example (`DISPLAY=:0`): reaches the render loop, the touch HUD spawns
  (ring + knob present every frame), the touch systems run with empty input
  without panicking or query-conflicting, and the keyboard path still flies the
  ship to a clean landing (verified via the temporary `DROPZONE_SMOKE`
  autopilot, since removed). Web showcase rebuilt via `npm run build`.
- NOT verified headlessly: actual finger gestures (no touch-injection tool in
  this environment). The `touch_lean` math is unit-tested and the routing is
  reasoned; real thumb-on-glass feel (radius, dead zone, zone split) wants a
  pass on a phone or a browser touch-emulator and light tuning of the `*_PX`
  constants.
