# Fruit ninja sound effects

The `06_fruitninja` example (`examples/06_fruitninja.rs`) loads one sound per
gameplay event from this directory. The files committed here are **tiny
generated placeholders** (short sine blips, one distinct pitch each) produced by
`scripts/gen-placeholder-sounds.py` so the example runs and is audible out of
the box. They are not meant to be shipped as the final sound design.

## Dropping in real audio

Replace each file below with a real sound **at the same path and filename**.
No code changes are needed: the example loads these fixed paths, and the crate
audio module (`SfxPlugin`) plays whatever handle it is given.

- Formats: WAV works today (the example enables bevy's `wav` decoder for
  dev builds). OGG Vorbis also works, since vorbis is on by default; to use
  `.ogg` files, change the extensions in the `SfxAssets` load calls in
  `setup` (`examples/06_fruitninja.rs`) from `.wav` to `.ogg`.
- Suggested: 44.1 kHz, mono or stereo, normalized but not clipping. Keep them
  short; these are one-shot effects, not loops.
- To regenerate the placeholders (e.g. after deleting them):
  `python3 scripts/gen-placeholder-sounds.py` from the repo root.

## Required files

| File | Event | Character / length |
| --- | --- | --- |
| `menu_select.wav` | Clicking "play" on the main menu | short UI click / confirm, ~0.1-0.3 s |
| `slice.wav` | A plain fruit is sliced (swipe connects) | crisp blade whoosh, ~0.15-0.3 s |
| `splat.wav` | A sliced fruit bursts into fragments | juicy squish / pop, ~0.2-0.4 s |
| `combo.wav` | A combo reaches x2 or more (pitched up with the chain) | rising chime / ding, ~0.3-0.5 s |
| `golden.wav` | A golden bonus fruit is sliced | bright sparkle / shimmer, ~0.3-0.5 s |
| `bomb.wav` | A bomb is sliced (lethal) | punchy explosion, ~0.4-0.8 s |
| `game_over.wav` | The run ends (game-over screen) | short somber sting, ~0.8-1.5 s |
| `launch.wav` | A fruit or bomb is launched from below | soft airy whoosh, ~0.2 s (played quietly) |

## Per-event volume

The example plays most sounds below full volume (see the `play_sfx_volume`
calls in `examples/06_fruitninja.rs`); `launch.wav` in particular is played
quietly (0.35) because it fires on every spawn. Mix your real assets with that
in mind, or adjust the per-shot volumes in the code. A global
`SfxMasterVolume` resource scales everything at once if you want a master
slider.
