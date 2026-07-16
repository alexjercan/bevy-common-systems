# Retro: dev/harness autopilot + screenshot state-drivers

- TASK: 20260704-175421
- BRANCH: dev-harness (squash-merged to master as c0c6617)
- REVIEW ROUNDS: 1 (APPROVE; 1 MAJOR, 3 MINOR, 4 NIT, all addressed in round 1)

Wave 1 headline of the dev-harness-and-app-scaffolding spike
(`tasks/20260704-175058/SPIKE.md`). See `tasks/20260704-175421/TASK.md`,
`REVIEW.md` and `docs/dev-harness.md` for what was built and the design
decisions; this retro is only about how the working went.

## What went well

- Facts-first against the actual API surface, not memory. Before writing a line
  I verified every uncertain Bevy 0.19 primitive against the vendored crate
  source: `AppExit` is a `Message` (so `world.write_message` / `MessageWriter`,
  not `EventWriter`); the screenshot flow is `Screenshot::primary_window()` +
  `save_to_disk`; `WindowResolution::set`; `FreelyMutableState` lives at
  `bevy::state::state::` and is not in the prelude; `InputSystems` (plural in
  0.19) owns the keyboard clear. The first full-feature compile was clean. This
  is the standing "copy/verify the 0.19 surface, do not improvise it" lesson
  applied up front, and it paid off again.
- Parallel understand-first. An Explore agent mapped the state enums, input
  resources and `main()` boilerplate across 08/09/11 while I read the debug
  module and recent retros. That surfaced the load-bearing design fact early:
  variant names differ (`Result` vs `GameOver`) and per-frame input has no
  common shape, so the game must supply the timeline as `S` values and the
  input as a closure. The generic seam fell out of the evidence, not a guess.
- Evidence before claim, including the hard-to-see half. The skeptical review
  (below) flagged that the vent path *might* be a silent no-op. Rather than
  reason it away, I traced `RUST_LOG=bevy_common_systems=trace` and confirmed 19
  vent SFX actually fire under autopilot. The "no panic" line alone would have
  been a hollow proof; the SFX trace is the real one. Same for the screenshot:
  I did not call it done until a PNG existed and I had *looked* at it (which is
  what caught the inspector-overlay problem, see below).
- The dev tool ate its own dog food. `08_dropzone` and `11_overload` were
  verified with the very plugin this task shipped, and the `$DISPLAY`-set
  screenshot let me see and fix the overlay. The tool proved itself by being
  used, not just by compiling.

## What went wrong

- The MAJOR bug (M1) was mine and shipped in the first draft: `autopilot_drive`
  was an unordered exclusive `Update` system, so the input closure's
  `just_pressed` poke raced the game's `just_pressed` read and Bevy's
  per-frame `InputSystems` clear. For 11_overload's `just_pressed`-based vents
  that meant the "drive the vent path" comment could be a no-op while the run
  still logged "cycle complete, no panic" (the autopilot force-sets state
  regardless). Root cause: I reasoned about *what* the closure pokes but not
  *when* it runs relative to input collection and the game's readers -- the
  exact "order against a real edge, not by luck" caveat AGENTS.md already
  documents for `camera/shake`. Fix was a pinned `PreUpdate.after(InputSystems)`
  edge. A self-review question -- "in which schedule, relative to the input
  clear, does this poke land?" -- would have caught it before review.
- Fixing M1 exposed a second-order bug I had not considered: once the closure
  reliably produces input in *every* state, it trips the menu/game-over "any
  key" transitions early, so the timeline fought the game's own state machine.
  Root cause: I treated the input closure as "gameplay input" without accounting
  for it running outside gameplay states. Fixed by gating the example closures
  to `GameState::Playing` and documenting the pattern. Lesson: a global
  per-frame input hook is not state-scoped for free; either the driver or the
  closure must be.
- The screenshot's first capture was cluttered by the inspector's physics-
  diagnostics panel (the `debug` feature starts it enabled), which defeats the
  layout-verification purpose. Caught only because I looked at the PNG, not from
  any check. It was a genuine miss in the initial design; the fix
  (`hide_debug_overlay`) is small but should have been foreseen from "this runs
  under `--features debug`, which turns the inspector on."

## What to improve next time

- For any system that reads or writes shared frame state (input, events,
  state), decide its schedule and ordering edge *as you write it*, and pin a
  direct `.before`/`.after` against a concrete member -- never leave it to
  executor tie-break. This is the third module to be bitten by ordering
  (`camera/shake`, now here); it is a standing rule, not a one-off.
- When a tool runs behind a feature flag, enumerate what else that flag turns
  on (here: the inspector overlay) and design around it up front.

## Action items

- [x] Lessons captured here. The ordering rule is already in AGENTS.md
  (Conventions, "Ordering caveat"); this is its second recurrence, so if it
  bites a third time it should be promoted from a caveat to a checklist item.
- [x] AGENTS.md updated as part of the task: the "add a TEMPORARY harness"
  gotchas now say "add `AutopilotPlugin`/`ScreenshotPlugin` + set the env var",
  and `debug/harness` is in the Module Map.
- [ ] No follow-up code tasks. Wave 2 of the spike (`assets` registry,
  `HighScore<T>`, `ui/menu` builders, `AnyStartPress`) remains open as its own
  seeded tasks; the deferred `game_app()`/run-reset layer still waits on the
  `tasks/20260704-134800` user decision. None re-seeded here.
- [ ] Optional, not filed: migrate the other stateful examples (06, 07, 09, 10)
  onto the harness for uniformity; 08 and 11 are enough to prove it.
