# Retro: port 07/08/10 onto camera/shake

- TASK: 20260704-144509
- BRANCH: feat/camera-shake-ports (squash-merged to master as 10183de)
- REVIEW ROUNDS: 1 (APPROVE, clean)

Follow-up to tasks/20260704-134500 (the camera/shake module itself). This retro
is about how the porting cycle went, not the diff (see TASK.md Resolution).

## What went well

- Ran the three ports as parallel subagents (one per example file). The files are
  independent, so there was no collision, and three ~large refactors landed in the
  time of one. Verification stayed with me (the parent): each agent only built its
  own example, and I ran the combined fmt/clippy/test/boot suite once at the end.
- Precise, gotcha-loaded per-file instructions paid off: every one of the three
  came back clean because the prompts named the specific traps up front -- keep
  the `MainCamera` marker (it has non-shake users), watch for a `&mut
  CameraShakeInput` vs `&mut Transform` borrow conflict on the camera entity,
  delete 07's now-orphaned helper unit test, and move 10's `fit_camera` to
  PostUpdate between Restore and Apply. None of those needed a rework round.
- The hardest correctness question -- does 10's non-chase `fit_camera` base driver
  compose without drift -- was already answered by a *library* test written in the
  previous cycle (`composes_with_a_moving_base_driver` models exactly a base
  rewritten between Restore and Apply). So the one thing a background boot cannot
  check (shake feel / framing) was covered by a deterministic test, not a guess.
  Investing in that test last cycle paid off this cycle.

## What went wrong

- Hit the `fresh`-worktree base pitfall again: my first worktree branched from
  `origin/master`, which lacks the unpushed camera/shake module, so the worktree had
  no `src/camera/shake.rs` to port onto. Root cause: took the origin-base form when
  the work depends on unpushed local commits. Already documented in memory from last
  cycle; I still reached for the wrong form first. Fixed by `git worktree add ...
  HEAD` + entering that worktree by path (use the sprout skill, based on local HEAD).
- A subagent left a formatting artifact (a stray blank line where it deleted 07's
  test), caught by `cargo fmt --check` at the combined step. Harmless but avoidable:
  the port agents were told to build and clippy their file, not to `cargo fmt` it.

## What to improve next time

- For a background worktree that must build on unpushed local commits, always
  create it from local HEAD (`git worktree add <path> -b <branch> HEAD`, or the
  sprout skill based on HEAD) and enter it by path -- never an origin-based form,
  which branches from origin and will be missing the local work. (Standing rule now;
  it has bitten two cycles.)
- When dispatching parallel edit subagents, tell them to run `cargo fmt` on their
  file before reporting, not just build/clippy. Formatting is cheap for them and
  saves a parent-side fixup.

## Action items

- [x] No follow-up code work: the port task fully delivered (07/08/10 on the
  module, ~90 lines of duplicated shake removed, module now demonstrated across a
  chase camera, a static camera and a fit-to-arena camera).
- [ ] Optional future: the remaining Wave-1 kit modules (`ui/popup`, `feedback`
  hit-flash) from the spike are still open tatr tasks (20260704-134530,
  20260704-134600) if the kit effort continues.
