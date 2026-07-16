# 13_glide: a UI-forward slide-merge puzzle

- DATE: 2026-07-05
- Example: `examples/13_glide.rs`
- Spike: `tasks/20260705-090421/SPIKE.md`
- Task: `tasks/20260705-090624`

## What it is

`13_glide` is a 2048-style slide-merge number puzzle rendered entirely in Bevy
UI. It is the headline demo of two crate modules that had no example of their
own before: [`tween`](../src/tween/mod.rs) and the pairing of
[`persist`](../src/persist/mod.rs) with
[`scoring/high_score`](../src/scoring/high_score.rs). Swipe (or arrow keys)
slides every tile; equal tiles that collide merge into their sum; a new tile
appears after each move; filling the board with no legal move ends the run. The
best score is saved across launches.

It adds the first *puzzle* to the gallery (every other example is action or
incremental) and follows the established `06_fruitninja` shape:
menu / playing / game-over `States`, `SfxPlugin` one-shots, a `Camera2d`, and a
wasm/trunk build.

## Why the design is what it is

### Everything animates plain `Node` fields, not `Transform`

The user's steer for this example was "lean on UI, produce copy-pastable UI
patterns". The reusable pattern the example teaches is: **drive UI motion by
tweening `Node`/`BackgroundColor` fields, read from a `Tween<T>` output each
frame.** Concretely:

- a tile **slide** is a `Tween<Vec2>` whose value is written into the tile's
  `Node { left, top }`;
- a tile **pop** (spawn / merge) is a `Tween<f32>` whose value is written into a
  `Node`'s `width`/`height` as a percentage;
- a merge **flash** is a `Tween<Vec4>` (linear-RGBA) written into a
  `BackgroundColor`;
- the score **readout** rolls to its new value on another `Tween<f32>`.

We deliberately avoided `Transform` scale on UI nodes. Bevy's UI layout owns a
node's transform, so animating `Transform` on a `Node` is version-fragile and
was flagged as an open question in the spike. Tweening layout fields instead is
robust and is the more instructive pattern for anyone building animated UI.

### The tile is a wrapper + a face (two nodes)

Position, scale and colour must animate independently without three systems
fighting over the same `Node`. So each tile is:

- a positioning **wrapper** (`Tile`): `position_type: Absolute`, sized to a full
  cell, `justify/align: Center`. Its `left`/`top` is driven by the slide tween.
- an inner **face** (`TileFace`): `width`/`height` in *percent* of the wrapper,
  carrying the `BackgroundColor` and the number `Text`. Its size is driven by
  the pop tween and its colour by the flash tween.

Because the wrapper centres the face, the pop grows from the centre for free.
This wrapper/face split is the second copy-pastable pattern.

### Board layout: fixed-size centered container, not CSS grid

The board is a fixed 346px square (`4*74 + 5*10`) centered in the window, with a
static 16-cell underlay drawn as absolutely-positioned nodes and the tiles as a
second absolute layer on top. We chose a fixed px board over a `Display::Grid`
node because (a) no other example uses CSS grid, so its API was unverified, and
(b) the tiles must animate *between* cells, which a pure grid layout cannot do --
the moving layer has to be absolute anyway. 346px fits a 390px-wide phone with
margin; verified with a `BCS_SHOT=390x844` screenshot.

### Move / merge coordination: a fixed timer, resolved before the tween advance

A move computes the whole result up front (`apply_move` -> new grid + per-tile
`GridMove`s + score), attaches a slide `Tween<Vec2>` to every moving tile, and
arms a `MoveAnim` timer for `MOVE_DURATION`. When the timer elapses,
`tick_move_anim` despawns the tiles that merged away, bumps each survivor
(value + pop + flash + text), spawns one new random tile, adds the score, fires
the "+N" popup and sound, and checks for game over. Input is locked while a move
animates.

The pure logic (`resolve_line`, `apply_move`, `is_game_over`) is separated from
the ECS and unit-tested (8 tests: empty/single/pair/four-equal, the
no-triple-merge rule, left/right mirroring, unchanged detection, and game-over
only when full and locked), per the crate's convention that pure logic gets
in-module tests and ECS behaviour is exercised by the example.

## Bug found and fixed: tween-completion vs despawn race

The first autopilot run flooded the error handler with `Entity despawned`
errors. Diagnosis: `tick_move_anim` despawns each merged-away tile at exactly
`MOVE_DURATION`, which is exactly when that tile's slide tween completes. If the
despawn is queued *after* `TweenSystems::Advance` has already marked the tween
finished, the plugin's completion commands (`insert TweenFinished`, and the
`remove` for the default `Remove` policy) apply to an entity that no longer
exists.

Two changes fixed it, and both are worth remembering when using `tween` with
entities you also despawn:

1. **Run the despawning system before `TweenSystems::Advance`.** `tick_move_anim`
   is ordered `.before(TweenSystems::Advance)`, so the entity is gone before the
   plugin ever processes its completion. This is the real fix.
2. **Use `TweenOnComplete::Keep` on tweens whose entity may be despawned.** With
   `Keep` the plugin never issues a `remove` command; the applier systems simply
   hold the final value. This removes a second, subtler race and costs nothing
   (a few finished `Tween` components lingering on long-lived tiles).

The `TweenFinished` marker is still inserted on completion regardless of policy,
so ordering (1) is the load-bearing fix; `Keep` (2) is defence in depth.

The "+N" popup deliberately carries **no** `DespawnOnExit`: it self-despawns via
its own tween (`PopupPlugin`'s `Despawn` policy), so a state-exit despawn can
never race the plugin's own cleanup.

## Modules exercised

Headline: `tween`, `persist` + `scoring/high_score`. Also `ui/popup` (the "+N"),
`ui/menu` (`centered_screen` / `screen_text` / `TitlePulse` + `MenuPlugin`),
`input/pointer` (`UnifiedPointer` swipe), `input/state` (`set_state_on_key`),
`audio` (`SfxPlugin` + `SoundBank`, reusing the placeholder wavs), and the
`debug/harness` (`AutopilotPlugin` + `ScreenshotPlugin`).

## Follow-up

The three game-local UI-juice helpers here (a UI-node color flash, a UI pop, an
animated number) are the subject of the harvest follow-up `tasks/20260705-090557`
-- whether they should become a crate-level UI-`feedback` sibling to the
material-only `feedback/flash`. They are kept game-local in this example on
purpose.
