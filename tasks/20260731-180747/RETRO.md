# Retro: Fix clippy manual_contains in completion.rs

- TASK: 20260731-180747
- BRANCH: master (no feature branch)
- REVIEW ROUNDS: 1

## What went well

Cost of the whole task was one `git log -S'contains(&name)'` plus a suite run.
Checking whether the defect still exists BEFORE sprouting a worktree meant no
branch, no diff, no merge.

## What went wrong

The task was filed specifically to keep a lint fix OUT of task 20260731-172208's
comment-hygiene diff -- and 20260731-172208 folded it in anyway (`c0f67c5`,
whose message even lists "fix a pre-existing manual_contains lint in
completion.rs"). Splitting a task is only a real split if the sibling session
honours the boundary; filing the split does not enforce it.

The task also carried a speculative warning ("check `contains(&name)` actually
compiles, `&str` vs `&'static str`") that was never true. Deref coercion
handles it. Speculation written into a task record reads as a known hazard to
whoever picks it up later.

## What to improve next time

- A task whose whole body is a quoted tool diagnostic at a named line should
  re-run its own `cmd:` proof as step zero, before any planning. Cheapest
  possible check for "is this still real".
- When a task is filed to carve scope OUT of an in-flight sibling, say so in
  the SIBLING's record too, not only in the new one. One-way notes do not bind.
- Do not record a compile concern in a task unless it has been observed. State
  it as a question ("does this coerce?") or leave it out.

## Action items

- [x] Verify the four `cmd:` proofs on master (all pass; NOTES.md).
- [ ] None outstanding.
