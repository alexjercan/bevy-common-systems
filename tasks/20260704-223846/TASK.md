# assets: migrate 07/08/10/11 onto SoundBank registry

- STATUS: OPEN
- PRIORITY: 18
- TAGS: feature,audio,cleanup

> Follow-up from tatr 20260704-175422 (SoundBank registry), review NIT R1.1.

## Goal

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
opt-in `sounds_loaded` gate in one game with a `Loading` state (review R1.2).
