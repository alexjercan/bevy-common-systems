# Retro: 07_orbit - Orbit Runner surface-dodge example

- TASK: 20260703-165427
- BRANCH: feature/07_orbit (merged to master)
- REVIEW ROUNDS: 2 (R1 REQUEST_CHANGES, R2 APPROVE)

See TASK.md close-out and `tasks/20260703-165427/NOTES.md` for the what/why of
the example; this retro is only about how the working went.

## What went well

- Read before writing. Mapping the exact public APIs of the orbit family,
  `ChaseCamera`, `meth`, `Health` and `SfxPlugin` up front (a parallel Explore
  agent plus reading the sources directly), then studying `06_fruitninja` as the
  template, meant the ~1200-line example compiled with only one API miss. The
  Config/Input/Output split and the state/HUD scaffolding were copied from a
  known-good example rather than reinvented.
- Picked the correct-everywhere design over the easy one. Steering integrates a
  moving orthonormal frame along a great circle (`step_runner_frame`) instead of
  integrating `(theta, phi)` directly, which has a pole singularity. It is more
  code but it roams the whole sphere without blowing up -- exactly the AGENTS.md
  "correct over quick" call. The trickiest math (frame stays orthonormal,
  advancing moves toward the heading, no sideways drift, camera axes align) is
  pinned by unit tests, which is what let a headless cycle ship with confidence.
- Heeded the standing retro gotchas and they paid off: verified the wasm build
  through the real `trunk build` entry point (not assumed from the mirrored 06
  page), judged every build by a redirect + `$?` rather than a piped `| tail`,
  and ran `cargo test --example 07_orbit` explicitly because plain `cargo test`
  does not execute example in-file tests.
- Merging master mid-flight surfaced the WebKit audio-unlock shim that had just
  landed for 06; mirrored it into the 07 page in the same pass, so the new game
  is not silent-on-mobile from day one.

## What went wrong

- `AmbientLight` as a resource. I wrote `commands.insert_resource(AmbientLight
  {..})` from memory; in Bevy 0.19 `AmbientLight` is a per-camera component, not
  a resource. Root cause: coded a familiar engine idiom without checking it
  against the pinned version. Caught at first compile, cost one build round.
- The spawn-on-player defect (R1.1, MAJOR) escaped implementation. Hazards/orbs
  spawned at fully random angles with no guard, so a level-up top-up (or an
  initial hazard) could materialize on the marker and deal unavoidable damage.
  Root cause: I reasoned hard about the motion/camera correctness and did not
  think through spawn *fairness* -- a spawner edge case, not a math one. Caught
  in my own review, not implementation.
- The new `spawn_is_clear` test failed on first run. I built the "near" test
  candidate with `spherical_to_cartesian(1, theta=small, 0)`, assuming small
  theta is near the +Z marker -- but the module measures theta from -Z, so that
  point is on the *opposite* pole and reads as clear. The guard itself was
  correct (it compares real 3D directions); only the fixture was wrong. Fixed by
  building candidates as `Quat::from_axis_angle(..) * runner`, independent of the
  angle convention.

## What to improve next time

- When writing an engine API from memory (lights, UI, states), verify it against
  the pinned version before the first build -- a 10-second grep of the crate
  docs/source beats a multi-minute Bevy compile to find out.
- Add "can this spawn on or right next to the player and deal unavoidable
  damage?" to the mental checklist for any spawner in a gameplay example. It is
  the spawn analogue of the movement edge cases I did check.
- Build test fixtures independently of the coordinate convention under test:
  rotate a known vector by a known angle rather than hand-picking angles in the
  convention, so a convention mismatch cannot masquerade as a logic bug.

## Action items

- [x] Both review findings that needed code (R1.1 spawn guard, R1.2 defensive
  `frame_rotation`) fixed and re-verified in round 2.
- [ ] No new follow-up tasks. The CI-does-not-run-example-tests gap that also
  applies here is already tracked by 20260703-175735 (from the fruitninja-touch
  retro); the 11 example tests were run locally this cycle.
- Left as a retro observation (not an AGENTS.md edit yet): "Bevy 0.19
  `AmbientLight` is a per-camera component, not a resource" -- version-specific,
  will promote to a gotcha only if it recurs.
