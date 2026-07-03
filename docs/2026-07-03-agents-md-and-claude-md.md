# Decision: AGENTS.md carries the content, CLAUDE.md imports it

Date: 2026-07-03
Task: tasks/20260703-094842

## What changed

Added `AGENTS.md` at the repo root as the orientation document for agent
sessions: crate purpose (a collection of Bevy utilities to build games
faster), repository layout, module map, conventions, features and
dependencies, toolchain, verified build/test commands, examples, workflow
and gotchas. Added `CLAUDE.md` containing only `@AGENTS.md`.

## Why this shape

- Two agent ecosystems read two different filenames: Claude Code loads
  `CLAUDE.md`, most other tools have standardized on `AGENTS.md`. The
  content is identical by definition, so duplicating it would guarantee
  drift.
- Claude Code supports `@path` imports in CLAUDE.md, so a one-line import
  gives both ecosystems the same single source of truth.
- Alternative considered: a symlink `CLAUDE.md -> AGENTS.md`. Works too,
  but the import line is explicit, survives checkouts on platforms with
  poor symlink support, and matches the convention already used in the
  user's global configuration (`~/.claude/CLAUDE.md` imports
  `~/AGENTS.md`).
- Alternative considered: full content in both files. Rejected: drift is
  certain, and there is no benefit.

## Notable choices inside AGENTS.md

- Commands are documented as verified on 2026-07-03, including the honest
  state of `cargo test`: doctests currently fail, so the documented suite
  is `cargo test --all-targets` plus a pointer to the fix-up task
  (tasks/20260703-095339). Documenting a broken command as if it worked
  would cost future agents more time than the note costs in space.
- The Config/Input/Output/State component pattern is called out as the
  crate's core convention, since every module follows it and new code
  should too.
- Known code smells found while writing the docs were filed as tatr tasks
  (20260703-095339 doctests + clippy warning, 20260703-095509 macro
  default path) instead of being fixed in the docs branch, to keep the
  diff reviewable.

## Difficulties

- `cargo test` failing on doctests was anticipated during planning (the
  module docs visibly reference out-of-scope variables) and confirmed by
  running it; the resolution (document reality, file a follow-up task)
  was decided at planning time rather than discovered in a broken CI run.

## Self-reflection

- Reading every module head before writing (rather than trusting the
  planning summary) surfaced two real inaccuracies that would otherwise
  have landed in the docs: the post-processing plugin keys off a marker
  component (not off `Camera3d` as its module doc claims), and the status
  bar closures take `&World` per update. Verify-before-document is worth
  keeping as the default.
- The cross-check pass caught a worse one: the draft claimed "there are no
  unit tests yet" because an earlier `cargo test | tail` truncated the lib
  test results. The crate has 13 passing unit tests. Lesson: when a
  command's output is summarized (tail/grep), do not derive negative
  claims ("there is no X") from it; re-run and look at the full relevant
  section first.
- Reading only the first 50-60 lines of leaf modules was enough for the
  module map but not for behavioral claims (the PD controller writes its
  torque to an Output component instead of applying it; discovered only
  when reading the whole file during cross-check). Behavioral sentences
  in docs need whole-file reads.
