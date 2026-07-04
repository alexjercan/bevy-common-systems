# input AnyStartPress + adopt UnifiedPointer; leaf helpers giveup_on_escape/status_bar_with_fps/glowing_material (Wave 2)

- STATUS: OPEN
- PRIORITY: 40
- TAGS: spike,feature

> Spike: docs/spikes/20260704-175058-dev-harness-and-app-scaffolding.md (read
> first). Wave 2 -- tiny extensions to already-shipped modules + leaf one-liners.

## Goal

A cluster of tiny, high-frequency harvests, grouped because each is only a few
lines:

- **`AnyStartPress` on `input/pointer`.** The keyboard-inclusive "advance on any
  press" check (`mouse.just_pressed || keys.any || touches.any_just_pressed`) is
  copy-pasted ~7x for menu/game-over dismissal (07:677; 08:1067; 09:774; 11:599;
  ...). Add a `just_started()`-style helper. Related: `UnifiedPointer` already
  ships but only 10_asteroids uses it -- migrate the other five games onto the
  existing resource so the crate's own input module is actually adopted.
- **`giveup_on_escape`** -- the identical 3-line "Escape just_pressed -> set
  GameOver" system in 5 games; a reusable key-to-state helper.
- **`status_bar_with_fps()`** -- the identical 8-line FPS `status_bar_item` spawn
  block copied in all six; a convenience over the existing `ui/status`.
- **`glowing_material(base, emissive)`** -- the emissive-blooms-never-`unlit`
  `StandardMaterial` idiom retyped 4-5x; a helper that bakes in the footgun
  (never set `unlit: true` on an emissive material, per AGENTS.md).

Prove each by refactoring the affected games. This task is stepless on purpose
(spike output); run /plan to break it into steps before /work. Split into
smaller tasks at plan time if the cluster is too broad for one branch.
