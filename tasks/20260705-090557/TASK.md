# Follow-up: evaluate promoting 13_glide UI-juice patterns into the crate

- STATUS: CLOSED
- PRIORITY: 40
- TAGS: spike,feature,ui,harvest

## Goal

After `13_glide` ships, evaluate promoting its game-local UI-juice patterns into
the crate:

- a UI-node `feedback` sibling: a `Node` `BackgroundColor` flash (`Tween<Vec4>`)
  and a UI pop, paralleling the material-only `feedback/flash`
  (`feedback/flash` clones a `StandardMaterial`, so UI nodes have no equivalent);
- a "tween a `Node`'s `left`/`top` from a `Tween<Vec2>`" glue helper for animated
  UI surfaces (boards, inventories, cards);
- an animated-number readout (roll a displayed integer old->new via a tween).

Decide whether each generalizes cleanly and can reuse `tween` / `feedback`, or
should stay game-local. Depends on the MVP (20260705-090624) shipping first.

## Notes

Spike: docs/spikes/20260705-090421-ui-forward-slide-merge-puzzle.md

Stepless direction-level task -- `/plan` before `/work`. This is the "harvest
after proof" step: the concrete `13_glide` reference must exist before deciding
what to promote.

## Steps (plan)

- [x] 1. New leaf module `ui/animate` (reuses `tween`): `color_to_vec4` /
  `vec4_to_color` helpers; opt-in marker components `TweenNodeOffset`
  (`Tween<Vec2>` -> `Node.left/top` px), `TweenNodeScale` (`Tween<f32>` ->
  `Node.width/height` percent), `TweenNodeBackground` (`Tween<Vec4>` ->
  `BackgroundColor`); `UiAnimatePlugin` runs the appliers after
  `TweenSystems::Advance`; a `node_flash(to, duration)` constructor. Unit tests
  for the color round-trip and the flash constructor.
- [x] 2. Wire `ui/animate` into the `ui` + crate preludes.
- [x] 3. Refactor `13_glide` onto the module: delete local `apply_slide` /
  `apply_pop` / `apply_flash` / `flash_tween` / `color_to_vec4` / `vec4_to_color`;
  add the markers to the tile/face entities and `UiAnimatePlugin`.
- [x] 4. Evaluate the rolling-number readout: ship a `RollingNumber` only if it
  generalizes cleanly (source-of-truth + text formatting are game-specific);
  otherwise keep it game-local and say why.
- [x] 5. Decision doc `docs/2026-07-05-13glide-ui-juice-harvest.md`: per-candidate
  verdict (promote / keep local) with reasoning; update AGENTS.md module map.

## Close-out

Shipped `ui/animate` (the UI-node parallel to `feedback/flash`): opt-in markers
`TweenNodeOffset`/`TweenNodeScale`/`TweenNodeBackground` + `UiAnimatePlugin`,
`color_to_vec4`/`vec4_to_color`, and `node_flash`. Refactored 13_glide onto it
(deleted its local appliers + `TileFace`). The rolling-number readout stayed
game-local -- format + source are game-specific and the reusable core is a thin
tween pattern (docs/2026-07-05-13glide-ui-juice-harvest.md). Reviewed APPROVE in
one round; no follow-up seeded.
