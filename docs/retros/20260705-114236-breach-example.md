# Retro: 14_breach, a grounded first-person arena shooter

- TASK: 20260705-103236
- BRANCH: 14_breach (squash-merged to master as faf012e)
- REVIEW ROUNDS: 2 (Round 1 REQUEST_CHANGES on 1 major + 4 minor + 1 nit, Round 2 APPROVE)

See `tasks/20260705-103236/TASK.md` and `.../REVIEW.md` for what changed and the
findings; this is about how the working went.

## What went well

- The spike had already done the hard homework (camera/wasd is free-fly, no
  raycast, no character controller), so planning could commit to the right
  architecture up front: a dynamic capsule + `LockedAxes::ROTATION_LOCKED` driven by
  `LinearVelocity` (solver does collide-and-slide) instead of a hand-rolled kinematic
  controller. Front-loading the exact avian-0.7 / bevy-0.19 API signatures into the
  plan (SpatialQuery, CursorOptions, AccumulatedMouseMotion, layers) meant the
  ~1300-line example compiled after a handful of small fixes.
- The first runtime bug (tracer/flash inserted on an enemy despawned by the same
  frame's death chain) was the exact `13_glide` despawn-race, and the fix (order the
  side effect before the despawn) was already in muscle memory.
- Making the autopilot *aim* at the nearest enemy (setting the controller yaw in the
  input closure) was the right instinct to exercise the gun headlessly.

## What went wrong

- MAJOR (R1.1): the enemy melee barely worked and the lose condition was never
  verified, both hidden behind a green autopilot run. Two root causes. First, a
  per-enemy attack `Cooldown` gating a distance check is unreliable when the player
  and enemies are both dynamic bodies: collision knockback flings an approaching
  enemy out of `MELEE_RANGE`, so it is rarely in-range-and-cooldown-ready at once --
  a defenceless player survived 30s+. Fixed with continuous proximity damage
  (`ENEMY_DPS * dt` per in-range enemy) and by dropping player-vs-enemy physical
  collision so enemies overlap you. Second, straight-line enemy AI (no avoidance) got
  stuck on the interior cover blocks, so some enemies never arrived -> open arena.
- The deeper process failure: I declared the example "verified" off an autopilot run
  that reported Menu->Playing->GameOver with 2 kills. But `AutopilotPlugin`
  force-transitions on a fixed `.hold(Playing, N)` timer, so the observed GameOver was
  *always* the timer, never the game's own player-death path. The harness that proves
  the win/interaction side (kills, no panic) is structurally blind to a game-driven
  *lose* transition. This is the `13_glide` lesson recurring for the second cycle:
  a happy-path harness masks the path it doesn't drive.

## What to improve next time

- Treat `AutopilotPlugin.hold(state, N)` as proof only of what happens *up to* the
  forced transition, never of the transition itself. Any state change the *game*
  makes (lose condition, win, level-up) must be verified another way -- a headless
  `App` unit test (MinimalPlugins + StatesPlugin + the relevant plugin), which is
  what finally covered player-death -> RunOver -> GameOver here. When adding a game
  with a lose condition, write that test first, before trusting the autopilot.
- Diagnosing gameplay balance headlessly needs *measurement*, not just the pass/fail
  transition log. Extending the Playing hold, disabling the autopilot's fire, and
  logging per-second HP / enemy-distance is what turned "player mysteriously survives"
  into "enemies arrive at 7s, melt HP 100->0 in 3s" -- reach for that instrumentation
  earlier instead of reasoning about the numbers.
- Dumb straight-line AI and interior obstacles do not mix; decide arena topology from
  the AI's capabilities, not aesthetics.

## Action items

- [x] Fixed the melee/arena and added headless lose/score tests (in the merged commit).
- [x] Sharpened the AGENTS.md harness gotcha: a forced `.hold()` autopilot cannot
      verify a game-driven state transition (e.g. the lose condition); cover those
      with a headless App test. Added in this commit.
- [ ] tatr 20260705-103238 (already seeded): the FP-controller / camera-wasd harvest
      follow-up -- unchanged by this retro, noted for continuity.
