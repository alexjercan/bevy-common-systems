# Fruit ninja: wire sound effects with mock assets

- STATUS: CLOSED
- PRIORITY: 90
- TAGS: feature,example,audio

## Goal

Make `examples/06_fruitninja.rs` play a sound on each key gameplay event using
the crate's `SfxPlugin` (task 20260703-152612), driven by committed
placeholder audio files so the example runs and is audible today. Real assets
are dropped in later by overwriting the placeholder files at fixed paths, with
no code change.

## Steps

- [x] Decide and record the mock-asset format: use short WAV placeholders and
      enable bevy's `wav` decoder feature for examples only, by adding
      `bevy = { version = "0.19", features = ["wav"] }` to
      `[dev-dependencies]` in `Cargo.toml` (cargo unifies this with the main
      `bevy` dep for example/test builds; the library default features are
      unchanged). Rationale: WAV is trivially and deterministically
      generatable as a valid placeholder; vorbis stays default-on so the user
      may instead supply `.ogg` by changing only the path constants.
- [x] Create `assets/sounds/` and commit one tiny valid placeholder WAV at
      each real sound path (see the path list in Notes). A short (~0.1s) quiet
      sine or silence is fine; identical bytes copied to each path is
      acceptable. Generate with a committed helper script under `scripts/`
      (e.g. `scripts/gen-placeholder-wav.sh` or a small python/printf writer)
      so regenerating is reproducible, OR write the WAV bytes directly. Do NOT
      rely on ffmpeg/sox being installed unless confirmed available.
- [x] Add a `SfxAssets` resource to the example holding the loaded
      `Handle<AudioSource>` for each event, loaded in `setup` (or a dedicated
      startup system) via `asset_server.load("sounds/<name>.wav")`. Mirror the
      existing `FruitAssets` resource pattern.
- [x] Add `SfxPlugin` to the app in `main` (from `bevy_common_systems::prelude`).
- [x] Fire the right sound at each event via `commands.trigger(PlaySfx{..})`
      or `commands.play_sfx(handle)`:
      - menu start click -> `menu_click` (or `OnEnter(GameState::Playing)`).
      - fruit slice -> `slice_objects` fruit branch (the swipe hit), a whoosh.
      - fruit burst -> `on_fragments_spawned` (or when `ExplodeMesh` is
        inserted in `resolve_slice_pop`), a squish/pop.
      - golden fruit slice -> `slice_objects` golden branch, a bright sparkle.
      - combo milestone -> when `combo.count >= 2` in `slice_objects`, a rising
        chime (optionally pitch/volume up with combo count via the per-shot
        volume).
      - bomb slice -> `slice_objects` bomb branch, an explosion.
      - game over -> `on_player_died` or `OnEnter(GameState::GameOver)`.
- [x] Keep it tasteful: guard against retriggering the same one-shot many
      times per frame where relevant; do not add a sound to `move_projectiles`
      per-frame. Stretch (optional, only if cheap): launch whoosh in
      `spawn_projectile`, combo-end tally in `tick_combo`.
- [x] Add an `assets/sounds/README.md` listing every required file: path,
      which event it plays on, and a one-line description of the intended
      character and rough length, so the user can source and drop in real
      assets without touching code.
- [x] Verify it runs: `cargo run --example 06_fruitninja` (in `nix develop` on
      NixOS) boots with no missing-asset errors and plays the placeholder on
      events. Also `cargo run --example 06_fruitninja --features debug`.
- [x] Keep CI green: `cargo build`, `cargo clippy --all-targets`,
      `cargo clippy --all-targets --features debug`, `cargo fmt --check`,
      `cargo test`, `./scripts/check-ascii.sh`.

## Notes

