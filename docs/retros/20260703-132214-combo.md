# Retro: Fruit ninja combo scoring and combo text

- TASK: 20260703-132214
- BRANCH: feature/fruitninja-combo (merged to master, deleted)
- REVIEW ROUNDS: 1 (APPROVE, one NIT + one observation, both accepted)

See TASK.md close-out for what shipped; this is process only.

## What went well

- Applied the blade-trail retro's lesson before review could catch it: added
  the combo reset to `start_game` in the same change that introduced the
  `Combo` resource, alongside the release-branch reset. The reset-omission
  pattern that showed up twice before did not recur - the compounding worked.
- Turned the headless-verification problem into a real test instead of a weak
  one. A held-swipe over fruit cannot be faked in a headless boot (mouse press
  and cursor position are both OS-driven), so rather than a boot-that-cannot-
  fail, I extracted the combo math into a pure `advance_combo` that
  `slice_objects` actually calls, and unit-tested that. The tests exercise the
  real code path, not a copy - the `segment_hits_circle` lesson from the first
  example, reused deliberately when the integration test was out of reach.
- Reusing `spawn_floating_text` (built in the previous task) meant the combo
  banner and escalating "+N" were a few lines, not a new subsystem. Building
  the shared primitive one task earlier paid off exactly as planned.

## What went wrong

- Nothing blocking. The one judgment call left open: a "COMBO xN" banner spawns
  per fruit past the first, so a long swipe stacks several banners. I flagged it
  in my own review as a possible clutter NIT but left it, since the request
  explicitly asked for "cool looking text" and climbing banners read as
  escalation. Recording it so the choice is visible, not silent.

## What to improve next time

- When integration testing is genuinely impossible (OS-driven input here),
  reach for "extract the pure core and unit-test it" earlier, instead of
  spending boots on a harness that can only prove "did not panic". Two of the
  three UX tasks this flow needed that move; recognizing it up front would have
  saved a couple of throwaway-boot iterations.

## Action items

- [x] Combo scoring + text shipped, reviewed to APPROVE, merged. Goal complete.
- [ ] Optional follow-ups if the user wants them: time-window combos instead of
  hold-based; single combo banner (highest count) instead of stacking; centered
  popup anchoring (popup retro R1.1). None filed - cosmetic/preference.
