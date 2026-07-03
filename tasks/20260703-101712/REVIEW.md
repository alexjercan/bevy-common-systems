# Review: Replace non-ASCII typographic chars in src/camera/chase.rs docs

- TASK: 20260703-101712
- BRANCH: fix/chase-ascii

## Round 1

- VERDICT: APPROVE

Verified independently:

- The diff is exactly the intended 5 replacements - 4 en-dashes (U+2013)
  to `-` in the combining-list doc (lines 65-68) and 1 curly apostrophe
  (U+2019) to `'` on line 107 - and nothing else. Column alignment in the
  list is preserved.
- Whole-tree scan `grep -rnP '[^\x00-\x7F]'` over src/,
  bevy_common_systems_macros/src/ and examples/ returns nothing, so the
  Goal (no non-ASCII in the source tree) is met, not just chase.rs.
- `cargo fmt --check` clean and `cargo test --doc` passes (11), so the doc
  comments still render and compile.
- Pure text change, no code touched; TASK.md close-out matches the diff.

No findings. Trivial, correct, single-purpose diff.
