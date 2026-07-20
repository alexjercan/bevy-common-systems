# breach -- dedicated placeholder sound pass

- STATUS: CLOSED
- PRIORITY: 45
- TAGS: spike,breach,example,audio,historical

## Goal

Replace `14_breach`'s borrowed placeholder SFX with dedicated breach sounds.
Today all seven `Sfx` variants map to SHARED files
(`Sfx::Shoot`->launch, `Sfx::Hit`->pickup, `Sfx::EnemyDown`->combo,
`Sfx::Hurt`->hurt, `Sfx::Wave`->level_up, `Sfx::Select`->menu_select,
`Sfx::GameOver`->game_over), so a gunshot sounds like a fruit launch and a kill
like a combo chime. Generate breach-appropriate placeholders (gunshot, enemy hit,
enemy death, player hurt, wave, menu, game over -- plus a pickup SFX for the
pickups task) and remap the `Sfx` table to them.

## Notes

- Spike: tasks/20260705-132024/SPIKE.md
- Follow the established placeholder-audio pattern: extend
  `scripts/gen-placeholder-sounds.py`, regenerate into `assets/sounds/`, update
  `assets/sounds/README.md` (see `tasks/20260703-152544/NOTES.md`).
- Keep them clearly placeholder-quality; the point is distinct, readable audio
  cues, not final sound design.
- Coordinate with the pickups task if that adds a pickup SFX -- avoid duplicating.
- Verify: `cargo clippy --all-targets`, then run the example and confirm each cue
  fires (audio can only be confirmed by running, not by build).

## Steps

- [x] **Extend the gen script.** Add a `render_fx(kind, f0, f1, dur, amp, decay, seed)`
  supporting noise bursts and pitch sweeps with a punchy decay envelope (seeded, so
  deterministic), and a `BREACH_SOUNDS` table: `breach_shoot` (noise pop), `breach_hit`
  (short downward tick), `breach_kill` (noise burst), `breach_hurt` (low groan),
  `breach_wave` (rising alert), `breach_pickup` (bright rise). Write them alongside the
  existing sines.
- [x] **Generate + commit the WAVs.** Run the script; commit the six
  `assets/sounds/breach_*.wav`.
- [x] **Remap breach's Sfx table.** Point Shoot/Hit/EnemyDown/Hurt/Wave/Pickup at the
  new `breach_*` files; keep Select->menu_select and GameOver->game_over shared (generic
  UI cues, the crate's established reuse pattern -- note this).
- [x] **Docs + verify.** Update `assets/sounds/README.md` (breach section + required
  files) and note in `docs/`. `cargo clippy --all-targets`, `cargo test --example
  14_breach`, ascii, and a real run to confirm each cue fires (audio can only be
  confirmed by running). Update the AGENTS note.
