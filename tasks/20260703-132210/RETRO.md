# Retro: Fruit ninja floating +N popup

- TASK: 20260703-132210
- BRANCH: feature/fruitninja-popup (merged to master, deleted)
- REVIEW ROUNDS: 1 (APPROVE, one cosmetic NIT left)

See TASK.md close-out for what shipped; this is process only.

## What went well

- Designed `spawn_floating_text` as a reusable helper up front (per the plan's
  dependency on the combo task), so task C gets "+N" and "COMBO xN" popups for
  free. Building the shared primitive first, then the two callers, is the same
  refactor-before-feature move that paid off in the bombs task.
- Storing the base `color` on `FloatingText` and recomputing alpha from
  `age/lifetime` each frame (instead of multiplying the current color down)
  avoids the classic compounding-fade bug where the popup goes invisible too
  fast. Chose the stateless-fade formulation deliberately.

## What went wrong

- My first two verification boots were misleading. The auto-slicer sliced fruit
  the instant they spawned at y = -10 (below the view), so `world_to_viewport`
  projected them to y ~ 1108 in a ~1057-tall window: valid, but off-screen. The
  first probe reported `max_live = 0` and I could have wrongly concluded popups
  did not spawn; the second showed they spawned but off-screen. Only the third
  probe -- gated to slice only in-view fruit (-6 < y < 8), like a real player --
  proved popups land on-screen. Root cause: the throwaway auto-slicer does not
  reproduce *where* a human slices, and position was exactly what this feature
  is about. A verifier that exercises the code in an unrepresentative regime can
  fail the feature while the feature is fine.

## What to improve next time

- When the thing under test is spatial/positional, make the throwaway harness
  reproduce the real input regime, not just "trigger the code path". Here that
  meant slicing only fruit inside the view frustum and asserting the projected
  point is within `window.width() x window.height()`, not merely that a popup
  entity exists. Assert the property the feature promises (on-screen), not a
  proxy (entity count).

## Action items

- [x] Floating popup shipped, reviewed to APPROVE, merged.
- [ ] Carried into task C (20260703-132214): reuse `spawn_floating_text` for
  escalating "+N" and the "COMBO xN" banner; may revisit popup anchoring
  (R1.1) once wider combo text exists.
