# Review: fix tween despawn-race crash

- VERDICT: APPROVE
- ROUNDS: 1

## Verified

- `cargo fmt --check`, `cargo clippy --all-targets` (clean), `scripts/check-ascii.sh`.
- `cargo test`: 105 lib tests + 54 doctests pass. `cargo test --example 14_breach`: 22 pass.
- Regression test proven BOTH directions: reverting the fix, the two new tests FAIL with
  the exact production panic ("Entity despawned: The entity with ID ... is invalid");
  with the fix they pass. So the test really reproduces the P100 crash.
- Sustained-combat integration check: a breach autopilot run with the Playing hold
  extended to 25s (many kills across waves; reverted, not committed) completed with no
  "Entity despawned" and no panic.

## Findings

- The fix is the crate's own established idiom: `try_insert`/`try_remove`/`try_despawn`
  (already used in camera/chase and feedback/flash for the same "entity may be gone"
  reason). One generic edit in `advance_tween` covers all four registered value types
  (f32/Vec2/Vec3/Vec4).
- The regression test needed `auto_insert_apply_deferred: false` on the test schedule.
  Without it, Bevy sync-points the despawner (ordered `.before` Advance) so the despawn
  applies BEFORE advance_tween's query runs -- advance_tween then never sees the entity
  and never queues the completion command, so the race never occurs and the test passes
  even against the buggy code (a false green). Disabling auto-insert forces the despawn
  and the completion command into one end-of-schedule flush, in order, which is the real
  cross-flush race breach hit. This subtlety is documented in the test helper.
- Call-site orderings that worked around this (breach flash-before-damage, glide tween
  ordering) are now belt-and-suspenders; left untouched to avoid churn.

## Nits

- None.
