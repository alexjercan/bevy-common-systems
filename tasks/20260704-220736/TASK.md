# build examples/12_warden -- top-down click-defense, canonical demo of camera/project

- STATUS: OPEN
- PRIORITY: 2
- TAGS: spike,examples,game

> Spike: docs/spikes/20260704-220530-new-prototype-game-ideas.md (read first).
> Recommended game #1 (Option A).

## Goal

A small (~2000 line) top-down click-defense prototype that is the **canonical,
first, validating demo of `camera/project`** -- a module that was harvested from
06/07/08/10 but is imported by no example (they still hand-roll their own
`viewport_to_world` glue). Building on it both showcases and battle-tests it.

Concept: a top-down camera over a `y = 0` play plane; a central core with a
`Health` pool; octahedron enemies spawn at the arena edge and crawl toward the
core. Click/tap anywhere on the ground -- `pointer_on_plane` turns the pointer
into a world point -- to fire at that spot; a hit inserts `ExplodeMesh` and pops
a `ui/popup` "+N" anchored over the enemy via `world_to_screen`. `ui/status`
HUD shows core integrity / wave / score; the core reaching zero
(`HealthPlugin` -> `HealthZeroMarker`) ends the run. Difficulty ramps enemy
count/speed per wave.

Must exercise both `camera/project` functions (the headline) and reuse the
established example shape: menu/playing/game-over `States`, `SfxPlugin`
one-shots, a wasm/trunk showcase build, touch-native (tap == click). Reuse the
juice kit (`camera/shake`, `feedback`, `tween`, `scoring/streak`,
`time/cooldown` for a fire cooldown, `input/pointer`) rather than re-rolling it.

Open at implementation time (from the spike): orthographic vs high-perspective
camera for the plane pick -- both work with `pointer_on_plane`; decide with a
`ScreenshotPlugin` grab. Differentiate the feel from `10_asteroids`' pointer
control via the placement/defense loop, not the input.

This task is stepless on purpose (spike output); run /plan to break it into
steps before /work.
