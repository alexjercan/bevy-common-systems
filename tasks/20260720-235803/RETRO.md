# Retro: tick 094942's dropped steps

## What went well

- Correct disposition: 094942 is a real flow task with proper records, so the
  historical tag would have been a wrong label. Flipping the already-annotated
  "(dropped, ...)" boxes to [x] is the honest fix - drop reasons kept verbatim,
  only the checkbox state changed.
- `tatr check` on bevy is now fully clean.

## What went wrong

- Nothing. Trivial 3-line diff, in-session review.

## What to improve next time

- Two disposition classes for closed-unchecked: FROZEN pre-flow tasks -> historical
  tag (tatr exemption); COMPLETED tasks with dropped steps -> tick the dropped
  boxes (reason inline). Don't reach for the tag when the task has real records.

## Action items

- [x] 3 boxes ticked; bevy tatr check clean; landed a4846cd.
