# Retro: live combo HUD indicator

- TASK: 20260703-140247 (merged, 1 round, APPROVE)

## What went well
- The soft dependency on the time-window timer paid off: fading the HUD alpha
  by timer/COMBO_WINDOW makes the combo visibly cool down, which only works
  because the earlier task added the timer. Sequencing the tasks that way was
  the right call.

## Improve next time
- Small cosmetic HUD; nothing notable.