- Depends on: 20260703-152612 (the `SfxPlugin` module must exist first).
- Relevant code in examples/06_fruitninja.rs: `setup` (asset load site, ~L463),
  `main` plugin registration (~L126), `menu_click` (~L786), `slice_objects`
  (fruit/golden/bomb/combo branches, ~L1029-1173), `resolve_slice_pop` (~L370),
  `on_fragments_spawned` (~L1248), `on_player_died` (~L574), `tick_combo`
  (~L283), `spawn_projectile` (~L933).
- Proposed asset paths (final names the user will overwrite with real audio):
  - `assets/sounds/menu_select.wav`  -- menu click to start
  - `assets/sounds/slice.wav`        -- fruit slice whoosh
  - `assets/sounds/splat.wav`        -- fruit burst into fragments
  - `assets/sounds/combo.wav`        -- combo milestone (>= 2)
  - `assets/sounds/golden.wav`       -- golden fruit slice sparkle
  - `assets/sounds/bomb.wav`         -- bomb explosion
  - `assets/sounds/game_over.wav`    -- run ends
  - (stretch) `assets/sounds/launch.wav` -- fruit launch whoosh
- The AssetServer loads relative to the `assets/` dir at the crate root, so
  paths passed to `load` omit the `assets/` prefix.
- Confirm `.gitignore` does not exclude `assets/` (there is a short
  `.gitignore` at the root; adjust if needed so the placeholders are committed).

## Outcome

Wired `SfxPlugin` into `examples/06_fruitninja.rs` and added an `SfxAssets`
resource (one `Handle<AudioSource>` per event) loaded in `setup`. Sounds fire
on: menu start (`menu_click`), plain-fruit slice (`slice_objects` else branch),
golden slice (golden branch), combo x2+ (pitched up with the chain via
`PlaySfx::with_speed`), bomb slice (bomb branch), fruit burst
(`resolve_slice_pop` at the pop->explode moment, fruit-only so bombs never get
a splat), the launch whoosh (`spawn_projectile`, quiet), and the game-over
sting (`play_game_over_sfx` on `OnEnter(GameOver)`, so it covers both bomb death
and Escape give-up).

Mock assets: `scripts/gen-placeholder-sounds.py` (stdlib `wave`/`struct` only,
no ffmpeg/sox) writes eight short quiet sine blips to `assets/sounds/*.wav`,
one distinct pitch each so events are distinguishable by ear. `assets/` is not
gitignored, so the placeholders are committed. `assets/sounds/README.md` lists
every required file, its event, character and length for the user to source
real audio (drop in at the same path, no code change).

Format decision: added `bevy = { version = "0.19.0", features = ["wav"] }` to
`[dev-dependencies]` so the WAV decoder (hound) is enabled for examples/tests
only; the library keeps bevy default features. Vorbis stays default-on, so a
user may instead supply `.ogg` by changing the extensions in the `SfxAssets`
load calls (noted in the README).

Difficulties: none of note. The fruit-burst sound placement was the one real
design choice - hanging it off `resolve_slice_pop` (fruit-only) instead of the
generic `on_fragments_spawned` observer avoids doubling a splat onto bomb
fragments, which also explode into `ExplodeFragments`.

Verification: `cargo build --example 06_fruitninja`, `cargo clippy
--all-targets`, `--features debug`, `cargo fmt --check`, `cargo test` (20 unit +
12 doctests), `./scripts/check-ascii.sh` all pass. A 25s `cargo run --example
06_fruitninja` booted and ran to the timeout with no asset-load errors or
panics (only a harmless X11 XSETTINGS warning), confirming the placeholders
load through bevy's asset server. Interactive per-event audition needs the
user's `nix develop` graphical session.

Self-reflection: generating valid WAVs from the Python stdlib was the right
call over depending on ffmpeg/sox or hand-writing header bytes - reproducible
and zero external deps. Enabling the decoder as a dev-dependency feature keeps
the library surface clean, which matters for a "copy-pastable" crate. The
header `//!` "no assets" claim is now stale; correcting it is task
20260703-152544 (docs).
