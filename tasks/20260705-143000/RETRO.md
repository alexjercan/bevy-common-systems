# Retro: 13_glide auto-solver

- TASK: 20260705-140110
- BRANCH: feature/glide-solver
- REVIEW ROUNDS: 1 (APPROVE, two NITs both addressed in the same round)

See `tasks/20260705-143000/NOTES.md` for the design and `TASK.md`/`REVIEW.md`
for the plan and findings; this is only the process retro.

## What went well

- **Reused the example's pure move logic instead of forking a headless path.**
  The example already exposed `apply_move` / `is_game_over` as pure functions
  off the ECS (a deliberate choice from the 13_glide build). The solver produces
  a `Direction` and feeds the same `start_move` path a human swipe does, so
  animation/scoring/sound/win-lose were shared for free and the solver cannot
  drift out of sync with the rules. Feature was purely additive.
- **Tested the decision function to the actual goal, not to "it runs".** Per the
  crate rule that a function returning a decision must be asserted directly, the
  key test drives a full deterministic game (an LCG standing in for the spawner)
  and asserts the solver reaches the 2048 tile -- the literal ask. That is far
  stronger evidence than the autopilot, which (per the 13_glide and 14_breach
  retros) force-drives states on a timer and cannot observe game-driven progress.
  The autopilot was still useful, but only as a boot/no-panic check.
- **Parametrised search depth so the test could stay cheap.** A depth-3 full-game
  test would be multi-second (~110k board evals/move); `best_move_with_depth` let
  the test run depth 2 (still reaches 2048) while the game ships depth 3. Keeping
  the depth a parameter, not a const baked into the search, made this a one-liner.
- **Verified the visual layer with a live window grab.** `$DISPLAY` was set, so a
  timed `xdotool`+`scrot` grab of the running window confirmed the new "AUTO" HUD
  tag renders and (via a persisted `Best: 7292`) that the solver really climbs --
  catching anything a state-entry screenshot would miss.

## What went wrong

- Nothing blocking. Two NITs, both self-caught in review:
  - **R1.1 pacing clock.** The first cut gated the interval timer behind the
    `anim.active` check, so the timer paused during each slide and the true
    cadence was `interval + slide` (~0.43s), not the 0.32s the doc claimed. Root
    cause: wrote the gate as one combined early-return without separating "should
    I fire now" (timer) from "am I allowed to fire now" (not animating). Fix:
    advance the timer every enabled frame, gate firing on `timer <= 0 ||
    anim.active`; the interval being longer than a slide makes it exact.
  - **R1.2 sample bias.** `chance_value` sampled the first 6 empties in row-major
    order (`take(6)`), a fixed top-left subset. Harmless here (still reaches 2048)
    but a systematic bias. Fix: stride across the empties so the sample spreads.

## What to improve next time

- When a timed action must not overlap an animation, separate the two clocks
  from the start: one counts down the cadence unconditionally, the other just
  holds the fire. Folding both into a single `if paused || cooling { return }`
  silently couples the cadence to the animation length.
- When approximating an expectation by sampling a fixed subset, spread the sample
  (stride/step) rather than taking a prefix; a prefix of an ordered container is
  almost never a representative sample.

## Action items

- [x] Documented the solver in the AGENTS.md `13_glide` entry and in
      `tasks/20260705-143000/NOTES.md`.
- No new tatr tasks: the solver is intentionally game-local (no second grid-
  puzzle user yet), matching the crate's "wait for a second user" harvest rule.
- No AGENTS.md rule change proposed: the two lessons above are local craft notes,
  not recurring cross-session patterns (the decision-function-testing rule they
  lean on is already in AGENTS.md).
