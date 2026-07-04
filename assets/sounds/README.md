# Example game sound effects

The `06_fruitninja`, `07_orbit` and `11_overload` examples
(`examples/06_fruitninja.rs`, `examples/07_orbit.rs`,
`examples/11_overload.rs`) load one sound per gameplay event from this
directory. The files committed here are **tiny generated placeholders** (short
sine blips, one distinct pitch each) produced by
`scripts/gen-placeholder-sounds.py` so the examples run and are audible out of
the box. They are not meant to be shipped as the final sound design.

`menu_select.wav`, `game_over.wav` and `combo.wav` are shared between the
games; the rest are per-game (`slice`/`splat`/`golden`/`bomb`/`launch` for fruit
ninja, `pickup`/`hurt`/`level_up` for orbit runner, `vent`/`alarm` for
overload, which also shares `menu_select`/`game_over`/`level_up`). Orbit runner
plays
`combo.wav` over the `pickup` blip once a collection streak reaches x2, pitched
up with the chain, the same way fruit ninja uses it for slice combos.

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
| `game_over.wav` | The run ends (game-over screen); shared with orbit runner | short somber sting, ~0.8-1.5 s |
| `launch.wav` | A fruit or bomb is launched from below | soft airy whoosh, ~0.2 s (played quietly) |
| `pickup.wav` | `07_orbit`: an orb is collected | bright blip / chime, ~0.1 s |
| `hurt.wav` | `07_orbit`: a hazard is touched (damage taken) | low thud / buzz, ~0.2 s |
| `vent.wav` | `11_overload`: a gauge is vented back toward green | soft relief hiss, ~0.1-0.2 s |
| `alarm.wav` | `11_overload`: a gauge is in the red (beeps while critical) | sharp warning beep, ~0.15-0.3 s |
| `level_up.wav` | `07_orbit`: a new difficulty level is reached | rising ding, ~0.2 s |

## Web (wasm) builds

These files are shipped into each web build by a `data-trunk rel="copy-dir"`
directive in `web/games/06_fruitninja/index.html` and
`web/games/07_orbit/index.html`; without it trunk copies no assets and the
browser fetch of `sounds/*.wav` 404s (silent game). See
`docs/wasm-web-builds.md` ("Assets") for the copy directive and the exact
fetched URL. Browser audio also needs a user gesture before it will play; the
game satisfies this with the in-canvas click that starts a run, so the first
sound (`menu_select`) fires on that click.

## Per-event volume

The example plays most sounds below full volume (see the `play_sfx_volume`
calls in `examples/06_fruitninja.rs`); `launch.wav` in particular is played
quietly (0.35) because it fires on every spawn. Mix your real assets with that
in mind, or adjust the per-shot volumes in the code. A global
`SfxMasterVolume` resource scales everything at once if you want a master
slider.
