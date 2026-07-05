# Follow-up: harvest FP character controller / camera-wasd upgrades from 14_breach

- STATUS: OPEN
- PRIORITY: 40
- TAGS: spike,feature,harvest,fps

## Goal

After `14_breach` ships, evaluate harvesting its game-local first-person pieces into
the crate:

- a reusable **first-person character controller** (walk + gravity + ground check +
  collide-and-slide), the biggest gap -- the crate has no character controller;
- **`camera/wasd` upgrades**: optional always-on look (not just RMB-drag), a
  cursor-grab / pointer-lock helper, and a pitch clamp -- the small changes that turn
  the free-fly tech-demo camera into a game-ready one;
- optionally a **hitscan / `SpatialQuery` helper** if the raycast-and-damage pattern
  is reusable.

Decide which generalize cleanly (collide-and-slide is level-specific, so the split
matters) and what altitude each belongs at. Depends on the MVP (20260705-103236)
shipping first so there is a concrete reference.

## Notes

Spike: docs/spikes/20260705-103116-grounded-fps-example.md

Stepless direction-level task -- `/plan` before `/work`. Harvest-after-proof: the
working `14_breach` controller must exist before deciding what to promote (mirrors
the `12_bastion` data-catalog and `13_glide` UI-juice follow-ups).
