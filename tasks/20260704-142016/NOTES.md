# 09_reactor mobile touch controls

Date: 2026-07-04
Task: `tasks/20260704-142016`

## What this adds

`examples/09_reactor.rs` is now playable on a phone in the wasm showcase. The
change is deliberately small because most of the reactor was already
touch-ready:

- **In-game controls need no new input code.** Manual Tap, Sell and the six
  shop cards are Bevy UI `Button` + `Interaction` widgets. `Interaction` on
  those widgets is driven by `bevy_ui`'s own `ui_focus_system`
  (`bevy_ui-0.19.0/src/focus.rs`), which reads `Res<Touches>` DIRECTLY
  (`touches_input.any_just_pressed()` for the press edge, line 187, and
  `touches_input.first_pressed_position()` for the hit-test point, line 208) and
  is registered unconditionally by the UI plugin (`lib.rs:175`), independent of
  the `bevy_picking` feature. So a tap drives `Interaction::Pressed` on those
  buttons exactly like a mouse click. The reactor already relied on `Interaction`
  for the desktop mouse path, so touch is a second input source into the same
  path -- no new input code. (bevy_picking also ships a touch backend that
  produces `PointerId::Touch` pointers, but the `Interaction` component here is
  the `bevy_ui` focus path, not the picking path.)
- **Menu / game-over navigation now accepts touch.** `menu_start` and
  `gameover_dismiss` read `Touches::any_just_pressed()` alongside the existing
  `MouseButton::Left` + any-key. winit-on-web delivers a tap as a `Touch` event
  and does NOT synthesize `MouseButton::Left`, so without this a phone could
  neither start a run (tap the menu) nor return from the meltdown screen. This
  is the same touch-nav fix `08_dropzone` and `11_overload` made. Esc-to-give-up
  stays keyboard-only; a tap is reserved for the common start/dismiss.
- **Web canvas.** `web/games/09_reactor/index.html` gains `touch-action: none`
  on `#game-canvas` so touch taps/drags feed the game (tap-to-start, the shop
  and tap/sell buttons) instead of scrolling / pinch-zooming the page. The
  viewport already had `user-scalable=no`.

- **Responsive HUD so every control fits a phone frame.** This was the real
  work. The in-game buttons only help if they are ON SCREEN: taps have no
  keyboard fallback, so a control below the fold is unreachable (the desktop
  `1..6` digit-buy keys buy a shop part whether or not its card is visible;
  touch has no such escape hatch). The original HUD was sized for a wide desktop
  window -- 34px readouts, a 520px-wide heat bar, `240px` shop cards, and a
  TAP/SELL row with long labels. The web showcase embeds every game in a FIXED
  4:5 portrait frame (`web/src/style.css` `.game-embed`, `aspect-ratio: 4/5`,
  max 560px), so on a phone the logical viewport is roughly 360x450, and at that
  width the readouts and TAP/SELL row clipped off the sides and FOUR of the six
  shop cards fell below the bottom edge -- unbuyable on touch. The fix makes the
  HUD adapt:
  - Shop cards are a PERCENTAGE of the shop width (`48%`), so the six always
    form a fixed 2-column grid that scales with the frame -- every card on
    screen, no scroll container needed (Bevy 0.19 has no built-in scroll-input
    system anyway, and a drag-to-scroll gesture would fight the tap-to-buy that
    fires on press). A fixed pixel width does NOT work: flexbox wraps before it
    shrinks, so a narrow phone drops the cards to a single column that overflows
    the bottom -- exactly the bug this fixes. The percentage keeps two columns at
    any width.
  - Readouts (23px, so they stay one line even on a ~320px frame), the heat bar
    (`width: 96%`, `max_width: 520px`), the heat label and TAP/SELL (compact
    one-line labels) all shrink / go fluid to fit the narrow frame.
  - The root gains extra top padding so the centred readouts clear the telemetry
    status-bar overlay (top-right corner), which otherwise overlapped them on a
    narrow frame.
  The desktop INPUT behaviour is unchanged; this is a visual/layout pass that
  also reads fine at the wider embed size. Verified by screenshotting the
  running example across the device range (see Testing).

## Why no virtual pad (unlike dropzone / overload)

`08_dropzone` added a virtual steer stick and `11_overload` added a bottom vent
pad because their gameplay input was analog steering / keyboard-only number keys
with no on-screen widgets to tap. The reactor is different: its controls are
already always-visible on-screen `Button`s that the desktop mouse uses too, so
there is nothing to reveal-on-first-touch and no window-fraction hit-test to
build. The reveal-on-first-touch / `TouchSeen` machinery those two examples
carry would be dead weight here. The touch retrofit is purely (a) the
tap-anywhere menu/game-over navigation and (b) the web canvas gesture handling.

The keyboard + mouse desktop behaviour is byte-for-byte unchanged: `touches`
is an additional OR-ed condition, so when no touch fires the two input functions
evaluate exactly as before.

## Testing / verification

- `cargo fmt --check`, `cargo clippy --example 09_reactor`,
  `cargo test --example 09_reactor` (9 passed), `scripts/check-ascii.sh` all
  clean.
- Ran the example (`DISPLAY=:0`, under `timeout`): it reaches the render loop
  (`bevy_render::view::window` swap-chain configuration line) with no panic.
- **Layout verified visually by screenshot**, not just reasoned. A temporary
  env-gated harness (`REACTOR_SHOT="WxH"`, since removed) forced the window to a
  fixed phone-portrait size and auto-started into Playing; `xdotool` +
  ImageMagick `import` captured and cropped the window. Confirmed all six shop
  cards (and their cost lines), the readouts, and the TAP/SELL row are on screen
  and clear of the telemetry overlay across the current-device range: 344x430
  (iPhone SE 2020/2022, the small-phone floor -- 375 CSS px minus the showcase's
  ~32px page padding gives a ~343px 4:5 embed), 360x450, and 560x700 (the
  showcase max embed). This caught the real regression: before the layout pass,
  only two of six cards were reachable at phone width.
- Device floor: at frames narrower than ~340px (e.g. a first-gen iPhone SE, 320
  CSS px -> ~288px embed) the bottom shop row can still clip -- there is simply
  not enough room for six descriptive cards plus the header in a 288x360 box, and
  the game iframe is a fixed 4:5 viewport so the page scroll does not reveal more
  of it. Every phone from the 2020 iPhone SE onward is covered.
- Touch->`Interaction` mechanism verified against `bevy_ui-0.19.0/src/focus.rs`
  (`ui_focus_system` reads `Touches` directly, registered unconditionally), not
  assumed.
- Wasm target compiles: `cargo build --target wasm32-unknown-unknown --example
  09_reactor` (via `nix develop -c`) finished clean, so the showcase build is
  intact (the change is example Rust + a one-line canvas CSS rule, both
  wasm-safe).
- NOT verified: real finger gestures (no touch-injection tool in this
  environment -- `xdotool` injects mouse, not touch events). The menu/game-over
  touch nav reuses the pattern proven in dropzone/overload, and the in-game
  buttons ride `bevy_ui`'s own focus hit-testing, so there is no bespoke
  hit-test math to get wrong here. The screenshots confirm every control is
  on-screen and sized reasonably; a final pass on a real phone / browser touch-
  emulator is still worthwhile to confirm thumb feel.
