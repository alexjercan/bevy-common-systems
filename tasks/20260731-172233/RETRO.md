# Retro: KISS pass: feedback/ tween/ ui/ transform/ + small modules

- TASK: 20260731-172233
- BRANCH: refactor/kiss-feedback-tween-ui-transform
- REVIEW ROUNDS: 2 (round 1 REQUEST_CHANGES, round 2 APPROVE; out-of-context both times)

## What went well

The intent-versus-value-guard split, which the previous cluster's review
invented and this one inherited, held up as a working rule across 106 blocks:
37 kept as tagged `NOTE:` in the body where they guard a number, the rest
dropped or promoted to `///` on the item whose behaviour they describe. It was
applied honestly enough that the reviewer found exactly one slip in the whole
diff (`ui/health_display.rs`) rather than a pattern.

`record-numbers-from-a-rerun` paid before the pass began: the 106-block
baseline was re-run on the work branch, the unit (blocks, not lines) was named
explicitly, and the per-file breakdown was pasted. When the round-1 reviewer
re-derived the baseline independently via `git archive master` into a clean
tree, it reproduced line for line, so the one number nobody could check
afterwards was never in question.

The `ui/status.rs` performance contract was treated as an instruction to
verify, not to preserve. Verifying it found the real defect: the contract was
true, but lived only in a comment copied from Bevy's own docs and was missing
from the module doc a caller actually reads. The same check refuted a sentence
of my own -- a first draft claimed `color_fn` also takes `&World`, and reading
the signature showed it takes the produced value and runs in an ordinary
parallel system. Ten seconds of grep against a claim written from a reading I
had just done.

## What went wrong

**The scope was phrased as a complement and then implemented as a list.** The
Story says "everything not claimed by the other children" and, in the same
breath, enumerates ten directories. I planned, measured and gated against the
enumeration. `src/material.rs` is a bare file at the top of `src/` that no
child claims; it fell through, and because this was the LAST child there was
nobody downstream to catch it. The epic-wide gate
`./scripts/check-comment-tags.sh src bevy_common_systems_macros/src` would have
exited 1 after the epic closed -- the epic failing its own deliverable by one
block.

It seemed sound at the time because the enumeration was concrete, testable and
authored by the same planning pass that wrote the complement; treating the list
as the operative definition felt like reading the Story precisely. The
complement was never computed against the tree even once. Computing it is one
command, and that command was already in the repository: running the checker
over ALL of `src` and diffing against my ten directories would have printed
`material.rs` on day one.

**A number went stale between measurement and record.** `ui/status.rs` went in
as 326 lines, measured before the final module-doc edit and never re-derived;
it was 328 at commit. Same failure mode the ledger already names, in the same
task where the guard otherwise worked -- the guard was applied to the baseline
(the number I expected to be challenged) and not to a downstream number I
treated as bookkeeping.

## What to improve next time

- A scope stated as a complement ("everything not claimed by X") is a query,
  not a list. Run the query against the tree once, print the result, and make
  THAT the enumeration -- before writing any DoD that names paths.
- When a project ships a checker, the last task in a multi-task cleanup should
  run it unscoped at least once. Per-task path scoping is what lets a gap hide
  from every individual task while remaining visible to the epic.
- Re-derive every number in a record at write time, not just the headline one.
  The stale 326 survived precisely because it did not feel load-bearing.

## Action items

- None requiring a new task. The DoD now carries the epic-wide checker
  invocation alongside the scoped one, so this specific gap cannot recur by
  scoping; the general lessons are in `LESSONS.md`.
