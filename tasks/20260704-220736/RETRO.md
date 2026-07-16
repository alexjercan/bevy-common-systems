# Retro: 12_bastion defend-the-core tower defense

- TASK: 20260704-220736
- BRANCH: feature/12-bastion (squash-merged to master as cb0bd54)
- REVIEW ROUNDS: 1 (APPROVE; 2 MINOR + 2 NIT, both MINORs fixed in-round)

The largest example cycle so far: a ~950-line game that had to close three
never-demoed modules at once. See `tasks/20260704-220736/TASK.md` (plan +
close-out), `REVIEW.md` and `tasks/20260704-220736/NOTES.md` for what was
built and why; this retro is only about how the working went.

## What went well

- Facts-first, fanned out. Three parallel Explore agents gathered the exact
  public APIs (core gameplay, juice/UI, and a file-referenced skeleton of
  10/07/11) before a line was written. The first full-feature compile had only
  two trivial errors (a `DirectionalLight` field rename and one borrow), not a
  batch -- the standing "copy/verify the 0.19 surface, do not improvise it"
  lesson paid off again, and the up-front skeleton meant the states/harness/menu
  idioms were right the first time.
- Verified assumptions against the real crate source, not memory. Reading
  `src/health/mod.rs` confirmed `HealthZeroMarker` is the *only* destroyed-gate,
  so the Core revives on a second run -- the highest-risk correctness question,
  answered by reading rather than guessing. Reading `point_rotation.rs` gave the
  exact yaw/pitch convention, so the pitch-clamp sign was derived up front
  instead of by trial and error, and the orbit-rig design (apply the look
  quaternion to a pivot, keep the camera at a fixed child offset) came out clean.
- Caught that `mesh/explode` has NO `on_fragments_spawned` observer (the crate
  only inserts `ExplodeFragments`) before relying on it -- the module map and an
  early read implied a hook that does not exist. Mirrored `10_asteroids`' own
  `On<Insert, ExplodeFragments>` observer instead.
- Verified by running, not just compiling. The autopilot cycle plus a live
  `scrot` during a lengthened Playing hold proved the whole loop (place -> kill
  -> reward -> Core damage, score rising), catching nothing broken but earning
  the "done" honestly, per the AGENTS "not done until it has actually been run"
  rule.

## What went wrong

- The orbit-spin MINOR (R1.1) shipped in the first draft: `orbit_camera` was
  gated `run_if(not(GameOver))`, but it writes a per-frame `PointRotationInput`
  delta that `PointRotationPlugin` keeps integrating every frame -- so once the
  system stopped running, the last nonzero delta spun the camera forever behind
  the game-over screen. Root cause: I reasoned about *what* the system writes but
  not what happens when it *stops* running while a plugin keeps consuming its
  last output. This is the same "reason about when, not just what" failure the
  dev-harness retro (`20260704-190405`) already recorded for the autopilot input
  closure -- second occurrence of an input-across-states bug.
- The tautological upgrade test (R1.2): `upgrade_cost_scales_with_level` asserted
  `x*1 == x`, testing nothing, because the cost formula lived inline in
  `upgrade_selected` with no pure function to test. Root cause: wrote the test
  against the same consts the code uses instead of extracting the formula first.
  The repo already has a documented lesson about aspirational tests
  (grep-doc-claims / back claims with assertions) -- another repeat.
- Time sink: tried to screenshot a frame *with towers* by running
  `BCS_AUTOPILOT` and `BCS_SHOT` together; the screenshot never fired. Root
  cause: assumed the two harness plugins compose, but both drive `NextState`, so
  they fight and the screenshot's frame-count never settles. Fell back to
  autopilot + a longer Playing hold + external `scrot`, which worked.

## What to improve next time

- When a system writes an Input that a plugin integrates every frame, make it run
  in *every* state (or explicitly zero the Input on state exit) -- never gate it
  by state without accounting for the stale last value. Generalizes the
  dev-harness lesson from the autopilot closure to any Input/Output driver pair.
- Extract the pure formula and test *it*; never assert against the same const the
  code uses inline. If the test would be a tautology, the formula is not yet a
  function.
- `AutopilotPlugin` and `ScreenshotPlugin` are mutually exclusive in one run
  (both drive `NextState`). To capture a mid-gameplay frame, use `ScreenshotPlugin`
  alone, or `AutopilotPlugin` + external `scrot`. Added to AGENTS.md.

## Action items

- [x] Proposed AGENTS.md gotcha: the two harness plugins are mutually exclusive.
- [ ] Data-driven tower/enemy catalog + `SpecCatalog<T>` evaluation
  (tatr 20260704-220719, already seeded; depends on this).
- [ ] Minor polish left in docs, not blocking: forward-offset the turret barrel
  mesh (R1.3), and visible projectiles instead of hitscan -- fold into a future
  polish task if the example is revisited.

## Follow-up (post-merge, fix/12-bastion-orbit-upgrade)

The user played the merged build and hit two control bugs that this cycle
missed, correcting the "the orbit-rig design came out clean" claim above:

- **The orbit never worked.** `orbit_camera` accumulated `PointRotationInput`
  and read `PointRotationOutput`, but never wrote the output quaternion onto the
  rig's `Transform.rotation` -- and the plugin only maintains the Output
  component, not the Transform. The camera sat at identity the whole time.
- **Upgrade was unreachable** because selection only happened outside build mode,
  with no HUD hint.

Root-cause lesson (the important one): my verification checked a *proxy* -- "the
autopilot cycles with no panic and towers/enemies appear" -- not the actual
behavior the user exercises. The autopilot pressed `D` but I never confirmed the
view rotated, and it never taps a tower so the upgrade path was never driven at
all. "It ran and rendered something" is not "the controls do what they claim".
Next time: for each interactive control an example advertises, verify *that
control's observable effect* (here, the camera pose changing, and a tower
becoming selected + upgraded), by logging the state it drives or diffing two
frames -- not just that the app stays up. The refix used exactly that: a
temporary yaw log proved the pivot rotation now changes frame to frame.
