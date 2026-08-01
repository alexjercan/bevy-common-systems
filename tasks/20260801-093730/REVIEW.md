# Review: check-comment-tags: flag a /// on a test fn guarding an unexplained literal

- TASK: 20260801-093730
- BRANCH: chore/check-comment-tags-doc-rule

Reviewed at 54fe465 (round 1) and a85d80f (round 2), against `master` d47b88b.

## Round 1

- REVIEWER: in-session primary
- VERDICT: REQUEST_CHANGES

The out-of-context round-1 default was NOT used: this session carries a
standing instruction not to spawn subagents unless the user asks. Compensating
measure: every finding below is backed by an executed probe against a
checked-in file, not by reading the awk.

### BLOCKER - a single lifetime tick silently blanks the rest of its line, desyncing brace depth

`scripts/check-comment-tags.sh:86` (`blank()`)

`blank()` treats every `'` as opening a char literal. A Rust LIFETIME is a lone
`'`, so a line carrying an odd number of ticks has everything after the first
one deleted -- including its `{`. Brace depth then under-counts, the test
region is judged closed early, and every test fn below it is never examined.

The rule fails SILENT: exit 0, "no test fn's /// guards a literal".

This is not hypothetical on this tree. `src/modding/registry.rs:340` is
`fn name() -> &'static str {` inside the `#[cfg(test)] mod tests` that opens at
line 320 -- one tick, one swallowed brace.

Failure scenario, executed:

```
cp src/modding/registry.rs /tmp/canary.rs
# append to the test module, before its closing brace:
#     /// A canary documenting 4.75 that rule 2 must report.
#     #[test]
#     fn canary_must_be_seen() {
#         assert!(compute() < 4.75);
#     }
./scripts/check-comment-tags.sh /tmp/canary.rs
-> check-comment-tags: ... no test fn's /// guards a literal
-> exit 0
```

A textbook violation, in a real file this checker gates, reported clean.

Minimal reproduction (`/tmp/probe6.rs`): a `struct Holder<'a> {` before a
documented test fn hides that fn entirely; delete the struct and the same fn is
reported.

Change: stop treating a lifetime tick as a quote. A char literal is
`'` + (escape or one char) + `'`; a lifetime is `'` + identifier with no closing
tick. Blank only the former. Both fixtures must gain a lifetime line, or this
regresses unobserved -- the current fixture set has none, which is why the
suite passes.

### MAJOR - `#[cfg(test)]` on a brace-less item latches, so public rustdoc is read as test doc

`scripts/check-comment-tags.sh` (the `pending` latch)

`pending` is set by the `#[cfg(test)]` attribute and cleared only by the next
line that opens a brace. Applied to a brace-less item -- `#[cfg(test)] use ...;`
is the common Rust form -- the latch survives to whatever opens a brace next,
arbitrarily far down the file, and marks production code as a test region.

Rule 2 then reads `///` on public items, which the script header and the
AGENTS.md bullet both promise it never does.

Failure scenario, executed (`/tmp/probe4.rs`):

```
#[cfg(test)]
use std::fmt;

mod real_code {
    /// The public contract: values settle under 0.35.
    pub fn settle(x: f32) -> bool {
        x < 0.35
    }
}
-> probe4.rs:7: 0.35 --         x < 0.35
-> exit 1
```

`settle` is public API and its doc is rendered by rustdoc; it is exempt by
design and is reported anyway.

Change: clear `pending` on any line that is neither an attribute, a comment,
nor blank, so it only bridges the attribute stack directly above an item. Add
the shape to the compliant fixture.

### MINOR - a `#[cfg(test)] mod t { ... }` opened on the attribute's own line is skipped

`scripts/check-comment-tags.sh` (same latch)

When the attribute line ALSO opens the brace, the `if` arm sets `pending` and
the `else if` that would enter the region never runs, so the whole module is
invisible. Nothing in the tree is written that way today, so this is a latent
false negative rather than a live one. Worth closing with the MAJOR fix, since
it is the same branch.

