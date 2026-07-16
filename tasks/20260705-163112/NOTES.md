# 08_dropzone: keep the camera on the crash site

Task: `tasks/20260705-163112`.

## What changed

When a run ends in destruction, `examples/08_dropzone.rs` now keeps the chase
camera framed on the spot where the hull broke apart, so the player watches the
`mesh/explode` debris fly, instead of the camera swooping up to the
planet-from-above vantage.

- New `CrashSite(Option<CrashView>)` resource records the world position and
  forward heading of the hull the frame it explodes. `CrashView` carries just
  `pos` and `heading` -- enough for the camera to rebuild the same
  `surface_frame(up, heading)` it uses in flight.
- Both ship-destruction paths set it: the hard-terrain crash branch in
  `resolve_collisions` and the integrity-depleted `on_ship_destroyed` observer.
  A soft landing and an asteroid shatter do NOT (a soft landing keeps the parked
  hull, which the camera follows directly; an asteroid is not the ship).
- It is cleared at run start (`start_run`) and on leaving the result screen
  (`cleanup_run_scene`), so a fresh run and the menu return to the spawn vantage.
- `drive_chase_camera` picks its anchor through a new pure helper,
  `camera_anchor(ship, crash)`: live ship first, then the crash site, then the
  spawn vantage.

## Why this shape

The crash hull carries `DespawnOnExit(Playing)`, so it is gone the moment the
state flips to `Result`, leaving only the fragments. `drive_chase_camera`,
finding no `Ship`, previously fell straight back to `ship_start_pos()` (above the
+Y pole) -- that fallback is what pulled the camera off the explosion. Recording
the last hull transform and using it as an intermediate fallback is the smallest
change that keeps the existing in-flight framing math (radial up + tangent
heading via `surface_frame`, per the orbit-rotation fix) and touches no other
state.

The result UI is a transparent centered text overlay, so the 3D scene shows
through behind it -- no UI change was needed to make the explosion visible.

## Verification

- `camera_anchor` is a pure function, unit-tested off the ECS for all three
  priority cases (ship wins over a recorded crash; crash site beats the spawn
  vantage; neither falls back to spawn). Per the glide and orbit-rotation retros,
  making the rendering-driver logic pure and testing it is more reliable than a
  screenshot, especially since `ScreenshotPlugin` snaps at state entry and the
  `AutopilotPlugin` forces `Playing -> Result` on a timer WITHOUT a real crash --
  so neither harness ever records a `CrashSite` or exercises the crash branch.
- `cargo fmt`, `cargo clippy --all-targets`, `cargo test --examples` (the new
  test passes), `./scripts/check-ascii.sh`, and a headless
  `BCS_AUTOPILOT=1 ... --features debug` run to `cycle complete, no panic` all
  pass.
- A live visual grab of the framed explosion was attempted (a throwaway edit to
  cut the autopilot's thrust so the ship free-falls) but abandoned: the harness
  window could not be captured reliably in this session and the detour was not
  worth more time given the logic is covered by the pure test. The behavior
  reduces to: both crash paths set `CrashSite` from the ship transform (trivially
  correct) and `camera_anchor` selects it (unit-tested).
