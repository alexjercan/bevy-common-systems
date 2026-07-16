# Retro: fruit ninja sound effects (audio SFX module)

- TASKS: 20260703-152612 (reusable `audio` SFX module), 20260703-152619 (wire
  sounds into 06_fruitninja with mock assets), 20260703-152544 (docs). All
  CLOSED and APPROVED, committed on feature/ninja-sounds.

## Context

The 06_fruitninja game was silent. Goal: add sound on every gameplay event,
implement against a committed placeholder so it runs today, and hand the user a
list of the real assets to source. Split into three tasks: a reusable crate
audio module first, then the example wiring, then docs - so the module lands as
a game-agnostic building block rather than example glue.

## What went well

- Verifying the exact bevy 0.19 audio API against the vendored crate source
  (`bevy_audio-0.19.0/src/{audio,volume}.rs`) before writing meant the module
  compiled first try - `Volume::Linear`, `PlaybackSettings::DESPAWN`,
  `with_volume`/`with_speed` were all confirmed, no fix-compile churn.
- Modelling `PlaySfx` on the existing `modding::GameEvent` (global `Event` +
  `commands.trigger` + `On<PlaySfx>`) kept the new module consistent with crate
  conventions; the review found nothing structural.
- The one real design call - fruit-burst sound placement - was reasoned out
  before coding: hanging `splat` off `resolve_slice_pop` (which only fruit
  enters) instead of the generic `on_fragments_spawned` observer avoids playing
  a fruit splat over a bomb's fragments. Both bomb and fruit explode into
  `ExplodeFragments`, so the generic observer would have doubled it.
- Placeholder generation via the Python stdlib (`wave`/`struct`) is
  reproducible and byte-deterministic (verified by re-running the script and
  `git diff --quiet`), with zero external tool dependency.

## Difficulties / bugs

- tatr `new` uses a second-resolution timestamp ID. Creating all three tasks in
  one shell line made the later `new` calls clobber the first two (same ID, same
  dir). Fix: space `tatr new` calls across separate tool calls so the IDs
  differ. This is the second time this bit a session (see the bevy-migration
  retro) - treat "one `tatr new` per second" as a hard rule.
- Reviewing my own docs task caught a broken internal link: AGENTS.md pointed at
  `docs/audio.md` while the note was committed under the repo's dated convention
  (`tasks/20260703-152544/NOTES.md`). The plan literally said
  `docs/audio.md` and I followed the repo convention for the filename but not
  for the reference. Lesson: when you deviate from a plan's placeholder name,
  grep for every reference to the placeholder in the same pass.
- The `wav` decoder is not in bevy's default features. Rather than fight to
  generate a valid `.ogg` without ffmpeg/sox, enabling `wav` as a
  dev-dependency feature kept the library's default features clean while making
  the example self-contained. Worth remembering: dev-dependency features are a
  clean way to give examples/tests extra capability without touching the
  published library surface.

## What to do differently

- One `tatr new` per tool call, always.
- After renaming/relocating a planned artifact, grep the tree for the old name
  before closing the task.
- Running the graphical example to completion needs a `nix develop` session;
  in a headless/background run, a timed `cargo run` that boots without asset
  errors is the best available signal, and it did catch that all eight WAVs
  load. Note that limitation in the task rather than claiming full audition.

## Follow-ups

- The branch feature/ninja-sounds is not merged: master advanced in parallel
  (the web-showcase merge), so it needs a real merge the user should own. No new
  tatr tasks opened - the feature is complete as scoped.
