# Retro: 08_dropzone crash-site camera on game over

- TASK: 20260705-163112
- BRANCH: feature/dropzone-crash-camera
- REVIEW ROUNDS: 1 (APPROVE)

See `tasks/20260705-163112/TASK.md` and
`tasks/20260705-163112/NOTES.md` for what changed and why. This retro
is about how the working went.

## What went well

- Reading the two crash paths (`resolve_collisions` terrain branch and the
  `on_ship_destroyed` observer) and the single `drive_chase_camera` fallback
  before writing anything located the exact fault: the camera fell back to
  `ship_start_pos()` the moment the hull despawned. The fix was then obvious and
  small (record the last hull transform, use it as an intermediate fallback).
- Extracting the anchor priority into a pure `camera_anchor(ship, crash)` helper
  made the whole feature unit-testable off the ECS, straight from the
  orbit-rotation and glide retros' standing advice ("make the rendering-driver
  logic pure and test it"). The test asserts all three priority cases including
  `assert_ne!(pos, ship_start_pos())` for the crash case, so it actually
  observes the behavior the feature exists for.
- Reusing the existing `surface_frame` (instead of a fresh `from_rotation_arc`)
  meant the crash-tumble edge case -- a nose-down heading nearly parallel to the
  radial up -- was already handled (`any_orthonormal_vector` fallback, with its
  own test). Checking that during review took one read, not a new test.

## What went wrong

- Sank real time into a live visual capture that did not pan out. The plan was
  to cut the autopilot's thrust so the ship free-falls and crashes, then grab
  the window. Two things bit: (a) the free-fall did not reach terrain inside the
  12s Playing hold even though a=5.5, alt=22 predicts a ~3s fall -- so the ship
  was not actually free-falling as assumed, and I never diagnosed why; (b)
  `xdotool` was not on PATH, so `nix run nixpkgs#xdotool` started compiling and
  the harness window was already gone by the time a grab was possible. Root
  cause: chasing a screenshot the project's own retros repeatedly say is the
  weakest form of verification for logic-that-drives-rendering, when the pure
  test already covered it. The autopilot cannot crash the ship on its own, so
  there was never a cheap path to a live crash frame.
- Tripped the `pkill -f` self-match footgun again (exit 144) killing the run by
  pattern instead of by the `$!` PID I already had. This is an existing
  MEMORY.md note; I did not apply it.

## What to improve next time

- When a feature's logic is already covered by a pure unit test, do NOT spend a
  second budget on a live screenshot "to be sure" unless there is a cheap,
  reliable path to the exact frame. For a state-machine example the autopilot
  cannot drive to a real lose-state, that path usually does not exist -- accept
  the pure test and say so, as the glide/orbit retros already prescribe.
- Kill background runs by the `$!` PID captured at launch, never `pkill -f` with
  a pattern that appears in the killing command's own args.

## Action items

- [x] No AGENTS.md change: the relevant lessons (pure-test over screenshot,
  `pkill` self-match) are already recorded in the Gotchas and MEMORY.md; this
  cycle is a reminder to apply them, not a new rule.
- [x] No follow-up task: the review's MINOR R1.1 (no integration test that the
  crash paths populate `CrashSite`) was accepted with reasoning -- a direct
  observer test needs `Res<SoundBank<Sfx>>` asset loading, disproportionate to
  two trivial writes. Recorded as the known coverage boundary in the doc note.
