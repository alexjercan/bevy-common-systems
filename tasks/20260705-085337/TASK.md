# Bastion: on-screen build + upgrade buttons with keybind labels

- STATUS: CLOSED
- PRIORITY: 80
- TAGS: feature,example,bastion,historical

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

- [x] Read `src/ui/touchpad.rs` for the exact `button_grid_at` signature and
      `Rect` convention, and re-read `11_overload.rs:674-837` (`spawn_vent_pad`,
      `vent_button_at`, `touch_vent_input`) as the reference.
- [x] Define the button zone as a shared constant (window-fraction `Rect`) and a
      helper that returns which button a normalized point hits: N tower buttons
      then 1 upgrade button. Unit-test this pure hit-test (left edge -> slot 0,
      right edge -> upgrade slot, above the strip -> None).
- [x] Spawn the strip in `spawn_hud` (or a new `spawn_build_bar` chained after
      it): iterate `catalog.towers` for the tower buttons (label
      `"{digit}  {name}\n{cost}c"`) plus a trailing Upgrade button (`"U\nUpgrade"`).
      Tag it `DespawnOnExit(GameState::Playing)`.
- [x] Add a `build_bar_input` system (run in Playing, after `orbit_camera` like
      the other tap consumers) that, on `drag.released_tap`, maps `drag.tap_pos`
      to a button: a tower button sets `build.spec = Some(i)` (and clears
      `selection`), the Upgrade button runs the same upgrade action as pressing
      U. It must CONSUME the tap so `place_or_select` does not also fire - order
      it before `place_or_select` and clear `drag.released_tap` (make DragState's
      field writable) or have `place_or_select` early-return when the tap is in
      the bar zone via the shared helper. Pick one mechanism and make it robust.
- [x] Refactor the upgrade action out of `upgrade_selected` into a shared helper
      (`try_upgrade_selected(...)` taking the needed params) so both the U key
      and the Upgrade button call it (no duplicated cost/credit logic - the retro
      warns against inline formulas).
- [x] Give the buttons feedback: tint the armed tower's button and the Upgrade
      button by affordability/selection each frame (a small
      `update_build_bar` system reading `Build`/`Selection`/`Credits`/`Catalog`).
      Grey out unaffordable buttons.
- [x] Make sure the strip does not overlap the top-left HUD text; place it along
      the bottom. Verify on a phone-shaped viewport with `ScreenshotPlugin`
      (`BCS_SHOT=390x844 ... --features debug`) that all buttons fit on one row
      at narrow width (use percentage/flex-grow widths, not fixed px + wrap -
      the reactor mobile retro lesson).
- [x] Update the module `//!` controls paragraph, the menu hint and the HUD
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

## Close-out

Done. Added an always-visible on-screen build bar to the Playing HUD: a bottom
strip of equal-width buttons, one per catalogued tower (keybind digit + name +
cost) plus a trailing Upgrade button (keybind U), spawned by `spawn_build_bar`
(chained after `spawn_hud`). Buttons are hand-rolled `Node`s copied from
`11_overload`'s vent-pad idiom (`border_radius` in `Node`, `BorderColor::all`,
`TextFont { font_size: FontSize::Px }`), tinted each frame by `update_build_bar`
(armed/selected glow, unaffordable dim).

Tap routing is region-owns-tap, not Bevy `Interaction` (composes with
`UnifiedPointer`): `build_bar_hit` (pure, over `button_grid_at` from
`ui/touchpad`) maps a `tap_pos` to a `BuildButtonKind`; `build_bar_input` acts on
bar taps (arm a tower / upgrade), and `place_or_select` early-returns for taps in
the shared `build_bar_zone`, so a button tap never also hits the world. The
`BUILD_BAR_H_FRAC` fraction is shared between the strip height and the hit-test
zone. The upgrade logic was extracted into `try_upgrade_selected`, called by both
the U key and the Upgrade button (no duplicated cost/credit logic).

Kept the standalone requirement: the bar is always visible (plain `Visibility`,
no `RevealOnTouch`/`TouchpadPlugin` gating), so desktop gets the TD palette and
mobile gets tappable controls from the same code. Mouse click and touch both
reach it via `UnifiedPointer`. Docs (module controls, reuse list, menu hint, HUD
action line) updated.

Verification:
- Pure hit-test unit-tested (`build_bar_hit_maps_columns_and_misses`,
  `build_bar_zone_matches_strip_height`).
- OBSERVABLE-effect integration test (`build_bar_tap_arms_tower_and_upgrades`)
  drives the real `build_bar_input` system through a minimal App with a sized
  Window: a left-column tap arms tower slot 0, and an Upgrade tap on a selected
  tower actually raises its level (1->2) and damage. This covers the pointer-tap
  path the autopilot (keyboard-only) cannot, per the follow-up retro's
  "verify the advertised control's effect, not a proxy" lesson.
- `BCS_SHOT=390x844` phone-width screenshot confirms all four buttons render on a
  single row with readable keybind + label + cost, tinted per tower (reactor
  mobile lesson: flex-grow columns, not fixed px + wrap).
- Checks: plain `cargo build --example` clean (no dead-code, per the packs-task
  lesson), `cargo clippy --all-targets` clean, `cargo fmt --check` clean,
  `./scripts/check-ascii.sh` clean, `cargo test --examples` all green.
