# Review: dev/harness autopilot + screenshot state-drivers

- TASK: 20260704-175421
- BRANCH: dev-harness
- REVIEW ROUNDS: 1
- VERDICT: APPROVE (all findings addressed in round 1)

Review was run by an independent skeptical subagent over the commit diff, plus
self-review. The change compiles clean (`cargo clippy --all-targets --features
debug`), all tests pass, headless autopilot runs of 08_dropzone and 11_overload
complete with "cycle complete, no panic", and an end-to-end screenshot writes a
valid PNG. Findings and resolutions below.

## Findings

### MAJOR

**M1 - vent input closure could silently no-op (ordering).** `autopilot_drive`
was an unordered exclusive `Update` system; 11_overload's `vent_input` reads
`just_pressed`, which Bevy clears every frame in `InputSystems` (PreUpdate). So
the vent path was only exercised when the tie-break happened to run the driver
before `vent_input` -- an "aspirational comment not backed by behavior" risk.
RESOLVED: the driver now runs in `PreUpdate` `.after(InputSystems)`, a pinned
direct edge, so a poked `just_pressed` reliably survives into the game's Update
input systems. (08_dropzone was already safe -- it uses level-triggered
`pressed(Space)`.) Follow-on: because the closure now reliably produces input in
every state, the example closures are gated to `GameState::Playing` so they do
not trip the menu/game-over "any key" transitions early; documented on
`AutopilotPlugin::input`. EVIDENCE: a `RUST_LOG=bevy_common_systems=trace` run
of 11_overload under `BCS_AUTOPILOT=1` shows 19 `on_play_sfx` traces at the vent
signature (volume 0.6, variable speed 1.0/1.15) during Playing -- the vent path
genuinely fires now, not a no-op.

### MINOR

**M2 - latent index-out-of-bounds after completion.** On the final step the
driver re-inserted its state with `index == schedule.len()`, relying on the
runner not calling `Update` again after `AppExit`. RESOLVED: added a `done`
flag; the driver early-returns once the timeline is finished, never indexing
past the end.

**M3 - screenshot could hang if the target state is never reached.** RESOLVED:
added a `MAX_WAIT_FRAMES` (~30s) budget; on expiry the plugin logs an error and
exits with `AppExit::error()` instead of hanging. (Autopilot cannot hang -- its
schedule always completes.)

**M4 - `resize_window` dropped `resizable: false`.** The old hand-rolled
harness pinned the window unresizable; the new code only set the resolution, so
a reflowing WM could resize it back and undermine the layout capture. RESOLVED:
`resize_window` now also sets `window.resizable = false`.

### NIT

**N1 - `ScreenshotPlugin` shadows Bevy's internal
`bevy::render::view::screenshot::ScreenshotPlugin`.** Not a `bevy::prelude`
collision (verified against bevy_render-0.19.0), so the crate's naming rule is
satisfied and the examples compile unambiguously. KEPT the natural name (a
rename would ripple through five files and the docs for a non-collision, and
create an odd asymmetry with `AutopilotPlugin`); added a doc note on the struct
distinguishing the two. Decision, not an oversight.

**N2 - transition/capture logs use `info!` where the convention says `trace!`
in systems.** KEPT: these log lines are the tool's entire deliverable (the
human greps for `autopilot: -> Playing` / `cycle complete, no panic`), so they
must show at the default log level. Reviewer agreed it is defensible; the
deviation is deliberate.

**N3 - `parse_resolution` accepted zero/negative dimensions.** RESOLVED: it now
requires both dimensions `> 0.0` (else `None`); added
`rejects_non_positive_dimensions` test.

**N4 - both env vars set would make the two harnesses fight over `NextState`.**
RESOLVED: `ScreenshotPlugin` now stands down (warns and adds nothing) when
`BCS_AUTOPILOT` is also set; autopilot wins.

## Categories the reviewer cleared

- Screenshot capture/exit ordering: correct -- `save_to_disk` writes the PNG
  synchronously in its observer, and `AppExit` is a deferred message read after
  the schedule, so the file always lands before exit.
- `parse_resolution` core parsing, generic bounds (`S: States +
  FreelyMutableState`), `schedule[0]` guarded by the non-empty check,
  `remove_resource().expect(...)` cannot fire, prelude/ASCII, module + item
  docs: all clean.
