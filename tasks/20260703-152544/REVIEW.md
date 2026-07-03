# Review: Document audio decision and required sound assets

- TASK: 20260703-152544
- BRANCH: feature/ninja-sounds

## Round 1

- VERDICT: REQUEST_CHANGES

Reviewed the docs commit: `docs/2026-07-03-audio-and-fruitninja-sounds.md`
(new), `AGENTS.md` (module map + example bullet), `examples/06_fruitninja.rs`
`//!` header. Ran `./scripts/check-ascii.sh` and `cargo fmt --check`: clean.

The content is accurate and complete except for one broken internal reference,
which matters because accuracy is this task's entire purpose.

- [ ] R1.1 (MINOR) AGENTS.md:230 - the `06_fruitninja` bullet says "see
  `assets/sounds/README.md` and `docs/audio.md`", but the decision note was
  actually committed as `docs/2026-07-03-audio-and-fruitninja-sounds.md` (the
  repo's dated doc-file convention). `docs/audio.md` does not exist, so the
  link is dead. Fix the reference to the real filename.
  - Response: fixed - AGENTS.md now points at
    `docs/2026-07-03-audio-and-fruitninja-sounds.md`. Grepped the tree: no
    other live reference to `docs/audio.md` remains (only the plan step's
    "e.g." and this finding).

## Round 2

- VERDICT: APPROVE

Verified R1.1: AGENTS.md:234 now references
`docs/2026-07-03-audio-and-fruitninja-sounds.md`, which exists on disk. No
remaining live reference to the non-existent `docs/audio.md` (grep confirmed).
`./scripts/check-ascii.sh` and `cargo fmt --check` clean. Docs are accurate and
consistent with the wiring; approved.

- [x] R1.1 (MINOR) AGENTS.md:230 - broken `docs/audio.md` reference.
  - Confirmed fixed.
