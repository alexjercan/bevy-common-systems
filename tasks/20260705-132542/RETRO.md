# Retro: harvesting the DoomController out of 14_breach

- TASK: 20260705-103238
- BRANCH: fps-harvest (squash-merged to master as 7dc191a)
- REVIEW ROUNDS: 2 (Round 1 REQUEST_CHANGES on 2 major + 1 minor + 2 nit, Round 2 APPROVE)

See `tasks/20260705-103238/TASK.md` and `.../REVIEW.md` for what changed and the
findings; this is about how the working went.

## What went well

- The harvest was scoped by evidence, not vibes: a survey of the crate's harvest
  precedent (`ui/animate`), the avian-coupled template (`pd_controller`), and the
  candidate homes (`camera/wasd`, `point_rotation`) settled the design before any code
  -- output-only (no avian dep), a `DoomEye` child marker, and a firm "do NOT extend
  wasd/point_rotation" with reasons. The module dropped in matching crate shape and the
  rewire shrank `14_breach` by ~113 lines, which is the sign a harvest actually paid off.
- Asking the user about scope surfaced the naming steer -- `DoomController`, reserving
  the premium `FirstPersonController` name for a better future controller. Cheap
  question, saved a rename later and set the module's expectations in its own doc.
- Behaviour was preserved verifiably: the rewired autopilot scored the same 6 kills, so
  the controller still aimed / moved / fired identically.

## What went wrong

- MAJOR (R1.1/R1.2): the rewired example wrote `DoomControllerInput` in systems that
  were not ordered before the module's `DoomControllerSystems::Drive` set, so the
  controller consumed last frame's look/move -- a one-frame input lag. Root cause: the
  module read an `Input` component inside a `SystemSet` but never *documented* the
  ordering contract (write Input before the set), so even I -- writing both the module
  and its first consumer in the same session -- got the consumer's ordering wrong. Only
  `apply_move_velocity` carried an edge (`.after(Drive)`); the symmetric write-before
  edge was missing and undocumented.
- The autopilot could not catch it. It aims by force-writing `DoomControllerState.yaw`
  directly, which *bypasses* the input path entirely, so "6 kills, no panic" said
  nothing about input latency. This is the third time this session a green harness hid a
  path it does not exercise (13_glide's rendered merge, 14_breach's lose condition, now
  the input path). The harness proves the path it drives; it is silent on the rest.

## What to improve next time

- A crate module that reads an `Input` component (or writes an `Output`) inside a named
  `SystemSet` must DOCUMENT the ordering contract on that set: "write Input
  `.before(...)`, read Output `.after(...)`." Without it, every consumer -- including the
  author's own example -- races the set. Added this to AGENTS.md conventions.
- When a test/AI harness shortcuts part of a system graph (here, force-writing state to
  skip the input pipeline), note explicitly which real paths it therefore does NOT
  exercise, and cover those another way (a unit test, or an assertion on the shortcut's
  precondition). The harness's convenience is exactly its blind spot.

## Action items

- [x] Fixed the ordering (chained feeds `.before(Drive)`) and documented the contract on
      the module (in the merged commit); added an `orient_eye` pairing test.
- [x] Added an AGENTS.md convention: document the write-before / read-after ordering
      contract on an Input/Output module's `SystemSet`. Added in this commit.
