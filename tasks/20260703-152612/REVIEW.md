# Review: Add reusable audio SFX module to the crate

- TASK: 20260703-152612
- BRANCH: feature/ninja-sounds

## Round 1

- VERDICT: APPROVE

Diff reviewed: `src/audio/mod.rs` (new) + `src/lib.rs` prelude wiring
(`git diff master...feature/ninja-sounds -- src/`). Check suite run on this
exact commit: `cargo fmt --check`, `cargo clippy --all-targets`,
`cargo clippy --all-targets --features debug`, `cargo test` (20 unit + 12
doctests, including the new `src/audio/mod.rs` doctest), `./scripts/check-ascii.sh`
all pass.

Assessment against AGENTS.md conventions and the bevy 0.19 audio API:
- One concern per module (transient SFX only), `*Plugin` naming (`SfxPlugin`),
  module `prelude` aggregated into `crate::prelude`, `Reflect` on the resource,
  module-level `//!` doc with a compiling usage snippet, `debug!` in `build` /
  `trace!` in the observer, plain ASCII. All satisfied.
- Global `Event` + `commands.trigger` + `On<PlaySfx>` observer mirrors the
  existing `modding::GameEvent` pattern - the right precedent for a
  non-entity-scoped event.
- Audio API used correctly: `PlaybackSettings::DESPAWN.with_volume(Volume::Linear(..)).with_speed(..)`
  matches the vendored bevy_audio 0.19 source; `DESPAWN` makes the sound entity
  self-cleaning, so polyphony (one entity per trigger) and cleanup are both
  correct. Volume is clamped with `.max(0.0)`.

No BLOCKER/MAJOR/MINOR findings.

- [ ] R1.1 (NIT) src/audio/mod.rs:135 - `speed` is passed through unclamped; a
  zero or negative `speed` would reach rodio. Not a defect (it is caller-driven
  and the module documents 1.0 as normal), but a `.max(0.0)` or a doc note that
  speed must be positive would harden it. Take it or leave it.
