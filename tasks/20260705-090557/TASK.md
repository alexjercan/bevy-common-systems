# Follow-up: evaluate promoting 13_glide UI-juice patterns into the crate

- STATUS: OPEN
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
