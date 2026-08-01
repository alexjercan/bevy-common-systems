# Retro: check-comment-tags: flag a /// on a test fn guarding an unexplained literal

- TASK: 20260801-093730
- BRANCH: chore/check-comment-tags-doc-rule
- REVIEW ROUNDS: 2

## What went well

Measuring three candidate rule definitions over the real tree before writing
any of them. The DoD's own definition read 183 hits in `src` and was
unshippable; the shipped correlation rule reads 8. That is not a judgement call
prose could have settled, and running the candidates cost minutes.

Keeping a canary -- a textbook violation injected into `src/modding/registry.rs`
-- as the standing question rather than declaring victory on the first fix. It
stayed green through three fixes and surfaced a new defect each time.

## What went wrong

Round 1 found the region parser reporting files clean that it had never read.
Four independent causes of one class: a lone lifetime tick read as an opening
quote, a brace in comment prose, a brace in a raw string, and a raw string
spanning lines. Plus a `#[cfg(test)]` latch that marked production code as a
test region and reported public rustdoc.

The failed decision, and why it looked sound: hand-rolling a brace-depth
scanner in awk to bound the test region. The alternative -- "everything after
`#[cfg(test)]` to end of file" -- has an obvious false positive on public docs
below the test module, and the tree to be parsed is rustfmt-normalised. What
that reasoning missed is that every failure mode of a depth counter is SILENT.
A miscounted brace closes the region early and the checker reports a clean run;
it never produces a wrong finding anyone would notice. So the class was
invisible to reading, to the fixture suite as first written, and to a clean run
over `src`.

Two of the four were found by accident: writing a fixture for the char-literal
case tripped the comment-prose bug, and the canary tripped both raw-string
ones. Accident is not a strategy, which is why the response was not four fixes
but a balance guard -- a `.rs` file that does not end at depth 0 is now a tool
error at exit 2, so the fifth cause cannot hide the way the first four did.

Smaller: the plan's DoD widened the gate to `examples/`, which rule 1 has never
covered (769 untagged blocks). Caught during work and corrected in the record
rather than absorbed as scope.

## What to improve next time

Breadth: 202 lines of script plus 269 of fixtures and probes for a one-rule
change. It did not grow from missed splits -- half of it is the probe suite the
rule needs in order to be trustworthy, and that is the right half to have. No
independently landable piece was missed.

Churn: the plan-time question that would have prevented round 1 is not the
from-scratch challenge -- the design was right -- but a missing one: **when the
deliverable's failure mode is silent, what makes it loud?** The plan specified
the rule and its probes and never asked how the checker announces that it has
lost the plot. That question produces the balance guard on day one, and the
balance guard alone catches all four desyncs.

Context: no pressure observed. No compaction warning, no handoff, no
delegation.

## Action items

- None carried forward. The balance guard, the four fixture lines and the CI
  step all landed here; the one-line-body limitation is recorded in NOTES.md as
  unreachable under `cargo fmt --check`.
