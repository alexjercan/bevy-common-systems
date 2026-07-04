# dev/harness: autopilot + screenshot state-driver plugins (Wave 1, headline)

- STATUS: OPEN
- PRIORITY: 80
- TAGS: spike,devtools,feature

> Spike: docs/spikes/20260704-175058-dev-harness-and-app-scaffolding.md (read
> first). Wave 1 headline -- highest evidence, lowest charter risk.

## Goal

Promote the throwaway verification harness that every game re-invents into one
reusable `dev`/`debug`-feature module (`dev/harness`), so the crate's own "an
example is not done until it has been run once" rule stops costing a
hand-rolled-and-deleted harness per game. Evidence: the autopilot has been
hand-rolled and deleted 7 times and the screenshot harness twice, under
different env-var names each time (`DROPZONE_AUTOPILOT`, `DROPZONE_SMOKE`,
`OVERLOAD_SMOKE`, `REACTOR_SHOT`, ...), and AGENTS.md already documents the
pattern as a standing gotcha and "the single highest-leverage technique."

Two entry points over a shared state-driver core:

- **autopilot** -- an env-gated plugin generic over the game's `States` type
  that force-advances Menu -> Playing -> ... -> GameOver on a schedule, runs an
  optional game-supplied per-frame input closure, logs each transition and a
  final "cycle complete, no panic" line, and exits cleanly (as an `AppExit`, not
  a `std::process::exit` -- AGENTS.md notes the latter segfaults on wgpu
  teardown).
- **screenshot** -- a sibling env-gated plugin that overrides `WindowResolution`,
  auto-advances to a named state, waits N settled frames, writes a PNG, exits.

Charter note: this is dev tooling behind the existing feature flag (like
`debug/inspector`), so the "no framework machinery" rule does not bind it.

Prove it the crate way: refactor at least 08_dropzone (physics) and one 2D game
(09 or 11) onto it and confirm they still verify headlessly. See the spike's
open questions on autopilot generality and screenshot frame-settling. After it
lands, replace the AGENTS.md "add a temporary harness" gotcha with "add the
harness plugin + set the env var."

This task is stepless on purpose (spike output); run /plan to break it into
steps before /work.
