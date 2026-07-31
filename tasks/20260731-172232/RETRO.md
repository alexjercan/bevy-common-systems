# Retro: KISS pass: modding/ + persist/ + macros subcrate

- TASK: 20260731-172232
- BRANCH: refactor/kiss-modding-persist-macros
- REVIEW ROUNDS: 1 (APPROVE, out-of-context reviewer)

## What went well

The split criterion inherited from the previous two clusters held up as a
criterion rather than a habit: applied honestly to `modding/events.rs` -- the
crate's largest remaining code body and the file the epic's Fog named -- it
returned KEEP. Recording the reasoning (four layers with non-disjoint
dependency sets, no cut that reduces any resulting file's imports) makes the
negative result reusable instead of leaving the next reader to re-open the
question.

Treating "verify this guidance" as a real instruction paid. The task said to
check the macros `EventKind` rustdoc rather than trim it; the check found the
AGENTS.md gotcha had been wrong since the derive was fixed, and the wrongness
was invisible precisely because nothing in the repo exercised the path.

Review round 1 caught a measurement error I had already been burned by once
(`record-numbers-from-a-rerun`, written by the previous cluster) and a misuse
of the `///`-on-a-test-fn route that the previous cluster had just sanctioned.
Both were found by re-derivation, not by reading my record -- which is what the
out-of-context reviewer is for.

## What went wrong

**The measuring grep matched less than it claimed.** `persist/mod.rs` went into
the record as 200 total / 200 code. Its test module is
`#[cfg(all(test, not(target_arch = "wasm32")))]`, a form my `#[cfg(test)]` grep
does not match, so it reported no test module and I read the file as all code.
The number was 148. It seemed sound at the time because the grep had just
produced five other rows that re-derive exactly -- five right answers made the
sixth look like data rather than a matcher gap. Same shape as
`probe-a-new-checker-both-ways`: a clean run on real sources cannot reveal a
matcher that silently covers less than intended.

**The `///`-on-a-test-fn route got misused on its second outing.** The previous
cluster established it and wrote the line into AGENTS.md: `///` for what the
test PROVES, tagged `NOTE:` for what guards a value in the body. In
`persist/backend.rs` I promoted a *purity guard* ("kept pure so it never races
the env-based test") to `///` and never said what the test proves -- an untagged
body comment moved into the checker's blind spot, which is exactly what the
convention was written to prevent. The sibling hazard about the same env var, in
`persist/mod.rs`, stayed a correct `NOTE:`. Having authored the rule was not
enough to apply it.

**The new test shipped its own clippy warning.** The first version bound
`<OnQuiet as EventKind>::Info` with a `let`, which is `clippy::let_unit_value`.
Caught by the run rather than by review, and the replacement
(`fn requires_unit_payload<E: EventKind<Info = ()>>()`) is a stronger guard
anyway -- the original defect was a compile failure, so a compile-time bound
reproduces it where a runtime binding only gestured at it.

**The DoD initially miscounted the in-scope `cargo doc` warnings** as four when
two of them live in `helpers/`, cluster 20260731-172233's scope. Corrected
mid-task rather than quietly satisfied.

## Diagnosis

**Breadth.** The diff is small (6 files, +73/-59) and matches the plan. The one
unplanned addition -- a new test -- is inside scope, since the task explicitly
asked whether the macros guidance was still true and a claim about the derive's
default is only checkable by deriving.

**Churn.** No review rework in the implementation sense: zero BLOCKER/MAJOR, and
all five findings are record or convention accuracy. The plan-time question that
would have prevented R1.1 is not in `plan` at all but in the measuring step
itself -- the plan said "measure code-before-tests per file" and did not say
"and show the command handles every `cfg` form it will meet". That is a gap in
how a measurement Step is written, not in the design.

**Context.** No threshold crossing during implementation. The session compacted
once mid-verification; the cost was a stale empty log from an interrupted build,
re-run rather than trusted, and no record was lost because NOTES.md and the DoD
were already written down.

## What to improve next time

- A measuring command that feeds a record gets one adversarial case before its
  output is believed: construct or find the variant it might miss. Five correct
  rows are not evidence about the sixth.
- When a convention exists specifically to stop a thing, check my own diff
  against it explicitly before the review round, rather than assuming authorship
  confers compliance.
- Where a record claims a guard, prefer the compile-time form. The
  `EventKind<Info = ()>` bound cannot be satisfied by accident and needs no
  assertion message.

## Action items

- None requiring a new task. The remaining `cargo doc` warnings in `helpers/`
  and `input/` are already owned by 20260731-172233, the last epic child.
