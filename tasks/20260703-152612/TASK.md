# Add reusable audio SFX module to the crate

- STATUS: OPEN
- PRIORITY: 100
- TAGS: feature,audio

## Goal

Add a small, reusable, game-agnostic sound-effect module to the crate so any
game can fire a one-shot sound by handle without repeating Bevy's
`AudioPlayer` / `PlaybackSettings::DESPAWN` boilerplate. This is the building
block the fruit ninja example (task 20260703-152619) will consume.

## Steps

- [ ] Create `src/audio/mod.rs` with a module-level `//!` doc comment and a
      short usage snippet (match the style of `src/health/mod.rs`).
- [ ] Define a `PlaySfx` event (fired via `commands.trigger(...)`) carrying at
      least `handle: Handle<AudioSource>` and an optional per-shot
      `volume: f32` (default 1.0). Derive `Event` and `Reflect`. Verify the
      exact bevy 0.19 observer/event trait against how the crate already uses
      triggers (`src/modding/events.rs`, `src/health/mod.rs`) - global
      (non-entity) events use `On<PlaySfx>` in the observer.
- [ ] Add an `SfxMasterVolume` resource (newtype `f32`, `Deref`/`DerefMut`,
      `Reflect`, default 1.0) that scales every played sound.
- [ ] Add `SfxPlugin` (`Plugin`): `debug!("SfxPlugin: build")` in `build`,
      `init_resource::<SfxMasterVolume>()`, register the type, and
      `add_observer` an `On<PlaySfx>` handler that spawns
      `(AudioPlayer(handle), PlaybackSettings::DESPAWN.with_volume(Volume::Linear(per_shot * master)))`
      so the sound entity cleans itself up. Use `trace!` in the observer.
- [ ] Add a `Commands::play_sfx(handle)` (and maybe `play_sfx_volume`)
      extension trait for the common fire-and-forget case, wrapping the trigger.
- [ ] Define `pub mod prelude` in the module re-exporting `SfxPlugin`,
      `PlaySfx`, `SfxMasterVolume`, the `SfxCommandsExt` trait, and any
      `*Systems` set if one is added.
- [ ] Register the module in `src/lib.rs`: add `pub mod audio;` and add
      `audio::prelude::*` to the crate `prelude` block.
- [ ] Build and lint clean: `cargo build`, `cargo clippy --all-targets`,
      `cargo clippy --all-targets --features debug`, `cargo fmt --check`,
      `cargo test`, `./scripts/check-ascii.sh`.

## Notes

- Relevant files: src/lib.rs (module list + prelude), src/health/mod.rs
  (plugin/observer/prelude template), src/helpers/despawn.rs (tiny
  observer-driven plugin template), src/modding/events.rs (trigger usage).
- Bevy audio one-shot today is literally
  `commands.spawn((AudioPlayer(h), PlaybackSettings::DESPAWN))`; the module's
  value is the by-handle trigger + master volume + prelude ergonomics, kept to
  one concern (fire-and-forget SFX). Do not build a full music/mixer system.
- Bevy 0.19 volume API: `PlaybackSettings::with_volume(Volume::Linear(x))`
  (verify exact path against the bevy 0.19 in Cargo.lock). No `wav`/decoder
  feature is needed here - this module only spawns players; decoding features
  are the consumer's concern (handled in task 20260703-152619).
- No unit tests are strictly required (behavior is ECS-side, exercised by the
  example), but add one for any pure helper if you introduce one.
