# Fruit ninja: golden bonus fruit

- STATUS: OPEN
- PRIORITY: 72
- TAGS: feature,example

## Goal

Add a rare golden fruit worth a bonus (+5) with a distinct gold look and popup,
and have it extend the combo time window so it helps you keep a combo going.

## Steps

- [ ] Add a `gold_material` (bright gold, a little emissive/metallic) to
      `FruitAssets`, built in `setup`.
- [ ] Add a `Golden` marker component. In `spawn_projectile`, with a small
      chance (independent of the bomb roll, e.g. ~8%) spawn a golden fruit: the
      gold material, the `Golden` marker, optionally a slightly larger scale.
      Ensure a projectile is not both bomb and golden.
- [ ] In `slice_objects`, when a sliced fruit is `Golden`, award a flat bonus
      (`GOLDEN_POINTS = 5`) in addition to / instead of the combo point, and
      spawn a distinct gold "+5" popup (bigger, gold color).
- [ ] Extend the combo window: on slicing a golden fruit, refresh the combo
      timer by a larger amount (`COMBO_WINDOW_GOLDEN`, e.g. 2.5s) so golden
      fruit buys extra combo time.
- [ ] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, real boot (auto-slice a golden fruit;
      confirm +5 popup and extended timer, no panic).

## Notes

- Depends on: 20260703-140243 (time-window combos) for the combo-timer to
  extend. Priority orders it after.
- `FruitAssets` at :252 ({ mesh, materials, bomb_material }); `spawn_projectile`
  at :543 rolls `is_bomb`; `slice_objects` fruit branch handles scoring and
  `spawn_floating_text`.
- Decide golden vs combo interaction: simplest is golden = flat +5 that does NOT
  advance the escalating combo count but DOES refresh (extend) the timer; or +5
  plus a combo step. Pick one and note it.
- `on_fragments_spawned` already reuses the sliced shell's material, so a golden
  fruit bursts gold automatically.
- No new dependencies.
