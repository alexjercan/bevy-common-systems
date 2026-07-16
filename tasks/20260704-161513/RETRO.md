# Retro: ui/touchpad reveal-on-touch + hit-test primitives

- TASK: 20260704-161513
- BRANCH: feat/ui-touchpad (squash-merged to master as e035b98)
- REVIEW ROUNDS: 1 (APPROVE, 1 informational NIT accepted)

Wave A item 3, and the last of the input/projection harvest's first wave
(`tasks/20260704-161210/SPIKE.md`). The
most-documented duplication in the repo (4 touch docs, 3 touch retros) is now a
module.

## What went well

- Asked the one real scope fork up front instead of guessing. The task flagged
  a NEEDS-A-DECISION (bare primitives vs a button-row builder) that the spike
  said might need a user call; surfacing it with `AskUserQuestion` before
  writing any code got a clear "primitives only" and avoided building an
  opinionated pad widget the user did not want. This is exactly the "ask at the
  genuine fork, then proceed" the flow prescribes.
- Last cycle's AGENTS.md gotcha paid off immediately: I checked every new
  prelude name (`TouchSeen`, `RevealOnTouch`, `button_grid_at`, ...) against
  `bevy::prelude` before committing, so there was zero repeat of the `Pointer`
  collision. The compounding worked -- a lesson written down last cycle
  prevented the same class of bug this cycle.
- For a relocate-the-logic refactor, traced the exact visibility semantics
  (`Hidden`/`Visible` for the pad, `Inherited`/`Hidden` for the legend) from
  the deleted systems into the new markers, and proactively caught the one real
  behaviour change -- `mark_touch_seen` now runs in all states, so a menu tap
  marks `TouchSeen` -- before review rather than shipping it silently.

## What went wrong

- Nothing of substance. The lone NIT was an informational note about the
  all-states mark (a benign improvement), accepted as intended. Generalizing
  the hit-test also fixed two latent bugs in the originals for free (a
  `radius == dead` div-by-zero NaN in `touch_lean`, and a missing upper-y bound
  in `vent_button_at`), both unreachable in practice but strictly more correct.

## What to improve next time

- Keep asking the genuine scope fork before building, not after. It cost one
  `AskUserQuestion` round and saved a whole builder API's worth of work that
  would then have been argued over in review.
- Keep tracing relocate-the-logic refactors semantics-first (enumerate the old
  states, map each to the new mechanism, name any state that changes). It is
  what turns a "looks equivalent" refactor into a verified one.

## Action items

- [x] `ui/touchpad` shipped; Wave A of the input/projection harvest is complete
  (`camera/project`, `input/pointer`, `ui/touchpad` all landed).
- [ ] Wave B remains (sketch-then-commit each): `scoring` (tatr 20260704-161518),
  radial gravity (tatr 20260704-161522), `progress` (tatr 20260704-161526).
  Also still open from Wave A's tail: the enhanced-input pointer bridge
  (tatr 20260704-173937).
