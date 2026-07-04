# Retro: 08_dropzone hazards pass

- TASK: 20260704-103553
- BRANCH: feature/08-dropzone-hazards (squash-merged to master as 519483c)
- REVIEW ROUNDS: 1 (R1 APPROVE)

See `tasks/20260704-103553/TASK.md` close-out and
`docs/2026-07-04-dropzone-hazards.md` for what was built. This is about how the
working went. Third follow-up from the fun+mobile spike
(`tasks/20260704-102022`, Part 1 Tier B), on top of the Tier-A fun pass.

## What went well

- Read the load-bearing crate APIs before writing, via a focused subagent that
  returned exact signatures and field names for `RandomSphereOrbit`,
  `HealthPlugin` (`HealthApplyDamage` is an `EntityEvent`, `HealthZeroMarker`
  observed with `On<Add>`) and `07_orbit`'s proximity-collision pattern. The
  code compiled with only two trivial errors (both `play_sfx` vs
  `commands.trigger` for volume), and the design choices (proximity for
  asteroids, marker classification in `resolve_collisions`) were reasoned from
  the real source, not guessed.
- Reused existing crate systems throughout (`transform/random_sphere_orbit`,
  `health`, `mesh/explode`, `helpers/temp`), exactly as AGENTS.md asks. The
  example gained four hazard families without any new framework machinery, and
  the `resolve_landing` -> `resolve_collisions` refactor made the contact logic
  stricter (planet vs obstacle) rather than bolting hazards onto the side.
- Applied the Tier-A retro lesson directly: the new cross-run `Wind` resource is
  reset in `start_run` alongside the others. The specific bug that retro called
  out (adding a per-run resource and forgetting to reset it) did not recur
  because I looked for it on purpose.
- Ran the example, did not just build it (the recurring AGENTS.md gotcha). With
  no input-injection tool, revived the env-gated autopilot (`DROPZONE_SMOKE`) and
  extended it: snapping the ship's `Position` directly onto an asteroid, then
  onto a rock, forced the proximity-hit + shatter path and the obstacle-collision
  path deterministically, which a free-fall autopilot would only hit by chance.
  That flushed out that all the hazard systems run together without a query
  conflict and that the integrity-kill chain
  (`HealthApplyDamage` -> `HealthZeroMarker` -> `on_ship_destroyed`) fires.
- Delegated an independent skeptical review to a subagent (the standing lesson).
  It cross-checked the health/explode/orbit source and confirmed no correctness
  bugs, and surfaced the one cosmetic edge worth documenting (a lethal graze in
  the same frame as a soft touchdown reads as LANDED), which became a one-line
  code comment.

## What went wrong

- The first smoke run ended suspiciously fast (0.42s into Playing) and I could
  not tell from it whether that was a legitimate asteroid hit or an
  instant-spawn-collision bug, so I had to write a second, cleaner smoke pass.
  Root cause: the first harness conflated "force hazard hits" with "observe
  whether the run survives", so a fast end was ambiguous. The fix was to
  establish a baseline first (free flight with hover-thrust survives several
  seconds at hull 100, confirming no spawn bug) and only then force each hazard.
  Lesson: a verification harness should prove the baseline before perturbing it.
- Two `play_sfx` signature errors: I wrote
  `commands.play_sfx(PlaySfx::new(..).with_volume(..))`, but `play_sfx` takes a
  bare `Handle`; volume needs `commands.trigger(PlaySfx..)`, as the existing
  pickup code already did. Caught at compile, but a glance at the sibling call
  site would have avoided it -- the same "copy the idiom from an existing call,
  not from memory" lesson that keeps recurring for Bevy APIs.
- The first web build was silently truncated: I wrapped `npm run build` in
  `timeout 550`, which killed it mid `build:games` (on 10_asteroids, with more
  games and the whole webpack step still pending), and the compound command's
  exit 0 was `tail`'s, not npm's -- exactly the piped-tail-masking gotcha. I
  only caught it by grepping the log for a webpack success marker and finding
  none. Re-running detached (no foreground timeout) to completion, then gating on
  `BUILD_EXIT=0` plus "webpack compiled successfully", got a real green.

## What to improve next time

- For a slow multi-step build (the whole wasm showcase is minutes), run it
  detached (`run_in_background`) with no foreground `timeout` cap, and judge it
  by an explicit success marker (`BUILD_EXIT=0` and webpack's
  "compiled successfully"), never the wrapper command's exit code. A `timeout`
  that fires mid-build looks like success through a trailing `tail`.
- Design a smoke/verification harness to establish the baseline (does the normal
  path survive?) before forcing the thing under test, so an early exit is
  unambiguous rather than needing a second run to interpret.
- When calling a crate helper (`play_sfx`), check the one existing call site for
  the exact form before writing a variant; the volume path was already in the
  file.

## Action items

- [ ] Follow-up (now more warranted): `start_run` resets seven cross-run
  resources (Fuel, Outcome, RunTimer, ShipInput, CameraShake, FuelSpawner,
  Wind). Extract a `reset_run_state` helper so a future eighth cannot be
  forgotten. Carried over from the Tier-A retro; the count has only grown.
- [ ] Optional AGENTS.md note: the "baseline-before-perturb" smoke-harness
  refinement, and gating the web build on an explicit marker rather than a
  wrapper exit code (a specific case of the existing piped-tail gotcha). Left
  unfiled unless it recurs.
