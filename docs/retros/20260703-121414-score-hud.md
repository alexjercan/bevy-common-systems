# Retro: Fruit ninja on-screen score UI

- TASK: 20260703-121414
- BRANCH: feature/fruitninja-score-ui (merged to master, deleted)
- REVIEW ROUNDS: 1 (APPROVE, one NIT fixed same round)

See TASK.md close-out for what shipped; this is process only.

## What went well

- Grepping the existing status-bar module for the Bevy 0.18 UI spawn shape
  (`Node`/`Text::new`/`TextFont`/`TextColor`) before writing meant the HUD
  compiled first try with no version-guessing. Reusing the repo as the API
  reference beat recalling Bevy UI from memory.
- Factoring `score_label(usize)` so the initial spawn text and the update
  system share one formatter removes a class of drift bug (HUD showing a
  different format than the initial value) for free.
- The 06-retro verification recipe carried over cleanly: a throwaway
  auto-slice boot with a temporary `eprintln` proved the HUD actually counts
  up on real hardware, which a compile check cannot show.

## What went wrong

- Nothing blocking. The only finding was a stale doc comment ("Shown in the
  status bar") on the `Score` resource that I did not update when moving the
  score out of the status bar. Root cause: edited the display site but did
  not re-scan for prose that described the old behavior. Minor, caught in
  review, fixed same round.

## What to improve next time

- When moving a feature from one surface to another (status bar -> HUD), grep
  for references to the old surface ("status bar") and fix the docs in the
  same edit, not just the code. Cheap sweep, avoids a review round for prose.

## Action items

- [x] Score HUD shipped, reviewed to APPROVE, merged.
- [ ] Nothing to promote to AGENTS.md; one-off doc-staleness lesson stays here.
