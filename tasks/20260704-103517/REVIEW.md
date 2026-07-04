# Review: 08_dropzone mobile virtual-pad touch controls

- TASK: 20260704-103517
- BRANCH: feature/08-dropzone-touch

## Round 1

- VERDICT: REQUEST_CHANGES

An independent skeptical pass plus self-review. The flight controls are correct
(zone routing, floating-origin slide, `touch_lean` sign/clamp, the keyboard+touch
merge, query disjointness, coordinate space, and reveal-on-first-touch all
verified clean), but two real gaps block the "playable on a phone" goal.

- [x] R1.1 (MAJOR) examples/08_dropzone.rs:menu_input/result_input - state
  navigation is keyboard/mouse only, so a phone cannot start or restart a run.
  `result_input` reads only `KeyCode` (Space/Esc); `menu_input` reads keys +
  `MouseButton::Left`, and winit-on-web (bevy 0.19) delivers taps as `Touch`
  events without synthesizing `MouseButton::Left`. The flight controls are done
  but the run cannot be entered or retried by touch. Fix: also accept
  `Touches::any_just_pressed()` in `menu_input` and `result_input` (a tap starts
  / retries).
- [x] R1.2 (MAJOR) examples/08_dropzone.rs:update_touch_control - the
  single-id-per-zone latch drops input in real cases: a finger held across a run
  restart (`start_run` resets `TouchControl`) is never re-adopted until lifted,
  and if two fingers land in a zone, releasing the first cuts thrust/steer while
  the second is still down. Root cause: state is latched to one id and only
  re-adopted on `iter_just_pressed`. Fix: derive state from currently-pressed
  touches by their `start_position()` each frame -- `thrust = any pressed touch
  started left of split`; steer = keep the tracked touch if still pressed and
  right-zone, else adopt another right-zone pressed touch (re-centring the
  origin). Removes the latch for both the multi-finger and restart cases.
- [x] R1.3 (MINOR) examples/08_dropzone.rs:touch_lean test - covers dead zone,
  full-deflection sign, past-radius clamp and the axis ceiling, but not the
  linear ramp. Add a mid-range assertion (offset halfway between dead and radius
  yields ~half `MAX_LEAN`) so a broken `mag` formula is caught.
- [x] R1.4 (MINOR) examples/08_dropzone.rs:read_input - while any steer finger is
  down, touch fully preempts keyboard lean (in the dead zone that means a resting
  thumb suppresses A/D/W/S). Acceptable priority, but document it (touch does not
  just add to lean, it takes over while steering).
- [x] R1.5 (MINOR) examples/08_dropzone.rs:touch_lean - touch clamps the combined
  deflection vector to `MAX_LEAN`, while keyboard W+A reaches `MAX_LEAN` on each
  axis (~1.41x diagonal). The stick cannot reach the keyboard's diagonal extreme.
  Intentional-looking (a stick capped at max tilt), but note the asymmetry.
- [x] R1.6 (NIT) examples/08_dropzone.rs:495 - `update_touch_hud` is unordered vs
  `update_touch_control` (only `update_touch_control.before(read_input)` is
  declared), so the HUD may render one frame stale. Add `.after(update_touch_control)`
  to make the intent explicit.

## Round 2

- VERDICT: APPROVE

Verified every Round 1 finding against the new diff (commit dc31798):

- R1.1 RESOLVED - `menu_input` and `result_input` accept
  `Touches::any_just_pressed()`; a phone can start and retry.
- R1.2 RESOLVED - `update_touch_control` now derives thrust/steer from the
  currently-pressed touches by starting zone each frame (no id latch); the
  `thrust_id` field is gone. Fixes the second-finger and held-across-restart
  cutout. Re-verified via the autopilot smoke run (738 frames, clean landing, no
  panic).
- R1.3 RESOLVED - `touch_lean` test now asserts the mid-range linear ramp
  (half deflection -> half MAX_LEAN).
- R1.4 RESOLVED - `read_input` documents that touch fully preempts keyboard lean
  while steering (dead zone included).
- R1.5 RESOLVED - `touch_lean` documents the diagonal vector-clamp asymmetry vs
  the keyboard.
- R1.6 RESOLVED - `update_touch_hud.after(update_touch_control)`.

Checks re-run clean: fmt, clippy --all-targets, 7 unit tests, check-ascii,
example boots + full cycle flown, web showcase rebuilds. Approving.
