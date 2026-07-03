# Replace non-ASCII typographic chars in src/camera/chase.rs docs

- STATUS: OPEN
- PRIORITY: 10
- TAGS: bug

## Goal

`src/camera/chase.rs` is the only source file containing non-ASCII
typographic characters (en-dashes `-` and a curly apostrophe), which
violates the repo-wide plain-ASCII writing rule documented in AGENTS.md.

## Steps

- [ ] In src/camera/chase.rs, replace en-dashes with `-` (lines ~65-68) and
      the curly apostrophe with a straight `'` (line ~107). Confirm with
      `grep -rnP '[^\x00-\x7F]' src/` returning nothing.
- [ ] Run `cargo fmt --check` and `cargo test --doc` to confirm the doc
      comments still render and compile.

## Notes

- Found during 20260703-095339 while editing chase.rs doctests; these
  chars are in the component/field docs, not the parts that task touched,
  so they were left for a separate change per the review convention.
- Pure text cleanup; no behavior change.
