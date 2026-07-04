# assets + scoring: migrate 07/08/10/11 onto SoundBank and HighScore

- STATUS: OPEN
- PRIORITY: 18
- TAGS: feature,audio,scoring,cleanup

> Follow-up from tatr 20260704-175422 (SoundBank), review NIT R1.1, and tatr
> 20260704-175423 (HighScore<T>), review NIT R1.2. Two harvest modules landed
> with 06/09 migrated; finish the other games in one pass.

## Goal (part 1: SoundBank)

`audio::SoundBank<K>` shipped and 06_fruitninja + 09_reactor moved onto it, but
07_orbit, 08_dropzone, 10_asteroids and 11_overload still hand-roll their own
`SfxAssets` handle-bag. Migrate the remaining four for consistency and to grow
the registry's payoff, mirroring the 06/09 refactor:

- Replace `struct SfxAssets { ...Handle fields... }` with a `#[derive(Clone,
  Copy, PartialEq, Eq, Hash, Debug)] enum Sfx { ... }`.
- Replace the inline `insert_resource(SfxAssets { field: assets.load("sounds/
  x.wav"), ... })` with `SoundBank::load(&assets, [(Sfx::Field, "x"), ...])`
  (base filename, no `sounds/` prefix or `.wav`).
- `Res<SfxAssets>` / `&SfxAssets` -> `Res<SoundBank<Sfx>>` / `&SoundBank<Sfx>`,
  and `sfx.field.clone()` -> `sfx.get(Sfx::Field)`.

Keep behaviour identical (same files, same volumes). Verify each `get()` key is
in the load list (else a runtime panic) and boot each game. Optional: demo the
opt-in `sounds_loaded` gate in one game with a `Loading` state.

## Goal (part 2: HighScore)

`scoring::HighScore<T>` shipped and 06/09 moved onto it; 07/10/11 (and any other
game with a best-score) still hand-roll a local `struct HighScore` + `NewBest`.
Migrate them, mirroring the 06/09 refactor:

- Drop the local `struct HighScore(T)` and `struct NewBest(bool)`; use
  `Res<HighScore<T>>` (the local `HighScore` currently shadows the prelude one,
  so removing it is safe).
- `record_high_score` -> `high.record(score)`; game-over screen -> `high.is_new_best()`
  and `high.best()`. Ensure `record_high_score` runs before the game-over screen
  in the `OnEnter(GameOver)` chain.
- Where a game persists its best (if any), use `PersistPlugin::<HighScore<T>>`.
