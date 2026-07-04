# Review: ui/popup floating +N text module

- TASK: 20260704-134530
- BRANCH: feat/ui-popup

## Round 1

- VERDICT: APPROVE (with one MINOR test-gap addressed at implementer discretion)

Independent review traced `animate_popups` (rise/fade/despawn, lifetime<=0 guard,
Option-queried Node/TextColor so despawn always fires), confirmed 07_orbit
behavior is preserved (module defaults 0.8/70 match the old consts; STREAK banner
keeps rise 28 + centered layout + DespawnOnExit; no popup dropped), verified the
base-alpha fade is equivalent for 07's opaque colors, and checked conventions and
the doctest. Full suite passes.

- [x] R1.1 (MINOR) src/ui/popup.rs:116-117 - the doc claims "an expired popup is
  despawned even if it lacks [Node or TextColor]" but no test backs the no-Node
  despawn path (the "aspirational comment, test only the easy half" pattern
  AGENTS.md flags). Add an ECS test spawning a bare `Popup` (no Node/TextColor),
  step past lifetime, assert it despawned.
  - Response: Added `bare_popup_without_node_or_text_despawns`, which spawns a
    `Popup` with no `Node`/`TextColor`, steps past its lifetime, and asserts the
    entity is gone.

## Round 2

- VERDICT: APPROVE

R1.1 resolved: `bare_popup_without_node_or_text_despawns` added and passing (5
lib tests total). Full suite green: fmt, clippy, cargo test, cargo test
--examples, check-ascii; 07_orbit boots to the render loop.
