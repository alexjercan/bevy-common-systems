# Replace non-ASCII typographic chars in src/camera/chase.rs docs

- STATUS: CLOSED
- PRIORITY: 10
- TAGS: bug

## Goal

`src/camera/chase.rs` is the only source file containing non-ASCII
typographic characters (en-dashes `-` and a curly apostrophe), which
violates the repo-wide plain-ASCII writing rule documented in AGENTS.md.

## Steps

- [x] In src/camera/chase.rs, replace en-dashes with `-` (lines 65-68) and
      the curly apostrophe with a straight `'` (line 107). Confirmed
      `grep -rnP '[^\x00-\x7F]'` over src/, bevy_common_systems_macros/src/
      and examples/ returns nothing.
- [x] Run `cargo fmt --check` and `cargo test --doc` to confirm the doc
      comments still render and compile. Both pass.

## Notes

- Found during 20260703-095339 while editing chase.rs doctests; these
  chars are in the component/field docs, not the parts that task touched,
  so they were left for a separate change per the review convention.
- Pure text cleanup; no behavior change.

## Close-out

What changed: replaced 4 en-dashes (U+2013) with `-` and 1 curly
apostrophe (U+2019) with `'` in src/camera/chase.rs doc comments, via a
targeted `sed` on those two byte sequences so nothing else was touched.
Alignment spacing in the combining-list doc (lines 65-68) was preserved.

Difficulty: none. The only care needed was replacing the specific
codepoints rather than a blind transform, to avoid disturbing the ASCII
around them; verified by a whole-tree non-ASCII scan before and after.

Self-reflection: this class of issue (typographic chars slipping into
docs) is best prevented, not repeatedly cleaned. Noted as a candidate for
a pre-commit / CI grep in the retro rather than fixed reactively each
time.
