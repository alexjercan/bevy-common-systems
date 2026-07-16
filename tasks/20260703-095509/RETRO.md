# Retro: Fix stale default event_info path in EventKind derive macro

- TASK: 20260703-095509
- BRANCH: fix/eventkind-default-path (merged to master, deleted)
- REVIEW ROUNDS: 2 (R1 APPROVE + 1 NIT, R2 verified the NIT fix)

See TASK.md close-out for the technical decision; this is process only.

## What went well

- Writing the regression test first turned a mis-scoped task into a correct
  fix. The task (and the earlier discovery) framed this as a one-line path
  change; compiling an actual attribute-less derive immediately surfaced
  the real blocker (`GameEventInfo` has no `Serialize` impl, so no path to
  it could satisfy `EventKind::Info`). The one-liner would have "looked
  done" and still not compiled.
- The fix was chosen against alternatives on merit (fully-qualified path,
  deriving Serialize on the wrapper, `()`), with the rejected options and
  their concrete failure modes written down, so a future reader sees why
  `()` and not just that it works.
- Applied the previous retro's lesson directly: doctests/generated code
  referencing crate internals need care about what actually resolves and
  what bounds must hold - here the bound, not the path, was the crux.

## What went wrong

- The task's Steps encoded an assumption ("change the default path") that
  was only half the bug. Root cause: the original discovery
  (in 20260703-094842) was a read-only observation - the macro was eyeballed,
  not compiled - so it recorded the visible path typo and missed the latent
  trait-bound failure behind it. A read-only "this looks broken" note is a
  hypothesis, not a diagnosis.

## What to improve next time

- When filing a bug from reading alone, label it as unverified and, if
  cheap, attach a minimal repro (here: a 3-line bare derive + `cargo build`)
  so the fixer starts from a real compiler error, not a guess. The planned
  Step can then be accurate instead of optimistic.

## Action items

- [x] EventKind trait doc now documents the derive defaults (R1.1), so the
  `()` default is discoverable, not folklore.
- [ ] No follow-up tasks; backlog now has only 20260703-101712
  (non-ASCII in chase.rs), which is next in this flow.
