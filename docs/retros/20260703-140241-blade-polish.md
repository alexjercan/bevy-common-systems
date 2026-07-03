# Retro: thicker gradient blade trail

- TASK: 20260703-140241 (merged, 1 round, APPROVE)

## What went well
- Guarded the perpendicular against zero-length segments up front, avoiding a
  NaN-from-normalize crash that a held-still cursor (repeated points) would hit.

## Improve next time
- Cosmetic gizmo tweak; nothing notable.
