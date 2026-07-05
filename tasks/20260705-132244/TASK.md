# breach -- dedicated placeholder sound pass

- STATUS: OPEN
- PRIORITY: 45
- TAGS: spike,breach,example,audio

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

- Spike: docs/spikes/20260705-132024-breach-fun-pass.md
- Follow the established placeholder-audio pattern: extend
  `scripts/gen-placeholder-sounds.py`, regenerate into `assets/sounds/`, update
  `assets/sounds/README.md` (see `docs/2026-07-03-audio-and-fruitninja-sounds.md`).
- Keep them clearly placeholder-quality; the point is distinct, readable audio
  cues, not final sound design.
- Coordinate with the pickups task if that adds a pickup SFX -- avoid duplicating.
- Verify: `cargo clippy --all-targets`, then run the example and confirm each cue
  fires (audio can only be confirmed by running, not by build).
