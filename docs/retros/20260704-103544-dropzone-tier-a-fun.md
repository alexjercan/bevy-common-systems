# Retro: 08_dropzone Tier-A fun pass

- TASK: 20260704-103544
- BRANCH: feature/08-dropzone-fun (merged to master as f6d6280)
- REVIEW ROUNDS: 2 (R1 REQUEST_CHANGES -> R2 APPROVE)

See `tasks/20260704-103544/TASK.md` close-out and
`docs/2026-07-04-dropzone-tier-a-fun.md` for what was built. This is about how
the working went. Grew out of the fun+mobile spike (`tasks/20260704-102022`).

## What went well

- Read the load-bearing source before writing the tricky parts, so they were
  reasoned not guessed: `chase.rs` (the `ChaseCameraSystems::Sync` set runs in
  PostUpdate and writes camera translation absolutely, so the camera punch had
  to be an additive offset ordered after it), `explode.rs` (it only adds
  fragments, it does not hide the hull, which is why "keep the landed ship
  visible" needed the `DespawnOnExit` lifecycle rework), and `builder.rs`
  `apply_noise` (surface radius = `R * (1 + noise(dir))`, so the pad beacon sits
  flush). All compiled first try, and the noise reading paid off twice when the
  pad later had to be randomized.
- Ported known-good shapes from `07_orbit` verbatim (`FloatingText`,
  `CameraShake`, `apply_camera_shake`) instead of inventing, so the feel matches
  the sibling example and the code stayed consistent.
- Actually ran the example, not just built it (the repeated AGENTS.md gotcha).
  With no `xdotool` for input injection, revived the tuning task's env-gated
  autopilot (`DROPZONE_SMOKE`) to fly the real systems through
  Menu -> Playing -> Result to a clean landing, confirmed the new paths (random
  pad, cans held at 3, guide arrow), then removed the harness before commit.
- Delegated an independent skeptical review to a subagent (the dropzone-example
  retro's lesson, applied on purpose). It caught the `CameraShake`-not-reset bug
  and the untested fuel cap - both real, both cheaper to catch there than in a
  play-test.

## What went wrong

- The first pass shipped a FIXED pad and FIXED fuel-can positions; the user
  immediately asked to randomize the pad, randomize + maintain the fuel field,
  and add a guide arrow. Root cause: the spike and task framed A1/A2 statically
  ("a pad", "cans on the descent line") and I implemented exactly that without
  asking whether per-run variety was wanted. For a game example, replayability
  is almost always wanted and cheap; this turned into a whole second
  implementation round (pad moved `setup` -> `start_run`, fuel became a spawner,
  a new guide system). The first pass's scoring and lifecycle carried over, so
  it was not wasted, but a one-line planning question would have front-loaded it.
- `CameraShake` was not reset in `start_run` (R1.4): I reset Fuel / Outcome /
  RunTimer / ShipInput but forgot the shake resource I had added in the same
  change, so residual crash-trauma bled into the next run. Root cause: added a
  new cross-run resource without adding it to the run-reset step.
- The first `maintain_fuel_cans` replaced a collected can instantly (the timer
  started <= 0) and let the timer run unboundedly negative. Caught by my own
  re-reading during the smoke test, not by a test (ECS systems are hard to unit
  test) nor the reviewer. Root cause: wrote the "keep N" guard without matching
  the "over time" intent; the fix was to prime the timer while at target and
  only count down when below it.

## What to improve next time

- When adding a mechanic to an example/game, ask up front "should this vary per
  run?" - fixed-then-randomized is a full extra round, and variety is usually
  the point of a game demo.
- Any new resource that persists across states and matters per run must be reset
  in `start_run` (now: Fuel, Outcome, RunTimer, ShipInput, CameraShake,
  FuelSpawner). A single `reset_run_state` helper would make the omission
  impossible; worth doing if a fourth resource appears.
- For a time-paced spawner, decide the timer semantics explicitly ("prime while
  full, count down only when below target") rather than a bare decrement, which
  gives both instant refill and unbounded drift.

## Action items

- [x] Proposed AGENTS.md addition: the env-gated-autopilot smoke-test technique
  for verifying a stateful example's gameplay headlessly when no input-injection
  tool is available (this is its second use, after `tasks/20260703-213510`).
- [ ] Optional follow-up (not filed): if 08_dropzone gains a fourth per-run
  resource, extract a `reset_run_state` helper in `start_run`.
