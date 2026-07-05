# Review: breach -- navigable main menu + options (persisted sensitivity)

- VERDICT: APPROVE
- ROUNDS: 1

## Verified

- `cargo fmt --check`, `cargo clippy --all-targets` (clean), `cargo test --example
  14_breach` (22 pass, incl. 3 new headless menu-logic tests), `scripts/check-ascii.sh`.
- Headless `BCS_AUTOPILOT`: full cycle, no panic (no regression).
- Real windowed run + xdotool grab (actually SEEN, not just booted):
  - Main menu: BREACH title, subtitle, best score, PLAY + OPTIONS buttons, hint.
  - Options: title, "Look sensitivity", [-] x0.8 [+] stepper, BACK, hint.
  - Persistence end-to-end: the stepper wrote 14_breach.sensitivity.json = 0.8, and a
    FRESH launch loaded it (Options showed x0.8) -- persisted across launches, confirmed
    on disk AND on screen.

## Findings

- MAJOR (found + fixed here, latent on master): the Menu state had NO camera. The only
  camera is the Playing Camera3d, a child of the player (despawns with it), so Bevy UI
  had nothing to render to and the menu was invisible. Never caught because the autopilot
  force-transitions Menu->Playing and headless framebuffer captures come back black. Fix:
  spawn_menu spawns a Camera2d (DespawnOnExit(Menu)). Confirmed by an xdotool window grab.
- Menu navigation/start is a game-driven state change the autopilot can't prove, so it is
  covered by headless App tests driving the real menu_buttons/menu_keys with
  Interaction::Pressed: PLAY -> Playing, OPTIONS/BACK navigation, +/- step + clamp, and
  PLAY inert while the options panel is up (the screen gate, not visibility). clamp_sens
  is unit-tested. (Applies the breach lose-condition retro lesson.)
- Sensitivity is a multiplier on DoomController.look_sensitivity, which the controller
  uses for BOTH mouse and touch look -- one setting covers both, as asked.
- Panels toggle via Display::None (not Visibility), so the hidden panel takes no layout
  space; handling is also gated on MenuScreen.

## Discovered, filed as follow-up (not widening this branch)

- The game-over screen has the SAME no-camera bug. Filed as tasks/20260705-154058.

## Nits

- Sensitivity range/step (0.25..3.0, 0.1) and button styling are taste; easily tuned.
