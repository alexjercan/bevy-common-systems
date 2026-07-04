# assets + scoring + ui/input: migrate the games onto the Wave 2 harvests

- STATUS: OPEN
- PRIORITY: 18
- TAGS: feature,audio,scoring,ui,input,cleanup

> Follow-up from the dev-harness Wave 2 tasks: 20260704-175422 (SoundBank, NIT
> R1.1), 20260704-175423 (HighScore<T>, NIT R1.2), 20260704-175424 (ui/menu
> builders) and 20260704-175425 (leaf input/status/material helpers). Each
> harvest landed with two proof games migrated; this task finishes the rest in
> one pass so the crate's own modules are actually adopted everywhere.

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

## Goal (part 3: ui/menu builders)

`ui/menu` shipped (`centered_screen`, `screen_text`, `TitlePulse` + `MenuPlugin`)
with 06/07 migrated. Migrate the remaining games (08/09/10/11/12) off their local
`centered_screen`/`screen_text`/`pulse_menu_title`/`MenuTitle` onto the shared
module. 11_overload pulses its title differently (brightness, not alpha); leave
it or extend `TitlePulse` only if it maps cleanly.

## Goal (part 4: leaf helpers)

The leaf helpers shipped (`AnyStartPress`/`any_start_pressed`, `set_state_on_key`,
`status_bar_with_fps`, `glowing_material`) with 07/10 migrated. Migrate the rest:

- Replace each game's copy-pasted "advance on any press" check with
  `AnyStartPress` (adopting `UnifiedPointer` where the game does not already use
  it -- the spike's UnifiedPointer-adoption ask lives here now).
- Replace each local `giveup_on_escape`-style system with
  `set_state_on_key(KeyCode::Escape, GameState::GameOver)`.
- Replace the hand-rolled FPS `status_bar_item` block with `status_bar_with_fps()`.
- Replace emissive `StandardMaterial` literals with `glowing_material(base,
  emissive)` (spread it for extra fields).

Keep behaviour identical and boot each game.
