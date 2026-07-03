# Fruit ninja: live combo HUD indicator

- STATUS: OPEN
- PRIORITY: 65
- TAGS: feature,example

## Goal

Show the current combo while it is active so the player can see it climbing,
fading out when the combo resets.

## Steps

- [ ] Add a `ComboText` HUD element in `spawn_hud`, positioned distinctly from
      the score (e.g. top-center or below the score), initially empty.
- [ ] Add an `update_combo_text` system (Update, `Playing`) that reads the
      `Combo` resource: when `count >= 2` show "Combo xN" (optionally colored /
      scaled by the combo), and when `count < 2` clear it or fade it out.
- [ ] Optionally fade the text alpha with the remaining combo timer (from the
      time-window task) so it visibly "cools down" as the window runs out.
- [ ] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, real boot no panic.

## Notes

- `spawn_hud` (~:493) spawns the score `Text`; add the combo text there,
  `DespawnOnExit(Playing)`.
- Reads the `Combo` resource; if the timer-based combo (20260703-140243) is in,
  the fade can track `timer`; otherwise just show/hide on `count`.
- Soft dependency on 20260703-140243 for the timer-based fade, but the basic
  show/hide works against the current `Combo { count }`; note which.
- No new dependencies.
