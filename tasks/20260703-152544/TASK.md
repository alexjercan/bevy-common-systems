# Document audio decision and required sound assets

- STATUS: CLOSED
- PRIORITY: 80
- TAGS: docs,audio

## Goal

Document the audio work so future sessions and the user understand the design
and know exactly which sound assets to supply. Update the crate orientation
docs to reflect the new module and the example's new behavior.

## Steps

- [x] Add a decision note under `docs/` (e.g. `docs/audio.md`) covering: why a
      reusable `SfxPlugin` was added instead of inlining `AudioPlayer` in the
      example; the fire-and-forget `PlaySfx` + `SfxMasterVolume` design and its
      deliberate limits (one-shot SFX only, no music/mixer); the mock-asset
      decision (WAV placeholders + bevy `wav` dev-feature, vorbis still default
      so `.ogg` also works); and how the user swaps in real assets (overwrite
      files at the fixed `assets/sounds/*` paths, no code change).
- [x] Update `AGENTS.md` Module Map to add the `audio` module
      (`SfxPlugin` + `PlaySfx` + `SfxMasterVolume`), and update the
      `06_fruitninja` example bullet to mention that it now has sound.
- [x] Update the `//!` header doc of `examples/06_fruitninja.rs` so the
      "no assets" claim is corrected (it now loads sound assets).
- [x] Cross-check the `assets/sounds/README.md` from task 20260703-152619 is
      accurate and complete (every wired event has a listed file); reconcile if
      the wiring diverged from the plan.
- [x] Run `./scripts/check-ascii.sh` and `cargo fmt --check` so the doc/text
      edits keep CI green.

## Notes

- Depends on: 20260703-152619 (docs describe the final wiring and asset paths).
- The AGENTS global rule (docs/ folder holds decisions; plain ASCII only)
  applies. Keep the note concise; the retro (`/compound`) is separate.
- This task is docs-only; no functional code changes beyond the example header
  doc comment.

## Outcome

Added `tasks/20260703-152544/NOTES.md` (kept the repo's
date-prefixed doc-file convention instead of the plan's literal `docs/audio.md`
name) covering the reusable-module rationale, the `PlaySfx`/`SfxMasterVolume`
design and its scope limits, the WAV + dev-only-decoder mock decision, and the
asset-swap instructions. Updated `AGENTS.md`: added the `audio` module to the
Module Map and extended the `06_fruitninja` example bullet to mention sound and
point at the placeholders/docs. Corrected the example's `//!` header, which
claimed "no assets" - it now describes the SFX and points at
`assets/sounds/README.md`. Cross-checked the README against the wiring: all
eight wired events (menu_select, slice, splat, combo, golden, bomb, game_over,
launch) are listed and accurate.

Verification: `./scripts/check-ascii.sh`, `cargo fmt --check`, and
`cargo build --example 06_fruitninja` pass. Docs-only task; no behavior change.

Self-reflection: nothing tricky. Following the existing dated doc-file naming
rather than the plan's placeholder name kept `docs/` consistent.
