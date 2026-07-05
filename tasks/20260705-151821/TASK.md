# breach -- navigable main menu + options (persisted look sensitivity)

- STATUS: CLOSED
- PRIORITY: 60
- TAGS: feature,breach,example,ui


## Goal

Turn `14_breach`'s static title screen into a navigable main menu and add an
Options submenu with a persisted, adjustable look sensitivity.

- Main menu: interactive PLAY and OPTIONS buttons (Bevy UI `Button` + `Interaction`,
  so mouse + touch both work), plus keyboard (Enter = play). Keeps the pulsing title,
  tagline and best-score readout.
- Options menu: a look-sensitivity control (- / + stepper buttons) with a live
  readout, and a BACK button (Escape also backs out). Sensitivity is a multiplier over
  the base `LOOK_SENS`, clamped to a sane range.
- Persist the sensitivity across launches via `PersistPlugin` (like `HighScore`), and
  apply it to the `DoomController.look_sensitivity` at player spawn so BOTH mouse and
  touch look (both go through the controller's look_sensitivity) respect it.

Done = you can click/tap PLAY to start, open OPTIONS, change sensitivity with visible
feedback, relaunch and see it stuck, and feel the difference in-game.

## Notes

- `ui/menu` only gives `centered_screen`/`screen_text`/`TitlePulse`; the buttons +
  panel switching are game-local (the `Button`+`Changed<Interaction>` idiom from
  `09_reactor`). Menu panels toggle via a `MenuScreen { Main, Options }` resource
  driving Visibility (avoid respawn churn).
- Keep the death observer / gameplay untouched; this is menu-state only.
- Copy Bevy 0.19 UI idioms from an existing example (font_size/TextLayout/border_radius);
  do not improvise the visual layer.
- Pure logic (sensitivity clamp + step) gets `#[cfg(test)]` tests.
- Verify: `cargo fmt`, `cargo clippy --all-targets`, `cargo test --example 14_breach`,
  ascii, a headless `BCS_AUTOPILOT` run (the autopilot presses a key -> menu_start must
  still start the game, so keep a keyboard start path), and a real run. Update the `//!`
  header and the AGENTS note.

## Steps

- [x] **Persisted sensitivity resource.** `LookSensitivity(f32)` multiplier resource
  (`Resource + Serialize + Deserialize + Default` = 1.0); `PersistPlugin::<LookSensitivity>`
  under `"14_breach.sensitivity"`. Pure `clamp_sens(mult)` (range e.g. 0.25..3.0) and a
  step size const. Apply at player spawn: `look_sensitivity: LOOK_SENS * sens.0`.
- [x] **Menu panels + switching.** `MenuScreen { Main, Options }` resource (reset to Main
  on enter Menu). `spawn_menu` builds a Main panel (title/tagline/best + PLAY, OPTIONS
  buttons) and an Options panel (SENSITIVITY readout + `-`/`+` + BACK), both
  `DespawnOnExit(Menu)`, tagged `MainPanel`/`OptionsPanel`. `update_menu_visibility`
  shows the active panel.
- [x] **Button + key handling.** `menu_buttons` reads `Changed<Interaction>` on
  `PlayButton`/`OptionsButton`/`BackButton`/`SensDownButton`/`SensUpButton`: Play ->
  NextState(Playing) (+select sfx); Options/Back switch `MenuScreen`; Sens -/+ step and
  clamp `LookSensitivity` (persist auto-saves). Keep keyboard: Enter/click starts from
  Main, Escape backs out of Options. `update_sens_readout` shows "SENSITIVITY x1.0".
- [x] **Tests + verify.** Unit-test `clamp_sens` (in-range passthrough, both bounds) and
  the step helper. Run the full check suite + headless autopilot (confirm it still
  reaches Playing via a key) + a real run. Update `//!` header + AGENTS note.
