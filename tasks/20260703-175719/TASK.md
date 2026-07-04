# docs: bump bevy_enhanced_input 0.25 -> 0.26 in AGENTS.md

- STATUS: CLOSED
- PRIORITY: 30
- TAGS: docs

## Goal

AGENTS.md ("Features and Dependencies") says `bevy_enhanced_input 0.25`, but
Cargo.toml pins `0.26.0`. Fix the version and re-scan the module map for any
other staleness while there.

## Steps

- [x] Update the `bevy_enhanced_input` version in AGENTS.md to 0.26 (matching
      Cargo.toml).
- [x] Quick pass for other stale version numbers in the same section.

## Notes

- Surfaced by the retro for task 20260703-173128 (touchscreen support).
