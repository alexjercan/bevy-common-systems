# Retro: KISS pass: debug/ + lib.rs + completion.rs (sets the comment convention)

- TASK: 20260731-172208
- BRANCH: refactor/kiss-debug-cluster
- REVIEW ROUNDS: 2

## What went well

- Building the checker BEFORE the first source edit, and requiring it to
  report the predicted 37 blocks on the untouched tree, turned the pass from
  judgement into a worklist. "Did I miss one" never came up.
- Planning measured the tree instead of trusting the epic's prose: every
  candidate proof ran on base, and only the red ones were kept as
  discriminators (checker 37, HUID grep 2, in-scope doc warning 1). The
  generic suite stayed as regression guards, not as evidence of the change.
- Review re-derived "no behavior change" by a DIFFERENT method than the task
  claimed -- stripping comment and blank lines from both revisions and
  diffing -- which found the single real code delta (an added assert message)
  that the task's own `pub`-line grep would not have shown.
- Probing the checker on a synthetic file rather than reading its regex
  produced both MINOR findings and, indirectly, the MAJOR.

## What went wrong

- **The convention's scope was assumed, never decided.** The AGENTS.md bullet
  said "Inline (non-doc) comments"; the checker only matched own-line ones.
  The choice seemed sound because the epic's own HUID proof uses the same
  `^\s*//` anchor, so mirroring it felt like consistency with the parent. The
  flaw: that grep is a narrow HUID probe, while this task's deliverable is the
  RULE, and a rule takes its authority from the words, not from the grep it
  was modelled on. Cost one review round; would have cost four sibling tasks
  working against a rule whose stated scope and enforcement disagreed, with
  exit 0 reading as compliance.
- Two smaller instances of the same root cause -- writing the tool from intent
  instead of probing it. The tag regex hard-coded one space, so a correctly
  written `//NOTE:` was reported as untagged while the error text told the
  author to add the tag they had written; and filenames were word-split into
  awk. Neither is reachable in this repo today, but both are reachable through
  the `<path>...` interface the bullet now advertises.
- I nearly recorded "clippy clean" twice on the strength of `tail -3` output.
  The full log carries a pre-existing `manual_contains` at
  `completion.rs:88`. Exit code 0 made the DoD pass honestly, but the RECORDS
  would have been wrong. Caught only because round 2 grepped the whole log.

## What to improve next time

- When the deliverable IS a convention, treat the prose and the enforcing tool
  as one artifact: list what the words cover, list what the tool matches, and
  name every gap as either an explicit exemption or a bug. Neither half is the
  spec alone.
- Probe a new checker against cases it must REJECT and cases it must ACCEPT,
  on a synthetic file, before trusting it on the real tree. A clean exit on
  real sources proves nothing about a tool that silently matches less than
  intended; that failure mode is invisible by construction.
- For the sibling clusters: read the item's own rustdoc before ruling a
  comment load-bearing. Five of the 21 comments dropped here were restating a
  `///` block within ten lines.

## Action items

- [x] Narrow the AGENTS.md bullet to own-line comments and state the
      end-of-line exemption in both the bullet and the script header.
- [x] Accept any spacing after the slashes in the tag regex.
- [x] NUL-delimited filename hand-off in `check-comment-tags.sh`.
- [x] File the pre-existing clippy lint as its own task (20260731-180747)
      rather than folding it into a comment-hygiene diff.
- [ ] Siblings 20260731-172223 / 172224 / 172232 / 172233 reuse
      `scripts/check-comment-tags.sh <their paths>` as their comment proof; no
      task should re-derive the rule by hand.
