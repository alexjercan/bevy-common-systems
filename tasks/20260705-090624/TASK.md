# Build examples/13_glide -- UI-forward slide-merge (2048-style) puzzle

- STATUS: OPEN
- PRIORITY: 80
- TAGS: spike,feature,example,ui

## Goal

Add `examples/13_glide`: a UI-forward slide-merge (2048-style) puzzle. It is the
canonical headline demo of `tween` (tile slide/pop animations on Bevy UI nodes)
and of `persist` + `scoring/high_score` (a saved best score, which a 2048 lives
on). 4x4 board, swipe + arrow-key input, standard 2048 rules,
menu/playing/game-over states, `ui/popup` / `ui/menu` / `SfxPlugin`, `Camera2d`,
wasm/trunk build -- following the `06_fruitninja` shape.

Per user steer: treat UI quality as a first-class goal and produce reusable,
copy-pastable UI patterns. Core structure: a static responsive CSS-grid `Node`
underlay for the cells + a separate absolutely-positioned tile layer whose
`left`/`top` is driven from a `Tween<Vec2>`. Ship UI-juice helpers game-local
(node color flash / pop, animated number) -- do NOT promote to the crate here;
that is the follow-up.

## Notes

Spike: docs/spikes/20260705-090421-ui-forward-slide-merge-puzzle.md

This is a stepless direction-level task -- run `/plan` on it first to break it
into steps before `/work`. Open unknowns to resolve during planning/impl are in
the spike's "Open questions" (UI pop scaling on a `Node`, swipe->direction
resolution, whole-board tween coordination).
