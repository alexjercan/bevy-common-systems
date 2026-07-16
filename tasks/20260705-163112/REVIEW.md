# Review: Dropzone - keep camera on crash site so the explosion is visible

- TASK: 20260705-163112
- BRANCH: feature/dropzone-crash-camera

## Round 1

- VERDICT: APPROVE

Reviewed `git diff master...HEAD`, TASK.md, `surface_frame` and its tests, and
re-ran `cargo test --examples` (22 example tests green, including the new
`camera_anchor` case and the `surface_frame` degenerate-forward case). Clippy
was clean per /work; the diff adds no new lint surface.

The change delivers the Goal cleanly: a `CrashSite` resource records the hull's
position + heading the frame it explodes (set in BOTH destruction paths -- the
terrain-crash branch of `resolve_collisions` and the `on_ship_destroyed`
observer, and correctly NOT on a soft landing or asteroid shatter), it is
cleared at run start and on leaving Result, and `drive_chase_camera` prefers it
over the spawn vantage via a pure `camera_anchor` helper. The anchor position
equals the ship's last translation, so there is no camera jump across the
despawn frame -- the framing is continuous into the explosion. Doc comments on
`ship_start_pos` / `drive_chase_camera` were updated to match. Good reuse of the
existing `surface_frame` (no bare `from_rotation_arc`, per the orbit-rotation
gotcha).

Edge case checked and clear: a tumbling nose-down crash can store a `heading`
nearly parallel to `radial_up`, but `surface_frame` handles a degenerate
forward with `up.any_orthonormal_vector()` (and has its own unit test), so the
frame stays finite -- no NaN, just an arbitrary-but-stable yaw. Acceptable.

No BLOCKER or MAJOR findings. Two non-blocking notes left to the implementer's
discretion:

- [ ] R1.1 (MINOR) examples/08_dropzone.rs:2039,2267 - the pure `camera_anchor`
  test covers anchor *selection* but nothing verifies the two crash paths
  actually *populate* `CrashSite` (the glide-retro trap: test the side-effect,
  not just the result). If the `crash_site.0 = Some(..)` write regressed, every
  test would still pass while the feature silently broke. A headless `App` test
  that drives `on_ship_destroyed` and asserts `CrashSite` becomes `Some` at the
  ship's position would close it. Acknowledged friction: `on_ship_destroyed`
  takes `Res<SoundBank<Sfx>>`, whose handles need asset loading, so a direct
  observer test is non-trivial for an example; the two write lines are also
  dead-simple and mirror the adjacent (untested) `Outcome` writes. Reasonable to
  accept as-is given the example-as-integration convention, but flagging the gap.
  - Response: Accepted with reasoning (not fixed). A direct observer/collision
    App test needs `Res<SoundBank<Sfx>>` populated with loaded handles (asset
    loading in a headless test), disproportionate to two trivial write lines. A
    pure `CrashView::from_transform` helper would test the *value* but not that
    the write *happens* (the real regression risk), so it would not close the
    gap. Left as-is; recorded as the known coverage boundary in the doc note.

- [x] R1.2 (NIT) tasks/20260705-163112/TASK.md:50 - the verification step's
  "screenshot the Result state after a crash" is ticked, but the live visual
  grab was abandoned (transparently noted in
  `tasks/20260705-163112/NOTES.md`). The box slightly overstates what
  was done; the step is conditional ("if a display is available") and the core
  verification (the pure test) is complete, so this is cosmetic.
  - Response: Fixed. Reworded the TASK.md verification line to state the
    screenshot was attempted and abandoned, so the ticked box no longer
    overstates what was done. Verified by re-reading TASK.md.
