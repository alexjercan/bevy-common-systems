# 11_overload: on-screen touch vent buttons + touch menu/result nav

- STATUS: CLOSED
- PRIORITY: 100
- TAGS: feature,example,mobile,historical

## Goal

Make `examples/11_overload.rs` fully playable on a phone in the wasm showcase,
mirroring the `08_dropzone` virtual-pad pattern. The game is currently
keyboard-only (1/2/3/4 vent the four gauges) and its menu/result screens only
accept mouse+keys, so a phone can neither start a run nor vent a gauge. Add an
on-screen touch vent pad and touch menu/result navigation as an ADDITIVE input
source; the keyboard path stays byte-for-byte unchanged.

## Steps

- [x] Read `tasks/20260704-103517/NOTES.md` and the touch code in
      `examples/08_dropzone.rs` (TouchControl/TouchSeen resources, HUD reveal,
      `update_touch_control`) to reuse the established shape.
- [x] Add a `TouchSeen` resource (bool) flipped true the first frame
      `Touches::any_just_pressed()` fires, in a system that runs in every state
      (mirror dropzone). Used to reveal the pad only on touch devices.
- [x] Spawn an on-screen vent pad: a bottom-anchored row of 4 UI buttons, one
      per gauge, each labelled like the gauge (`1 HEAT`, `2 PRES`, `3 FLUX`,
      `4 CHRG`) and colour-cued. Spawn once (like the status bar) with a
      `TouchPad` marker root; start `Visibility::Hidden`, toggle to `Visible`
      once `TouchSeen` is true. Use Bevy 0.19 UI idioms (FontSize::Px,
      TextLayout struct literal, `Node { border_radius: BorderRadius::MAX }`
      for rounded buttons, `BorderColor` component).
- [x] Add a `touch_vent_input` system (runs only in `GameState::Playing`,
      alongside `vent_input`, not replacing it) that detects a tap on a vent
      button and calls the SAME `apply_vent(&mut reactor.gauges, i)` path plus
      the same SFX as `vent_input`. Prefer Bevy UI `Interaction` on the buttons
      (native mouse+touch); if touch does not drive `Interaction` in this Bevy
      version, fall back to manual `Touches::iter_just_pressed()` rect-testing
      against each button's `ComputedNode`/`GlobalTransform`. One tap == one
      vent (just-pressed semantics, matching the key path).
- [x] Make menu start touch-navigable: in `menu_start` (~line 567) accept
      `Touches::any_just_pressed()` in addition to the existing mouse+key check
      so a tap starts a run. Do the same for `gameover_dismiss` / result input
      (~line 833). Keep Esc-to-menu keyboard-only.
- [x] If a tap that starts/retries the run would also immediately register as a
      vent on a button under the thumb, guard against it (e.g. pad hidden until
      Playing, or ignore the first frame after a state change). Verify the menu
      tap does not leak into an unintended vent.
- [x] Add `touch-action: none;` to the `#game-canvas` in
      `web/games/11_overload/index.html` (copy the dropzone CSS block) and
      confirm the viewport has `user-scalable=no`.
- [x] Add a unit test for any pure helper introduced (e.g. a
      `vent_button_at(point, rects) -> Option<usize>` hit-test), covering
      in-button hits per index and a miss outside all buttons. If the
      Interaction approach needs no such helper, note that no pure logic was
      added.
- [x] Verify by running headless with the autopilot technique: env-gated system
      that drives Menu -> Playing, exercises a vent, reaches GameOver, confirms
      no panic / query conflict and that the touch systems run. Remove the
      harness before commit. Run `cargo fmt --check`, `cargo clippy
      --all-targets`, `cargo test --example 11_overload`, `scripts/check-ascii.sh`.
- [x] Rebuild the web showcase via `npm run build` (run `npm ci` first in the
      fresh worktree's `web/`; node_modules is git-ignored) to confirm the wasm
      build still succeeds.
- [x] Write a short `tasks/20260704-130314/NOTES.md` documenting the
      scheme and decisions, and extend the `11_overload` entry in AGENTS.md if
      controls changed.

## Notes

- Relevant files: `examples/11_overload.rs` (setup ~391, gauge_item ~373,
  vent_input ~672, apply_vent ~665, menu_start ~561, gameover_dismiss ~827,
  GAUGES table ~96), `web/games/11_overload/index.html`,
  `examples/08_dropzone.rs` (reference), `tasks/20260704-103517/NOTES.md`.
- The whole game is a `status_bar` on a `Camera2d`; the vent pad is additional
  2D UI, spawned once in Startup and reused across states (like the status bar).
- Additive-writer rule: `vent_input` (keyboard) is untouched. Touch vents are a
  separate system feeding the same `apply_vent`. There is one gauge model, two
  input sources.
- Reveal-on-first-touch (not `#[cfg(wasm)]`) so a desktop-browser session never
  shows the pad; matches dropzone's rationale.
- Bevy 0.19 UI gotchas (AGENTS.md): BorderRadius is a `Node` field not a
  component; FontSize::Px(..); TextLayout struct literal. Copy idioms from an
  existing example, do not improvise the visual layer.

## Close-out

Done on branch `asteroids-overload-touch`. Implemented an on-screen touch vent
pad for `11_overload` mirroring the `08_dropzone` pattern:

- `vent_button_at(point, window)` pure hit-test (bottom `VENT_ZONE_H_FRAC` strip,
  `GAUGE_COUNT` equal columns), unit-tested (10 example tests pass).
- `spawn_vent_pad` renders four tinted, labelled buttons over the same fractions;
  hidden until first touch, revealed by `update_touch_pad` (which also hides the
  keyboard legend it replaces). `TouchSeen` resource gates the reveal.
- `touch_vent_input` reads just-pressed touches -> `apply_vent` + shared
  `trigger_vent_sfx`, additive to the untouched keyboard `vent_input`. Frame-
  derived just-pressed avoids the menu-tap held-finger leak.
- `menu_start` / `gameover_dismiss` now take `Touches::any_just_pressed()`.
- `web/games/11_overload/index.html`: `touch-action: none` on the canvas.

Chose the window-fraction hit-test over Bevy UI `Interaction` to avoid first-use
`ComputedNode`/DPI risk and to keep the mapping unit-testable; see
`tasks/20260704-130314/NOTES.md`.

Verified: `cargo fmt --check`, `cargo clippy --example 11_overload`,
`cargo test --example 11_overload` (10 passed), `scripts/check-ascii.sh` clean;
a temporary `OVERLOAD_SMOKE` autopilot drove Menu -> Playing -> GameOver headless
with no panic / query conflict (removed before commit); `npm run build` rebuilt
the whole web showcase (all 5 trunk builds + webpack succeeded).

Not verified: real finger gestures (no touch-injection tool); strip height /
button feel want a pass on a real phone.
