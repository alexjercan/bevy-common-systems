# Fruit ninja: bombs, health and lose condition

- STATUS: OPEN
- PRIORITY: 80
- TAGS: feature,example

## Goal

Add bombs to `examples/06_fruitninja.rs` and wire the crate's health system so
that slicing a bomb ends the run. Bombs are visually distinct shapes that arc
up mixed with fruit; slicing one is an instant loss. A HUD element shows the
player's health, driven by the crate `HealthPlugin`.

Design (confirmed with user): slicing a bomb is an INSTANT game over (lethal
damage), and bombs are the ONLY thing that affects health. Missed fruit is
free. Health is therefore effectively a single life that only a bomb ends;
it is still modeled through the real `Health` component so the example
demonstrates the crate's health system end to end.

## Steps

- [ ] Add `HealthPlugin` to the app. On `OnEnter(GameState::Playing)`, spawn a
      `Player` entity carrying `Health::new(1.0)` (state-scoped to `Playing`).
- [ ] Add a `Bomb` component and a distinct look: reuse the octahedron mesh but
      a dark material (near-black / dark red), or build a small cube via
      `TriangleMeshBuilder`; keep it centered on the origin so it can still be
      sliced/exploded. Give bombs a slightly different scale or color so they
      are unmistakable.
- [ ] In `spawn_fruit` (or a shared spawner), spawn a bomb instead of a fruit
      with some probability (e.g. ~20%). Bombs arc and tumble like fruit and
      are despawned when missed (no penalty).
- [ ] In `slice_fruit`, detect slicing a `Bomb` the same way as a fruit
      (segment vs radius). On a bomb hit: trigger
      `HealthApplyDamage { entity: player, source: None, amount: 1.0 }`
      (lethal) instead of scoring. Still explode the bomb mesh for feedback.
- [ ] Observe `On<Add, HealthZeroMarker>` (on the player): transition
      `GameState` to `GameOver`. This is the real lose trigger; keep or remove
      the temporary `Escape` stand-in from the menu task (keep as "give up").
- [ ] HUD: show the player's health next to the score (e.g. a heart icon or
      "HP: 1"), updated from the `Health` component. State-scoped to `Playing`.
- [ ] Update AGENTS.md example description for `06_fruitninja` to mention
      bombs / health / menu.
- [ ] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, and a real boot: slice fruit to
      score, slice a bomb to trigger game over. Add a headless test if a pure
      helper emerges (e.g. bomb-vs-fruit spawn selection).

## Notes

- Depends on: 20260703-121418 (game states -- needs `GameState::GameOver`) and
  20260703-121414 (HUD to extend). Priority orders it last.
- Health API (`src/health/mod.rs`): `Health::new(max)`; trigger damage with
  `commands.trigger(HealthApplyDamage { entity, source, amount })`;
  `HealthZeroMarker` is auto-inserted at zero health -- observe
  `On<Add, HealthZeroMarker>` to react. `HealthApplyDamage` auto-propagates up
  the hierarchy, so the player entity should be the damage target directly.
- The bomb must still be a centered mesh with `Mesh3d` +
  `MeshMaterial3d<StandardMaterial>` for `ExplodeMeshPlugin` to slice it (the
  explode observer requires both).
- `on_fragments_spawned` currently reuses the sliced entity's material for its
  fragments, so a bomb will burst in its dark color automatically -- no change
  needed there.
- Assumption: a single `Player` entity per run holds Health; the damage target
  in `slice_fruit` is looked up via a `Single<Entity, With<Player>>` or a
  stored resource.
- No new dependencies.
