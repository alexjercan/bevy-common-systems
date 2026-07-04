# Retro: modding JSON-authored EventHandler registry

- TASK: 20260703-165439 (CLOSED)
- BRANCH: feature/modding-json-registry (squash-merged to master as 4e791b8)
- REVIEW ROUNDS: 2 (Round 1 REQUEST_CHANGES, one MAJOR + five MINOR + two NIT;
  Round 2 APPROVE)

See `tasks/20260703-165439/TASK.md` and
`docs/2026-07-04-modding-json-registry.md` for what was built and why. This
retro is only about how the working went.

## What went well

- Understand-first paid off on the design. Reading `events.rs`, the
  `EventKind` derive macro and `examples/03_modding` before writing anything
  made the core architecture right on the first pass: the trait objects cannot
  be deserialized directly, so the registry inverts it (the game registers
  concrete types under string names; JSON only ever names them). Round 1 raised
  zero BLOCKER/MAJOR findings against the architecture -- every finding was
  polish or a test gap, not a rethink.
- Delegating an independent skeptical review to a subagent earned its keep
  again. It caught the test-honesty gap (R1.1) and the `deny_unknown_fields`
  and by-reference-deserialize improvements -- exactly the kind of thing that is
  invisible when reviewing your own diff. Same lesson as the 11_overload retro:
  a second set of eyes on the diff finds what self-review rationalises away.
- Verified through the real entry points, not just `cargo build`: ran
  `03_modding` on the live display twice (before and after the review round) and
  confirmed it boots to the render loop AND that the JSON-authored filter
  actually gates the action (values below 0.5 add +1, above add +2). That is the
  standing "an example is not done until it has booted once" gotcha, applied.

## What went wrong

- R1.1 (MAJOR): the build test's comment claimed "the action adds the
  configured amount" while the test only called `handler.filter(...)` -- the
  action was never run and the counter never asserted. Root cause: I wrote the
  assertion comment aspirationally and stopped at the filter because exercising
  the action needed reaching for the `pub(super)` `actions` field, which I did
  not do until the review pushed me to. This is the *exact* recurring lesson
  from the 11_overload retro ("do not write a note that claims coverage the code
  does not have; write the test first"). It bit a second time, which makes it a
  pattern, not a one-off.
- R1.4: reflexively used `serde_json::from_value(params.clone())` (owned) when
  `F::deserialize(params)` deserializes the borrowed `&Value` with no clone.
  Small, but it is the habit of reaching for the owned serde entry point without
  thinking about the borrow.

## What to improve next time

- When a test comment asserts behaviour X, the same edit must make the test
  assert X. If reaching the behaviour needs a `pub(super)`/helper hop, take the
  hop -- do not downgrade the test and leave the comment. (Second occurrence;
  promoted to an AGENTS.md rule below.)
- For any serde data-authoring surface (mod files, configs), default to
  `#[serde(deny_unknown_fields)]` and by-reference deserialization from the
  start, rather than adding them under review.

## Action items

- [x] AGENTS.md: added a testing-convention line -- a comment/note that claims a
  test exercises some behaviour must be backed by an assertion in the same edit
  (this lesson has now surfaced in two consecutive retros).
- [ ] Follow-up (no task filed yet): the registry is the precondition for a
  data-driven "Reactor" modding game (mentioned in TASK.md); a future example
  could load handler JSON from an asset file rather than an inlined string,
  exercising the hot-reload / file path. Left out deliberately as out of scope.
