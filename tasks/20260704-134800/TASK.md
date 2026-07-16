# PROPOSAL (needs user): ui/menu button builder + game-flow state scaffolding (Wave 3)

- STATUS: OPEN
- PRIORITY: 15
- TAGS: spike,suggestion,ui

> Spike: tasks/20260704-134035/SPIKE.md (read
> first). Wave 3 -- DEFERRED, needs a user decision before any code.

## Goal

Five games (06, 07, 08, 10, 11) copy a Menu / Playing / GameOver state enum
plus button-spawning UI. Two parts, split by risk:

- LOW RISK, in scope: a `menu_button()` bundle builder mirroring
  `ui/status::status_bar_item()` -- a plain, opinion-light helper for the
  repeated button UI. This half can proceed once approved.
- NEEDS A USER DECISION: a reusable game-flow *state machine*. The state enum
  is game-specific and a generic version edges toward the "framework
  machinery" the crate charter explicitly warns against (AGENTS.md "What This
  Crate Is"). Do NOT build the state layer without an explicit call on how far
  to generalize.

Action: surface the state-machine question to the user; if they only want the
button builder, scope this down to that and drop the state part.
