# Bastion: on-screen build + upgrade buttons with keybind labels

- STATUS: OPEN
- PRIORITY: 80
- TAGS: feature,example,bastion

> Part of the 12_bastion polish goal. Building a tower is currently keyboard-only
> (number keys arm a tower, U upgrades), which is unusable on a touchscreen and
> not the norm for a TD game. The user wants on-screen buttons for buildings and
> upgrades, visible in the STANDALONE game too (not gated behind first-touch),
> each labelled with its keybind, so a phone tap and a desktop click both work.

## Goal

Add an always-visible on-screen button strip to `examples/12_bastion.rs`
Playing HUD: one button per catalogued tower (name + cost + its number keybind)
and an Upgrade button (labelled `U`). Tapping/clicking a tower button arms that
build slot; tapping Upgrade upgrades the selected tower. The buttons drive the
exact same code paths as the existing keys, and a tap on a button must NOT also
be treated as a world/ground tap.

## Design notes (from exploration)

- The crate has NO generic `button()` builder. Hand-roll a `Node` strip like
  overload's `spawn_vent_pad` (`11_overload.rs:674-735`): an Absolute bottom (or
  bottom-left) strip, `FlexDirection::Row`, each button a child `Node` with
  `BackgroundColor`/`BorderColor`/`border_radius` holding `Text` children for the
  keybind digit and the label (use `screen_text` for labels).
- Keep the strip ALWAYS visible (plain `Visibility::default()`,
  `DespawnOnExit(GameState::Playing)`), NOT `RevealOnTouch` - the user wants it
  in the standalone game. (Do not add `TouchpadPlugin` gating.)
- Route taps by region, not Bevy `Interaction`, to compose with `UnifiedPointer`
  (exploration: mixing `Interaction` + `UnifiedPointer` double-counts a press).
  Reuse the pure `button_grid_at` from `src/ui/touchpad.rs` (or an equivalent
  local rect test) against the strip's window-fraction `Rect` to map a
  `tap_pos` to a button index, sharing ONE zone constant between the visual
  layout and the hit-test (overload's key invariant).

## Steps

- [ ] Read `src/ui/touchpad.rs` for the exact `button_grid_at` signature and
      `Rect` convention, and re-read `11_overload.rs:674-837` (`spawn_vent_pad`,
      `vent_button_at`, `touch_vent_input`) as the reference.
- [ ] Define the button zone as a shared constant (window-fraction `Rect`) and a
      helper that returns which button a normalized point hits: N tower buttons
      then 1 upgrade button. Unit-test this pure hit-test (left edge -> slot 0,
      right edge -> upgrade slot, above the strip -> None).
- [ ] Spawn the strip in `spawn_hud` (or a new `spawn_build_bar` chained after
      it): iterate `catalog.towers` for the tower buttons (label
      `"{digit}  {name}\n{cost}c"`) plus a trailing Upgrade button (`"U\nUpgrade"`).
      Tag it `DespawnOnExit(GameState::Playing)`.
- [ ] Add a `build_bar_input` system (run in Playing, after `orbit_camera` like
      the other tap consumers) that, on `drag.released_tap`, maps `drag.tap_pos`
      to a button: a tower button sets `build.spec = Some(i)` (and clears
      `selection`), the Upgrade button runs the same upgrade action as pressing
      U. It must CONSUME the tap so `place_or_select` does not also fire - order
      it before `place_or_select` and clear `drag.released_tap` (make DragState's
      field writable) or have `place_or_select` early-return when the tap is in
      the bar zone via the shared helper. Pick one mechanism and make it robust.
- [ ] Refactor the upgrade action out of `upgrade_selected` into a shared helper
      (`try_upgrade_selected(...)` taking the needed params) so both the U key
      and the Upgrade button call it (no duplicated cost/credit logic - the retro
      warns against inline formulas).
- [ ] Give the buttons feedback: tint the armed tower's button and the Upgrade
      button by affordability/selection each frame (a small
      `update_build_bar` system reading `Build`/`Selection`/`Credits`/`Catalog`).
      Grey out unaffordable buttons.
- [ ] Make sure the strip does not overlap the top-left HUD text; place it along
      the bottom. Verify on a phone-shaped viewport with `ScreenshotPlugin`
      (`BCS_SHOT=390x844 ... --features debug`) that all buttons fit on one row
      at narrow width (use percentage/flex-grow widths, not fixed px + wrap -
      the reactor mobile retro lesson).
- [ ] Update the module `//!` controls paragraph, the menu hint and the HUD
      action-line to mention the buttons.

## Verification

- `cargo clippy --all-targets` clean; `cargo test --examples` runs the hit-test
  unit test; `cargo fmt --check`; `./scripts/check-ascii.sh`.
- `BCS_SHOT=390x844 cargo run --example 12_bastion --features debug` under
  `timeout` writes a screenshot; `Read` it and confirm the button strip renders
  with all buttons visible on one row and readable labels.
- Verify the OBSERVABLE effect of a button tap (per retro: not a proxy). Use the
  AutopilotPlugin input closure to inject a synthetic press at a button's screen
  coordinate (write `UnifiedPointer`/`Touches` or the DragState path) and confirm
  `build.spec` / a tower placement / an upgrade actually happens - or, if
  injecting pointer coords is impractical headlessly, screenshot before/after a
  scripted tap. Document how it was verified in the close-out.
- If `$DISPLAY` is set, also boot interactively under `timeout` to confirm it
  reaches the render loop.
