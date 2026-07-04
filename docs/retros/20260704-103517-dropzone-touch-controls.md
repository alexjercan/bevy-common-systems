# Retro: 08_dropzone mobile virtual-pad touch controls

- TASK: 20260704-103517
- BRANCH: feature/08-dropzone-touch (squash-merged to master as 254ab5a)
- REVIEW ROUNDS: 2 (R1 REQUEST_CHANGES -> R2 APPROVE)

See `tasks/20260704-103517/TASK.md` close-out and
`docs/2026-07-04-dropzone-touch-controls.md` for what was built. This is about
how the working went. From the fun+mobile spike (`tasks/20260704-102022`, Part 2).

## What went well

- Verified the non-obvious Bevy 0.19 input API against source before writing:
  `Touches` methods (`iter_just_pressed`, `get_pressed`, `start_position`,
  `any_just_pressed`) all matched, so `update_touch_control` compiled first try.
- Additive-writer design kept the keyboard byte-for-byte unchanged: touch
  distils into a `TouchControl` resource that `read_input` merges, so there is
  one steering model with two sources and zero risk to the existing path. The
  independent reviewer confirmed the keyboard is untouched when no touch fires.
- Reused the smoke-autopilot technique (now an AGENTS.md gotcha) to fly the whole
  Menu -> Playing -> Result cycle headlessly and confirm the touch systems and
  HUD run without panicking or query-conflicting, since there is no
  touch-injection tool in this environment.
- Answered the user's mid-flow "mobile vs PC?" question with reveal-on-first-touch
  rather than platform sniffing - platform-agnostic, no JS probe, correct for
  touch-laptops and hybrids.

## What went wrong

- Missed that the menu and result screens were not touch-navigable (R1.1, MAJOR):
  `menu_input` took keys + `MouseButton::Left` and `result_input` took only keys,
  but winit-on-web delivers taps as `Touch` events with no synthesized mouse, so
  a phone could never start or retry. The flight controls were thorough but the
  run could not be entered. Root cause: I implemented the task's enumerated steps
  (thrust / lean / routing / HUD) and none of them named menu/result navigation,
  so I did not either - even though the GOAL ("playable on a phone in the wasm
  showcase") plainly implied the whole journey. The review caught it.
- First touch-routing design latched one pointer id per zone with
  just_pressed/just_released bookkeeping (R1.2, MAJOR); it dropped input when a
  second finger was in a zone or a finger was held across a run restart. Root
  cause: I modelled the common single-finger case with event latching instead of
  deriving state from the live pressed-touch set each frame. The frame-derived
  rewrite (`thrust = any pressed touch started left of split`, steer = keep-or-
  adopt a right-zone touch) is both simpler and correct.
- Assumed `BorderRadius` was a component; it is a `Node` field in Bevy 0.19
  (`Node { border_radius: BorderRadius::MAX }`). Caught by a compile error, not
  up front - the same "verify Bevy UI API against source" class as the recurring
  FontSize/AmbientLight/TextLayout gotcha.

## What to improve next time

- For a "usable/playable on X" goal, walk the ENTIRE user journey on X (enter,
  act, exit/retry), not just the headline interaction. The enumerated steps can
  omit connective tissue (here: menu/result navigation) that the goal requires.
- Prefer stateless / frame-derived input state over event latching when the
  source already tracks live state (`Touches`). Latching invites lost-event edge
  cases (multi-touch, state resets); deriving from the current set each frame is
  simpler and avoids them.
- When spawning Bevy 0.19 UI, check whether a visual property is a component or a
  `Node` field before writing it (BorderRadius is a field; BorderColor is a
  component). Same lesson as the existing UI-idioms gotcha.

## Action items

- [x] Extended the AGENTS.md Bevy-0.19-UI gotcha with `BorderRadius` being a
  `Node` field (not a component), alongside FontSize/AmbientLight/TextLayout.
- [ ] Follow-up not filed: touch-feel constants (`STEER_RADIUS_PX`,
  `STEER_DEAD_PX`, `THRUST_ZONE_FRAC`) are reasoned, not thumbed - they want a
  pass on a real phone / browser touch-emulator, like the earlier flight-tuning
  cycle did for the physics constants.
