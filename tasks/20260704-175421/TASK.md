# dev/harness: autopilot + screenshot state-driver plugins (Wave 1, headline)

- STATUS: CLOSED
- PRIORITY: 80
- TAGS: spike,devtools,feature

> Spike: tasks/20260704-175058/SPIKE.md (read
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

## Steps

- [x] 1. Scaffold `src/debug/harness/` as a submodule of the `debug` feature:
  `mod.rs` (module doc, env-var consts `BCS_AUTOPILOT`/`BCS_SHOT`, shared
  state-driver helpers, `prelude`), `autopilot.rs`, `screenshot.rs`. Wire into
  `src/debug/mod.rs` and its prelude.
- [x] 2. `AutopilotPlugin<S: States + FreelyMutableState>`: env-gated on
  `BCS_AUTOPILOT` (inert unless set, so no add/remove churn). Builder
  `.hold(state, seconds)` to declare the timeline and `.input(Fn(&mut World,
  f32))` for the optional per-frame gameplay closure. An exclusive driver
  system force-sets `NextState<S>` per the schedule, logs each transition and a
  final "cycle complete, no panic" line, runs the input closure each frame, then
  writes `AppExit::Success` (clean exit, no `std::process::exit`).
- [x] 3. `ScreenshotPlugin<S: States + FreelyMutableState>`: env-gated on
  `BCS_SHOT` (value `WxH` overrides the window resolution). Builder `.state(s)`,
  `.settle_frames(n)`, `.path(p)`. Resize the primary window at startup,
  auto-advance to the target state, wait N settled frames, capture via Bevy
  0.19 `Screenshot::primary_window()` + `save_to_disk`, then `AppExit::Success`.
- [x] 4. Refactor `08_dropzone` (physics) onto the harness: gate the two plugins
  behind `#[cfg(feature = "debug")]` (like `InspectorDebugPlugin`), schedule
  Menu -> Playing -> Result with a `ShipInput` thrust closure. Verify headlessly
  with `BCS_AUTOPILOT=1 cargo run --example 08_dropzone --features debug` under
  `timeout`: confirm the transition log + "cycle complete, no panic".
- [x] 5. Refactor one 2D game (`11_overload`) onto the harness the same way
  (schedule Menu -> Playing -> GameOver, vent closure). Verify headlessly.
- [x] 6. Replace the AGENTS.md "add a TEMPORARY env-gated autopilot/screenshot
  harness" gotcha with "add the harness plugin + set `BCS_AUTOPILOT`/`BCS_SHOT`";
  add `debug/harness` to the Module Map.
- [x] 7. Verify the whole suite: `cargo fmt`, `cargo clippy --all-targets
  --features debug`, `cargo test --features debug`, `cargo test --examples`,
  `scripts/check-ascii.sh`. Write the decision doc in `docs/`.
