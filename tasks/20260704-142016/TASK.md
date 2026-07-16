# 09_reactor: mobile touch controls (menu/game-over tap nav + web canvas touch-action)

- STATUS: CLOSED
- PRIORITY: 40
- TAGS: feature,reactor,mobile

## Goal

Make `examples/09_reactor.rs` playable on a phone in the wasm showcase. The
in-game controls (Manual Tap, Sell, and the six shop cards) are Bevy UI
`Button` + `Interaction` widgets, and bevy_picking 0.19 ships a touch backend
enabled by default (`touch_pick_events`, `is_touch_enabled: true`), so taps
already drive `Interaction::Pressed` on those buttons -- verified against
`bevy_picking-0.19.0/src/input.rs`. The gaps are the tap-anywhere Menu and
Game-Over screens (they read `MouseButton::Left` + keys only, and winit-on-web
delivers taps as `Touch` events with no synthesized mouse) and the web canvas,
which does not yet swallow touch gestures.

This mirrors the additive-writer, touch-nav approach proven in
`08_dropzone` (`tasks/20260704-103517/NOTES.md`) and reused in
`11_overload` (`tasks/20260704-130314/NOTES.md`). Desktop keyboard
+ mouse behaviour stays byte-for-byte unchanged.

## Steps

- [x] `menu_start`: add `touches: Res<Touches>` and OR-in
  `touches.any_just_pressed()` alongside the existing mouse/key start.
- [x] `gameover_dismiss`: same -- OR-in `touches.any_just_pressed()` so a tap
  returns to the menu. (Esc-to-give-up stays keyboard-only.)
- [x] Confirm the in-game Tap/Sell/Shop buttons need no extra work (Bevy
  picking touch backend drives `Interaction`); document why in the design note.
- [x] `web/games/09_reactor/index.html`: add `touch-action: none` to
  `#game-canvas` so touch drags feed the game instead of scrolling / pinch-
  zooming the page (the viewport already has `user-scalable=no`).
- [x] Verify: `cargo fmt --check`, `cargo clippy --example 09_reactor`,
  `cargo test --example 09_reactor`, `scripts/check-ascii.sh`, and boot the
  example under `DISPLAY=:0` to confirm it reaches the render loop.
- [x] Write `tasks/20260704-142016/NOTES.md` (what/why, per the
  dropzone/overload docs).

## Notes

- Reactor's controls are always-visible on-screen buttons used by mouse too,
  so there is NO virtual-pad / reveal-on-first-touch HUD to add (unlike
  dropzone's steer stick or overload's vent pad). The touch retrofit is purely
  the menu/game-over navigation plus the web canvas gesture handling.

## Review round 1 (REQUEST_CHANGES -> addressed)

- MAJOR: the in-game buttons "work via touch" only if they are ON SCREEN. The
  HUD was sized for a wide desktop window; in the showcase's fixed 4:5 portrait
  frame (~360x450 on a phone) the readouts/TAP/SELL clipped and four of six shop
  cards fell below the fold -- unbuyable on touch (no keyboard digit fallback on
  a phone). Fixed with a responsive HUD: shop cards wrap into a 2-col (phone) /
  3-col (desktop) grid, readouts/heat-bar/buttons shrink/go fluid, and the root
  clears the telemetry overlay. Verified by SCREENSHOTTING the running example
  at 360x450 and 560x700 (temporary `REACTOR_SHOT` harness + xdotool + import,
  harness removed before commit).
- MINOR: corrected the doc/commit -- taps drive `Interaction` via `bevy_ui`'s
  `ui_focus_system` (reads `Touches` directly), not the `bevy_picking` backend.
- NIT (accepted): give-up (Esc) stays keyboard-only, matching 11_overload; a
  phone reaches game-over via meltdown and taps to dismiss.

## Review round 2: APPROVE

Independent re-review confirmed the MAJOR is fixed (2-column grid is
arithmetically stable under Bevy 0.19 BorderBox sizing -- 96% + 6px gap holds
two columns down to ~150px containers, no horizontal overflow under the 520px
cap), no new regressions (TAP/SELL stay thumb-sized, desktop untouched), the
screenshot harness is fully removed (0 grep hits), and the doc's
bevy_ui-focus-system attribution is accurate. Two cosmetic nits only (10px card
desc; a doc line-ref off by one, since corrected).
