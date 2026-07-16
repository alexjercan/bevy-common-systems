# Retro: migrate the games onto the Wave 2 harvests

- TASK: 20260704-223846
- BRANCH: cleanup/migrate-wave2-harvests (squash-merged to master)
- REVIEW ROUNDS: 1 (APPROVE, five informational notes)

The catch-up migration after the four Wave 2 harvest tasks: adopt SoundBank,
HighScore, ui/menu and the leaf helpers across every game that the harvest
tasks left on their old local copies. Seven example files, four concerns, one
branch.

## What went well

- Built the coverage matrix first (one grep sweep counting each local pattern
  per game) instead of migrating blind. It turned a vague "migrate the games"
  into a concrete per-game todo and surfaced the surprises up front: 08 has no
  high score, 09/11 pulse brightness not alpha, 08 builds its menu inline rather
  than via helpers.
- Migrated per-game with a build after each, not per-concern across all files.
  Each file was opened once, all four concerns applied, then compiled -- errors
  stayed local to the game in hand instead of piling up across seven files.
- Read before replacing on every judgement call, which is where the value was.
  Mechanical sed handled the `Res<SfxAssets>`/`.clone()` rewrites, but the wins
  came from *not* being mechanical: leaving 09/11's "any key" menus (AnyStartPress
  would have narrowed them), leaving 08's deliberately `unlit` streak material
  (glowing_material would have dropped `unlit`), and spotting that 09's
  `manual_controls` Space is a game action, not a menu advance.
- Guarded the one real regression myself before review: `SoundBank::get()` panics
  on an unloaded key. Statically diffed each game's used keys against its load
  list (a five-line shell loop) -- all clean, so the panic path is provably
  unreachable. Same discipline the original SoundBank task used.
- Let the title-pulse shape decide TitlePulse adoption. 10/12 pulse alpha (maps
  to TitlePulse exactly), 09/11 pulse RGB brightness (does not), so only the
  first two adopted it and the other two kept their local pulse. Reading the
  actual pulse math, not assuming, avoided a visual regression the ui/menu retro
  had explicitly flagged.

## What went wrong

- Nothing broke, but a `sed` that rewrote `Res<SfxAssets>` missed the by-reference
  helper signatures (`sfx: &SfxAssets` in 11's `trigger_vent_sfx` and 12's), the
  same gap the original SoundBank cycle hit. Caught it by adding an explicit
  `s/sfx: &SfxAssets/.../` arm and a `grep -c` for leftovers after each sed, so it
  cost nothing this time -- but it is a recurring blind spot: a Res-only find/replace
  never covers the `&T` helper params.

## What to improve next time

- When mechanically rewriting a resource type, grep for BOTH `Res<T>` and `&T`
  (and `ResMut<T>`) before writing the sed, not after. The by-ref helper params
  are invisible to a `Res<...>` pattern and have now been missed twice.

## Action items

- [x] All seven games migrated onto SoundBank / HighScore / ui/menu / leaf
  helpers as applicable; full suite + autopilot + boot + screenshot verified.
- The dev-harness Wave 2 (175422-175425) and its migration follow-up (this task)
  are now fully complete -- every harvested module is adopted across the games,
  not just in its two proof examples.
