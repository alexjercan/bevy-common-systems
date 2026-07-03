# Retro: 07_orbit polish - streak juice + hazard-hit impact feedback

- TASKS: 20260703-214926 (streak juice), 20260703-214927 (impact feedback)
- BRANCH: feature/07-orbit-polish (pre-created sprout worktree)
- REVIEW ROUNDS: 1 each (R1 REQUEST_CHANGES -> R2 APPROVE)

A `/flow` polish pass over Orbit Runner: add the "juice" layer `06_fruitninja`
has and `07_orbit` lacked. Two implementable tasks shipped; two proposals
(particle crate, real audio/music assets) were parked as tatr tasks for the
user because they need a dependency or licensing decision.

## What went well

- Checked the premise before building. The task as briefed was "add sounds like
  06_fruitninja," but a two-minute grep showed 07_orbit already wires SfxPlugin
  for every event -- the base sounds were done in the original cycle. Surfacing
  that up front turned the job into the higher-value work actually missing (the
  juice layer) instead of re-doing something already present. Reading the code
  beat trusting the brief.
- Ported known-good shapes rather than inventing. Streak/`Combo`, floating-text
  popups, and `CameraShake` were lifted almost verbatim from 06_fruitninja, so
  the feel matches the sibling example and the code stayed consistent. The one
  place that needed real thought (chase-camera shake) got it.
- The chase-camera shake ordering was reasoned from the source, not guessed.
  Reading `chase_camera_sync_transform_system` showed it sets `translation`
  absolutely every frame, which is exactly why an *additive* offset ordered
  `.after(ChaseCameraSystems::Sync)` cannot drift. Fruitninja's fixed-camera
  version set an absolute `CAMERA_BASE`; blindly copying that here would have
  fought the chase camera. The difference was caught by understanding, not by a
  failed play-test.
- Reused the existing `combo.wav` (user's explicit ask) and `PlaySfx::with_speed`
  for the rising-pitch pickup, so the audio polish needed zero new assets and
  zero new deps -- the placeholder-sound generator did not even need extending.
- Kept the pure logic unit-tested (streak scoring, pitch curve, trauma decay);
  the example test count went 11 -> 14, and the ECS/visual parts were left to
  the play-test, matching the repo's testing convention.

## What went wrong

- First build failed on `commands.play_sfx(PlaySfx::new(..))`: `play_sfx` takes a
  bare `Handle`, and a configured `PlaySfx` (volume + speed) must go through
  `commands.trigger(..)`. Root cause: reached for the convenience method by
  reflex without checking that the builder path is `trigger`. The audio module
  doc block spells this out ("Or trigger directly for per-shot volume / pitch
  control"); a 10-second look would have avoided the round-trip. Same class of
  mistake as the standing "verify the API against the source" gotcha, just in a
  crate-local module instead of Bevy.
- Self-review caught a real polish defect implementation missed: the "STREAK xN"
  banner was spawned fresh on every pickup at a fixed y, so a fast chain stacked
  overlapping banners. Fixed by keeping a single `StreakBanner` and replacing it
  in place. The failure mode was thinking about the single-pickup case, not the
  rapid-chain case that is exactly when the banner fires -- the popup analogue
  of a spawner edge case.

## What to improve next time

- When using a crate's ergonomic wrapper (`play_sfx`) vs its full builder
  (`PlaySfx` + `trigger`), check the module doc for which entry point takes the
  configured value before writing it. The wrappers deliberately take the simple
  type.
- For any transient UI popup, immediately ask "what happens when this fires
  several times inside its own lifetime?" -- fixed-position banners must be
  singletons or they overprint. Add it to the popup checklist next to the
  floating-"+N" pattern (those are fine because each spawns at a different
  position).
- Layering onto a plugin that owns a transform (chase camera): always confirm
  whether the plugin sets the value absolutely or incrementally before deciding
  additive vs absolute. Here reading the sync system was the whole ballgame.

## Action items

- [x] Both review findings that needed code/doc (R1.1 banner singleton, R1.1
  streak-break doc) fixed and re-verified.
- [ ] Play-test pass still owed (shake magnitude, streak window, banner
  centering via `Justify::Center` on a full-width node). Folds naturally into
  the existing tuning task 20260703-213510 style; not blocking.
- [ ] Two proposals parked for the user: 20260703-214928 (particle bursts --
  needs a particle crate or an in-crate helper) and 20260703-214929 (real audio
  assets + optional background music, the latter out of `SfxPlugin`'s SFX-only
  scope). Both need a dependency/licensing decision, so they wait on the user.
- Left as an observation, not an AGENTS.md edit yet: "configured `PlaySfx`
  (volume/speed) goes through `commands.trigger`, not `commands.play_sfx`" --
  crate-local, will promote to a gotcha only if it recurs.
