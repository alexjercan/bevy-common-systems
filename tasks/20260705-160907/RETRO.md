# Retro: tween despawn-race crash fix (P100)

- TASK: 20260705-155230 (fixes the bug report 20260705-151725)
- BRANCH: fix/tween-despawn-race (squash-merged to master as f1a3611)
- REVIEW ROUNDS: 1 (APPROVE)

## What went well

- The backtrace pinned it immediately: frame 22 was `tween::advance_tween::<f32>`'s
  apply_deferred, frame 5 the `insert::<TweenFinished>` command failing "Entity
  despawned". No repro-hunting needed to locate the cause -- read the stack, found the
  three `commands.entity(entity).insert/remove/despawn` calls, knew the fix.
- The fix was the crate's OWN established idiom: `try_insert`/`try_remove`/`try_despawn`,
  already used in camera/chase and feedback/flash for exactly "the entity may be gone".
  One generic edit in `advance_tween` fixed all four value types (f32/Vec2/Vec3/Vec4) and
  every example that tweens short-lived entities -- a crate fix, not a breach patch.
- Insisted on a regression test that actually fails on the old code, and verified BOTH
  directions (revert -> two tests panic with the exact production error; restore -> pass).
  That is the difference between a test and a decoration.
- Belt-and-suspenders left alone: the call-site orderings (breach flash-before-damage,
  glide tween ordering) that previously worked around this stay; the crate fix makes them
  redundant but removing them would be churn with no benefit.

## What went wrong

- The first regression test was a FALSE GREEN: a despawner ordered `.before(Advance)`
  passed even against the buggy code. Bevy's `auto_insert_apply_deferred` (default on)
  inserts a sync point for the ordering, so the despawn applied BEFORE advance_tween's
  query ran -- advance_tween never saw the entity, never queued the completion command,
  so the race never happened. Caught it by doing the revert-check (the test should have
  failed on the buggy code and didn't). Fix: disable `auto_insert_apply_deferred` on the
  test schedule so the despawn and the completion command land in one end-of-schedule
  flush, in order -- the real cross-flush race. Lesson: a despawn-race test must control
  sync points, and ALWAYS confirm a regression test fails without the fix.

## What to improve next time

- For any "command applied to a stale entity" panic, reach straight for the `try_*`
  entity commands; the plain ones panic and are almost never what you want for
  fire-and-forget effects on entities other code can despawn.
- When writing a test that depends on cross-system command-application order, remember
  auto-inserted sync points will reorder things; disable `auto_insert_apply_deferred` (or
  add explicit `ApplyDeferred`) to make the timing deterministic, and prove the test
  fails on the unfixed code before trusting it.

## Action items

- None. The fix is crate-wide; no follow-ups. (Two breach items remain open from earlier
  flows: the game-over no-camera follow-up 20260705-154058.)
