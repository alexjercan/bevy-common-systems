# Review: breach -- dedicated sound pass

- VERDICT: APPROVE
- ROUNDS: 1

## Verified

- `cargo clippy --all-targets` (clean), `cargo test --example 14_breach` (17 pass),
  `scripts/check-ascii.sh` (clean).
- Headless `BCS_AUTOPILOT`: full cycle, no panic, and NO asset-load errors for the new
  `breach_*.wav` files -- the autopilot fires the gun, so shoot/hit/kill all trigger and
  load cleanly (proof the files exist and are wired).
- Re-ran the generator: `git status` shows only the six new `breach_*.wav` added, the
  existing sounds byte-identical (the render is deterministic, noise seeded per name),
  so no spurious churn.

## Findings / checks

- The generator extension (`render_fx`) is additive and pure; the existing `render` and
  its 13 sines are untouched. Noise is seeded by file name -> reproducible.
- The `Sfx` table remap is a pure string change; `menu_select`/`game_over` intentionally
  stay shared (generic UI cues, the crate convention), and that reasoning is in a code
  comment + the README + the docs note.
- wasm coverage confirmed: `web/games/14_breach/index.html` copies the whole
  `assets/sounds` dir, so the new files ship without a web change.
- Docs updated: `assets/sounds/README.md` (breach section + required-files rows),
  `tasks/20260705-132200/NOTES.md`, and the AGENTS note.

## Nits (non-blocking)

- Timbre is a listen-test only (headless has no audio out); the cues are deliberately
  placeholder-quality, distinct by construction (noise vs sweep vs the other games'
  sines).
