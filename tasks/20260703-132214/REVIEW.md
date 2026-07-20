# Review: Fruit ninja combo scoring and combo text

- TASK: 20260703-132214
- BRANCH: feature/fruitninja-combo

## Round 1

- VERDICT: APPROVE

Combo scoring is correct and, notably, unit-tested through the real code path:
`advance_combo` is the exact function `slice_objects` calls, and two tests
(`combo_escalates_within_a_swipe`, `combo_resets_between_swipes`) lock the
1,2,3 escalation and the release reset. The combo resets in both the
not-pressed branch and `start_game` - the "reset new state in start_game in the
same change" lesson from the blade-trail retro was applied proactively, not
caught in review. Bombs are handled in the `is_bomb` arm so they do not
increment the combo (correct: a bomb ends the run). Popup and banner reuse
`spawn_floating_text`. Checks clean (`fmt`, `clippy --all-targets` both configs,
`check-ascii`, 8 example tests pass); boots with no panic. Module doc and
AGENTS.md updated.

- [ ] R1.1 (NIT) examples/06_fruitninja.rs:slice_objects fruit arm - a "COMBO
  xN" banner is spawned for every fruit past the first, so a fast 5-fruit swipe
  stacks x2/x3/x4/x5 banners at once. This reads as an escalating combo (the
  intended flashiness), but if it looks cluttered in play, show the banner only
  for the highest count reached this frame, or replace a prior banner. Left as
  the flashier option per the request ("cool looking text").
  - Response: Left as-is - the climbing x2/x3/x4 banners are the "cool looking
    text" the request asked for; revisit only if it reads as clutter in play.

- [ ] R1.2 (NIT) Observation, not a defect - by design a combo spans one continuous
  LMB hold, so a player who holds and keeps sweeping can chain a long combo
  without releasing. This matches the fruit-ninja feel and is documented in the
  code and TASK.md; noting it so it is a known choice, not an oversight. A
  time-window combo would be a follow-up if desired.
  - Response: Acknowledged - documented design choice, no change.
