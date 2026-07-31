# Review: KISS pass: mesh/ + meth/ + camera/

- TASK: 20260731-172224
- BRANCH: refactor/kiss-mesh-meth-camera

## Round 1

- REVIEWER: out-of-context
- VERDICT: APPROVE

No BLOCKER or MAJOR. All six findings are record accuracy or tidiness and were
fixed in this round; each Response records the fix.

- [x] R1.1 (MINOR) tasks/20260731-172224/NOTES.md:129 - the heading
  `### camera/shake.rs (20)` disagrees with its own 15-row table and makes the
  per-file headings sum to 62 against the `57 blocks` total. 20 came from
  TASK.md's "33 comments", which counts comment LINES, a different unit.
  - Response: fixed, heading is now `(15)`. Re-derived independently:
    `check-comment-tags.sh src/mesh src/meth src/camera` on master reports
    9/12/6/6/15/5/3/1 = 57. The per-file baseline is now written into
    NOTES.md's Baseline section with the block-vs-line unit stated, so the
    number is checkable rather than asserted.

- [x] R1.2 (MINOR) tasks/20260731-172224/NOTES.md:73 - "3 of 7 moved to
  `slice.rs`"; `builder.rs` on master has 6 `#[test]` fns, not 7.
  - Response: fixed to "3 of 6". Re-derived: `grep -c '#\[test\]'` on
    master's `src/mesh/builder.rs` is 6; 3 moved, 3 stayed.

- [x] R1.3 (MINOR) src/mesh/builder.rs:462 - eleven kept comments became `///`
  on `#[cfg(test)]` fns rather than tagged blocks. `check-comment-tags.sh`
  exempts rustdoc and rustdoc never renders items inside a `#[cfg(test)] mod`,
  so the route is invisible to both tools; it should be a recorded convention
  rather than an unstated one, since the remaining epic tasks will copy it.
  - Response: agreed and sanctioned explicitly. The pattern is not new - 13
    `///`-on-`#[test]` comments already exist in landed code, including
    `mesh/explode.rs` itself and both files of the preceding epic task - but
    it was undocumented. AGENTS.md's comment-convention bullet now carries a
    sub-bullet drawing the line: what the TEST proves goes in `///` on the fn;
    what guards a VALUE inside the body stays a tagged `NOTE:`; and the `///`
    form is never a way to move an untagged body comment out of the checker's
    reach. NOTES.md records the same decision with its precedent.

- [x] R1.4 (NIT) tasks/20260731-172224/TASK.md close-out - "Public API is
  byte-identical: `slice` is private" reads as a claim about the public
  `TriangleMeshBuilder::slice` method.
  - Response: reworded in both TASK.md and NOTES.md to say the `slice` MODULE
    is private and the METHOD is untouched.

- [x] R1.5 (NIT) src/mesh/slice.rs:17 - `edge_plane_intersection` was
  `pub(super)` although its only caller, `triangle_slice`, moved with it into
  the same file.
  - Response: fixed, it is plain private again. Only `triangle_slice` and
    `TriangleSliceResult` are `pub(super)`, which is exactly what `builder.rs`
    imports.

- [x] R1.6 (NIT) src/mesh/explode.rs:307 - the dropped "same component shape
  the example's target uses" was the one dropped comment pointing outside the
  file.
  - Response: folded into the test's existing `///` line, which already named
    `examples/05_explode.rs`.

Process signal: `check-comment-tags.sh` cannot see comments promoted to `///`
inside a `#[cfg(test)]` module. That is by design for genuine test intent, but
it is also a silent way to satisfy the checker without changing anything, and
the reviewer found it unprompted. Worth one ledger line for the two remaining
epic clusters.

Verified in-session after the fixes: `cargo fmt --check` exit 0 (fmt collapsed
the now-private fn signature back to one line, and the checker was re-run
after fmt per the sibling task's lesson); `cargo clippy --all-targets` exit 0;
`check-comment-tags.sh src/mesh src/meth src/camera` exit 0; `check-ascii.sh`
exit 0. Load-bearing claims re-derived independently rather than accepted: the
per-file 57-block breakdown and the 6-test count above.

Pending user checks: none. The `manual:` DoD items (NOTES.md completeness,
public API unchanged) were both checked against the diff by the reviewer and
in-session.
