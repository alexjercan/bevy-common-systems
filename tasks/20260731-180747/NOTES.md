# NOTES - 20260731-180747

## Outcome: no code change needed; already fixed

The lint was fixed before this task was picked up. `c0f67c5`
("refactor(kiss): comment pass + split integrity damage out of plugin",
task 20260731-172208) carries:

```
-        self.pending.iter().any(|p| *p == name)
+        self.pending.contains(&name)
```

The task record predicted the opposite -- it was filed precisely so a lint fix
would NOT be folded into the comment-hygiene diff. It got folded in anyway.
The `&str` / `&'static str` concern the task flagged was a non-issue:
`Vec<&'static str>::contains(&&str)` coerces fine.

`others_pending` (`src/completion.rs:95`) uses `any(|p| *p != name)`, which is
not a `contains` pattern and needs no change. Step 2 checked, nothing to do.

## Verification (on master, 4d44397)

| Proof | Result |
| --- | --- |
| `cargo clippy --all-targets` | exit 0, 0 `manual_contains` |
| `cargo clippy --all-targets --features debug` | exit 0, 0 `manual_contains` |
| `cargo test` | exit 0 (59+148 passed) |
| `cargo test --features debug` | exit 0 (66+155 passed) |

Only the expected `proc-macro-error2` future-incompat note.

## Next time

A task that names a specific lint at a specific line should re-run its `cmd:`
proof as step zero. This one was verified stale in one grep of `git log -S`;
the whole cost was the suite run.
