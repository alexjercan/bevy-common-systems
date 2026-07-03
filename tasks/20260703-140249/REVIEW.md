# Review: menu title pulse

- TASK: 20260703-140249
- BRANCH: feature/ninja-pulse

## Round 1

- VERDICT: APPROVE

Self-review. Added the marker via a (screen_text, MenuTitle) tuple in the
children! macro; pulse animates TextColor alpha (not font size, which is fixed),
gated to Menu. Checks clean, boots no panic.
