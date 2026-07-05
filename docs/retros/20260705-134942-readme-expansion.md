# Retro: Expand README.md into a proper crate README

- TASK: 20260705-134942
- BRANCH: docs/readme-expand (squash-merged to master as 775eb03)
- REVIEW ROUNDS: 2 (1x REQUEST_CHANGES, then APPROVE)

See TASK.md close-out for what changed; this is about how the working went.

## What went well

- Sourced the module map from `src/lib.rs` + each module's `//!` doc and the
  version facts from `Cargo.toml`, not from AGENTS.md. That was the right call:
  AGENTS.md turned out to be partly stale (see below), so trusting it would
  have shipped wrong facts into a public-facing README.
- The Quickstart snippet was adapted from `src/health/mod.rs`'s own tested
  doctest rather than improvised, so the one code block in the README is known
  to reflect the real API.
- Cheap, targeted verification (grep for type names, `ls examples/`, an ASCII
  scan of the README specifically since check-ascii.sh does not cover it)
  caught concrete facts instead of vibes.

## What went wrong

- R1.1 (MAJOR): the `scoring` line invented `Score` and `Combo` type names.
  The real API is `Streak` + `HighScore<T>`; `Combo` only exists as a doc-
  comment reference to the game-local pattern `Streak` replaces. Root cause:
  I wrote the scoring one-liner from the mental model of the fruit-ninja combo
  mechanic instead of reading `src/scoring/`, even though I had explicitly
  decided to source everything from code. One module slipped past that
  discipline -- the one whose name (`scoring`) most invited a guess.
- AGENTS.md is not a safe single source of truth for a public doc: its Module
  Map omits the harvested modules (feedback, input, material, persist,
  scoring, time, tween, and several camera/ui submodules) and its "Features
  and Dependencies" section still lists bevy 0.18 / avian3d 0.6 / inspector
  0.36 while Cargo.toml is 0.19 / 0.7 / 0.37. I noticed this mid-task and
  routed around it, but it cost a re-derivation pass.

## What to improve next time

- When a doc claim names a *type* or *API*, grep for that exact symbol before
  writing it, every module, no exceptions -- do not let a familiar-sounding
  module name (scoring -> "Score/Combo") substitute for reading the code. The
  cross-check step caught it, but it should not have needed to.
- Treat AGENTS.md as a lead, not a citation. Its module map and dependency
  versions lag the code; verify both against src/ and Cargo.toml.

## Action items

- [ ] tatr 20260705-140043: refresh AGENTS.md's stale dependency versions and
  fill in the missing modules in its Module Map (follow-up; out of scope for a
  README-only task).
- [ ] Considered but not proposed: an AGENTS.md rule "grep the symbol before
  documenting it". It is already implied by the existing "nothing invented"
  convention and the grep-doc-claims memory; adding another line would be
  noise. Left as a retro lesson only.
