# Fruit ninja: menu title pulse

- STATUS: OPEN
- PRIORITY: 55
- TAGS: feature,example

## Goal

Give the main menu a little life: gently pulse the "FRUIT NINJA" title so the
menu is not a static screen.

## Steps

- [ ] Add a `MenuTitle` marker to the title text spawned in `spawn_menu`.
- [ ] Add a `pulse_menu_title` system (Update, `Menu`) that oscillates the
      title's `TextColor` alpha (and/or color between two golds) over time using
      a sine of elapsed seconds, for a soft breathing effect.
- [ ] Keep it `Menu`-only via `run_if(in_state(GameState::Menu))`.
- [ ] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, real boot (sit on the menu; confirm
      the pulse and no panic).

## Notes

- `spawn_menu` at :451 spawns the title via `screen_text("FRUIT NINJA", 72.0,
  ...)`. `screen_text` returns a bundle; add the marker alongside it (spawn the
  title as its own child/entity with the marker, or extend `screen_text` to
  accept an extra marker).
- Bevy UI text size is fixed per `TextFont`; animate color/alpha rather than
  font size for the pulse (scaling UI text is awkward).
- Smallest task; purely cosmetic, menu-only.
- No new dependencies.
