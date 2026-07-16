# Retro: Fruit ninja blade trail

- TASK: 20260703-132207
- BRANCH: feature/fruitninja-blade-trail (merged to master, deleted)
- REVIEW ROUNDS: 1 (APPROVE, R1.1 MINOR + R1.2 NIT both fixed in-round)

See TASK.md close-out for what shipped; this is process only.

## What went well

- Applied the previous retro's lesson immediately: created the feature branch
  in the same command that flipped STATUS to IN_PROGRESS, before touching any
  code. The exact slip from last task did not recur.
- Confirmed `Gizmos` availability against the installed source
  (`GizmoPlugin` in `bevy_internal` default_plugins) before choosing the
  immediate-mode approach, so the plan committed to a design I had verified was
  present - no mid-work surprise.
- Chose gizmos over spawning trail entities: no assets, no per-point entity
  churn, no cleanup beyond clearing a deque. The right tool for a transient
  visual.

## What went wrong

- My first verification boot was a false positive in the making: I seeded the
  trail from a temp system while `slice_objects` was still running its
  release-branch `blade.points.clear()` every frame (LMB not pressed in a
  headless boot). The deque oscillated 0/1, so `draw_blade_trail`'s `count < 2`
  guard skipped every frame and the gizmo path never actually executed - yet
  the boot "passed" (no panic) and printed a misleading `points=1`. I caught it
  only because I logged the point count and it was 1, not ~16. Root cause: I
  verified a system whose input was being clobbered by another system I forgot
  ran unconditionally in the same state.
- Reviewing my own code surfaced R1.1: `start_game` reset every per-run
  resource except `BladeTrail`, so a swipe interrupted by game-over could flash
  a stale trail next run. I wrote the reset for the resources I was thinking
  about (score/timer/cursor) and missed the one I had just added.

## What to improve next time

- When a headless verification depends on resource/component state, account for
  every system that writes that state in the current schedule before trusting
  the result. Here the tell was a logged invariant (point count) not matching
  the expected value; log a concrete expected number and assert against it, do
  not just check "no panic". A boot that cannot fail is not a test.
- When adding a new piece of per-run state, grep the reset/`start_game` path in
  the same change and add the reset there. New long-lived state and its reset
  should land together, the same way new display-surfaces need their old-copy
  cleared (score-HUD retro lesson).

## Action items

- [x] Blade trail shipped, reviewed to APPROVE, merged.
- [ ] Pattern worth watching: "new per-run/long-lived state -> reset it in
  start_game in the same commit". Third time a reset-omission shows up
  (score-HUD staleness, this one) it graduates to an AGENTS.md note.
