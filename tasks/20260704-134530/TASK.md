# ui/popup: floating +N text module (Wave 1)

- STATUS: OPEN
- PRIORITY: 38
- TAGS: spike,feature,ui

> Spike: docs/spikes/20260704-134035-game-juice-and-scaffolding-kit.md (read
> first). Wave 1 -- promote recurring example juice into the library.

## Goal

Add a `ui/popup` module for the floating "+N" score/damage text that three
games (06, 07, 08) hand-roll: a helper (e.g. `spawn_popup(text, pos, color)`
or a `Commands` extension) that spawns a label which rises and fades over a
lifetime, then self-despawns (build on `helpers/temp`).

Decide at planning time (spike open question) between worldspace billboarded
3D text and a screen-space UI node tracked to a projected world point; if the
latter, the popup helper needs a camera handle for world-to-screen. Prove it
by refactoring one example (07_orbit) onto the module. Once the `tween` module
(task 20260704-134630) exists, back the rise/fade with it rather than a bespoke
lerp.