## Verified and correct

- Rule 2's definition and its three narrowings match `NOTES.md`, and the
  measurements re-run independently: `check-comment-tags.sh src
  bevy_common_systems_macros/src` gives 8 hits on `master` and exit 0 on the
  branch; dropping the `0`/`1`/`2` clause gives 14, and all 6 of the difference
  are false positives.
- Rule 2 reads 0 hits under `examples/`, so scoping the gate to
  `src bevy_common_systems_macros/src` loses nothing. The Close-out states the
  DoD correction plainly instead of widening scope quietly. Good.
- The six fixed test fns each keep an intent-only `///` and gain a body
  `NOTE:`; none had its `///` deleted.
- The self-test is genuinely red without the rule (rule 2's reporting branch
  disabled -> 2 probes fail), and the exit-2 probes correctly separate tool
  error from hits.
- `#[cfg(all(test, not(target_arch = "wasm32")))]` matches, nested modules
  match, multi-line fn signatures match (probes 2, 3).
- fmt, clippy in both configurations, `cargo test` and `cargo test --features
  debug`, `check-ascii.sh` and `actionlint` all re-run clean on the branch.

## Pending manual items

- The AGENTS.md bullet reads correctly and names both the case and the
  enforcing checker.
- The `git diff master` inspection of the six fixed test fns.

Neither blocks; both were checked and hold.

### Round 1 verdict

REQUEST_CHANGES. One BLOCKER (silent false negative on a real file in the gated
tree), one MAJOR (false positive on public rustdoc), one MINOR sharing the
MAJOR's branch. Both fixes need fixture coverage in the same change, or the
probe suite keeps passing over them.

## Round 2

- REVIEWER: in-session primary
- VERDICT: APPROVE

Same recorded exception as round 1.

### Round 1 findings, all confirmed fixed

| Finding | Verification |
| --- | --- |
| BLOCKER, lifetime tick desync | the canary (a violation injected into `src/modding/registry.rs`'s real test module) now exits 1 and names the literal; it exited 0 at round 1 |
| MAJOR, `#[cfg(test)]` latch onto public code | the brace-less `use` probe now exits 0; it reported `measure_wobble`'s public rustdoc at round 1 |
| MINOR, one-line `#[cfg(test)] mod t {` | enters the region, as intended |

The BLOCKER fix went further than the finding asked, correctly. Fixing the
stated cause left the canary green, and chasing it surfaced three more
brace-depth desyncs -- comment prose, raw strings, multi-line raw strings.
Every one was silent. The response is the right one structurally: an
unbalanced file is now a TOOL error at exit 2 rather than a clean run, so the
fifth cause of this class cannot hide the way the first four did.

### Verified this round

- `test-check-comment-tags.sh` exits 0, and is red when any of the lifetime
  fix, the raw-string cross-line state, or the balance guard is reverted --
  re-derived independently by patching each and re-running.
- No desync is reported anywhere in `src`, the macros crate or `examples`.
- Re-probed for fix regressions: nested closures and `match` blocks, escaped
  quotes and a brace inside a plain string, one-line module headers. All
  behave.
- `check-comment-tags.sh src bevy_common_systems_macros/src` exits 0;
  `check-ascii.sh` exits 0.
- No Rust source changed this round (`git diff --name-only 4e06cc6 -- '*.rs'
  ':!scripts'` is empty), so round 1's cargo results stand. Re-checked rather
  than assumed.

### Accepted, not a finding

A fn whose whole body is on the `fn` line is unscanned. `cargo fmt --check` is
a CI gate and rustfmt splits that form -- verified by running rustfmt over it,
and the formatted shape is caught. Recorded in `NOTES.md`. Coding around a
formatter guarantee would add a branch for no reachable case.

### Pending manual items

Both round-1 manual items were checked and hold. Neither blocks.

### Round 2 verdict

VERDICT: APPROVE
