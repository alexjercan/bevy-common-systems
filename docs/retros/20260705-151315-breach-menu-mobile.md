# Retro: 14_breach menu + mobile polish

- TASK: 20260705-132207
- BRANCH: feat/breach-menu (squash-merged to master as df4dbd8)
- REVIEW ROUNDS: 1 (APPROVE)

Final task of the breach fun-pass flow (spike
`docs/spikes/20260705-132024-breach-fun-pass.md`).

## What went well

- Reading the existing menu first kept the scope honest: the spike had assumed "add a
  controls hint", but the desktop hint / pulsing title / best readout already existed, so
  the task became a touch hint line + tap-aware prompt + the one genuinely valuable
  mobile change (aim-assist). Not re-building what was there is the whole point of the
  spike's "assess vs the ask" framing.
- Aim-assist landed cleanly because the crate `DoomController` already exposes a settable
  `state.yaw` and `Drive` integrates look incrementally (`yaw -= look*sens`). Running the
  nudge BEFORE Drive means the player's own look composes on top for free -- no fighting
  the controller for the transform.
- Gating on `TouchInput.fire` makes the feature provably desktop-safe (a mouse never sets
  it), and extracting the shortest-arc `step_angle_toward` made the one bug-prone bit
  (angle wraparound across +/-pi) a pure unit test instead of a feel-it-on-a-phone gamble.

## What went wrong

- Nothing broke. The honest limitation: aim-assist can only be *felt* on a real touch
  device; headlessly I could only prove it is structurally safe (gated, Single query,
  early-return) and that its math is correct. Said so in the review rather than implying
  it was play-verified.

## What to improve next time

- When a "polish" task's spike bullet turns out already-done, say so and redirect the
  budget to the highest-value adjacent gap (here, aim-assist) instead of padding the
  done thing. The review's "deliberately left as-is" section (fire-button sizing, whose
  visual and hit-zone constants are separately tuned and easy to desync) is the honest
  way to record a skip.
- Touch-only features that alter a shared control (here, yaw) should be gated on a
  touch-only signal, not a mode flag, so desktop is untouched by construction.

## Action items

- None. Fire-button/stick sizing left as-is (usable); revisit only with a real device to
  tune the button and its `read_touch` fire zone together.
