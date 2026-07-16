# Retro: 13_glide, a UI-forward slide-merge puzzle

- TASK: 20260705-090624
- BRANCH: 13_glide (squash-merged to master as 416a251)
- REVIEW ROUNDS: 2 (Round 1 REQUEST_CHANGES on 1 blocker + 1 major + 4 minor/nit, Round 2 APPROVE)

See `tasks/20260705-090624/TASK.md` and `.../REVIEW.md` for what changed and the
findings; this is about how the working went.

## What went well

- Front-loading the exact crate APIs into the plan (an Explore agent dumped every
  signature for `tween`/`persist`/`popup`/`menu`/`pointer`/`audio`/harness before a
  line was written) made the ~1300-line example mostly mechanical and it compiled
  after one small fix. Copying the visual idioms from `11_overload` rather than
  memory avoided the recurring Bevy-0.19 UI gotchas entirely.
- Two real bugs were caught by *acting on* the plan's flagged unknowns rather than
  guessing: the tween-completion vs despawn race showed up on the first autopilot
  run (a merged tile despawns exactly when its slide tween completes), fixed by
  ordering the resolve system `.before(TweenSystems::Advance)` plus
  `TweenOnComplete::Keep`. Deciding up front to animate `Node` fields (not
  `Transform` scale on UI) sidestepped an entire class of version-fragility.
- The independent review sub-agent and my own re-read converged on the same
  blocker, and the fix (a pure `classify_moves` helper) both fixed it and made it
  testable in the same move.

## What went wrong

- BLOCKER (R1.1): merged tiles rendered the stale (un-doubled) number because the
  `merges` list was always empty. Root cause: `start_move` classified a merge off
  each `GridMove.merged` flag, but `resolve_line` emits the base placement
  (`merged:false`) to a cell *before* the incoming (`merged:true`) move, so the
  `merged:true` branch always hit the "cell already claimed -> despawn" path and
  never bumped the survivor. The pure grid/score logic was correct, so it was a
  silent visual desync.
- Why it survived my "verification": I declared the example verified off a headless
  autopilot run (no panic, reached game-over) and a `ScreenshotPlugin` grab -- but
  the screenshot was taken at Playing *entry*, before any move, so it showed only
  the two initial `2` tiles. I never observed a *rendered merge*. And the
  moves-to-entity mapping (the exact buggy code) had zero test coverage: every
  `apply_move` test discarded the moves vector with `_`. This is the repo's own
  "tested only the easy half" trap, and I walked into it.
- Time sink: trying to get a live gameplay screenshot afterwards, `scrot` kept
  returning a byte-identical stale menu framebuffer from an orphaned earlier run in
  the headless X session, even after the live game provably reached Playing. Several
  cycles were lost to window/compositor state (and a `pgrep -f` self-match that
  killed my own shell). The app-native `ScreenshotPlugin` render had worked fine;
  `scrot` of the root window did not.

## What to improve next time

- For any state-machine example, a screenshot at *state entry* is not gameplay
  verification. Either drive N moves before the capture, or -- since
  `ScreenshotPlugin` and `AutopilotPlugin` are mutually exclusive and `scrot` is
  unreliable headless -- assert the mid-game invariant in a unit test instead. Here
  the fix was to make the entity-mapping pure (`classify_moves`) and test it; that
  is more reliable than any screenshot and should be the default for
  "logic-that-drives-rendering".
- When a pure function returns *both* a result and a list describing side effects
  (here the grid *and* the per-tile moves), test the list, not just the result. The
  result being right (correct grid/score) actively masked the list being
  mishandled downstream.
- Prefer the app-native `ScreenshotPlugin` (captures the app framebuffer) over
  `scrot` of the root window for visual checks; `scrot` fought the WM here. If a
  live gameplay frame is truly needed, kill stale game windows first (by exact
  process name, `pgrep -x`, never `pkill -f <pattern-in-your-own-cmdline>`).

## Action items

- [x] Fixed the blocker and added 3 `classify_moves` tests (in the merged commit).
- [x] Proposed AGENTS.md gotcha (added in this commit): a state-machine example
      screenshot at state entry is not gameplay verification; test
      logic-that-drives-rendering as a pure function.
- [ ] tatr 20260705-090557 (already seeded): the UI-juice helpers harvest follow-up
      -- unchanged by this retro, noted for continuity.
