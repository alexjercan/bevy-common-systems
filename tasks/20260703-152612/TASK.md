# Add reusable audio SFX module to the crate

- STATUS: CLOSED
- PRIORITY: 100
- TAGS: feature,audio,historical

## Goal

Add a small, reusable, game-agnostic sound-effect module to the crate so any
game can fire a one-shot sound by handle without repeating Bevy's
`AudioPlayer` / `PlaybackSettings::DESPAWN` boilerplate. This is the building
block the fruit ninja example (task 20260703-152619) will consume.

## Steps

- [x] Create `src/audio/mod.rs` with a module-level `//!` doc comment and a
      short usage snippet (match the style of `src/health/mod.rs`).
- [x] Define a `PlaySfx` event (fired via `commands.trigger(...)`) carrying at
      least `handle: Handle<AudioSource>` and an optional per-shot
      `volume: f32` (default 1.0). Derive `Event` and `Reflect`. Verify the
      exact bevy 0.19 observer/event trait against how the crate already uses
      triggers (`src/modding/events.rs`, `src/health/mod.rs`) - global
      (non-entity) events use `On<PlaySfx>` in the observer.
- [x] Add an `SfxMasterVolume` resource (newtype `f32`, `Deref`/`DerefMut`,
      `Reflect`, default 1.0) that scales every played sound.
- [x] Add `SfxPlugin` (`Plugin`): `debug!("SfxPlugin: build")` in `build`,
      `init_resource::<SfxMasterVolume>()`, register the type, and
      `add_observer` an `On<PlaySfx>` handler that spawns
      `(AudioPlayer(handle), PlaybackSettings::DESPAWN.with_volume(Volume::Linear(per_shot * master)))`
      so the sound entity cleans itself up. Use `trace!` in the observer.
- [x] Add a `Commands::play_sfx(handle)` (and maybe `play_sfx_volume`)
      extension trait for the common fire-and-forget case, wrapping the trigger.
- [x] Define `pub mod prelude` in the module re-exporting `SfxPlugin`,
      `PlaySfx`, `SfxMasterVolume`, the `SfxCommandsExt` trait, and any
      `*Systems` set if one is added.
- [x] Register the module in `src/lib.rs`: add `pub mod audio;` and add
      `audio::prelude::*` to the crate `prelude` block.
- [x] Build and lint clean: `cargo build`, `cargo clippy --all-targets`,
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

## Outcome

Added `src/audio/mod.rs` (`SfxPlugin`, `PlaySfx` event, `SfxMasterVolume`
resource, `SfxCommandsExt` with `play_sfx` / `play_sfx_volume`) and wired it
into `src/lib.rs` (`pub mod audio;` + prelude). `PlaySfx` carries `handle`,
`volume` and `speed` with a builder (`new`/`with_volume`/`with_speed`); the
`speed` field was added beyond the plan so a consumer can pitch-shift a sound
(the fruit ninja combo chime will use it). The observer spawns
`(AudioPlayer, PlaybackSettings::DESPAWN.with_volume(Volume::Linear(v)).with_speed(s))`
so the audio entity self-despawns.

Decisions / deviations:
- Kept `PlaySfx` a plain global `Event` (mirrors `modding::GameEvent`), not an
  `EntityEvent` - SFX are not tied to a target entity. `commands.trigger` +
  `On<PlaySfx>` is the same pattern the crate already uses.
- Derived `Reflect` on both `PlaySfx` and `SfxMasterVolume`; only
  `SfxMasterVolume` is `register_type`d (it is the useful inspector/config
  surface). `Handle<AudioSource>` is `Reflect`, so the derive compiles clean.
- No new decoder feature here: the module only spawns players. The consumer
  (task 20260703-152619) owns choosing/enabling an audio format.

Verification: `cargo build`, `cargo clippy --all-targets`,
`cargo clippy --all-targets --features debug`, `cargo fmt --check`,
`cargo test` (20 unit + 12 doctests incl. the new audio doctest),
`./scripts/check-ascii.sh` all pass.

Self-reflection: verifying the exact bevy 0.19 audio API (`Volume::Linear`,
`with_volume`/`with_speed`, `DESPAWN`) against the vendored crate source before
writing paid off - no compile-fix churn. The only rework was rustfmt reflowing
the enlarged prelude import block, which one `cargo fmt` fixed. Next time run
`cargo fmt` (not just `--check`) as the first formatting step.
