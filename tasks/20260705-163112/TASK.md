# Dropzone: keep camera on crash site so the explosion is visible on game over

- STATUS: CLOSED
- PRIORITY: 80
- TAGS: feature,example

## Goal

When a run in `examples/08_dropzone.rs` ends in destruction (a hard terrain
crash or integrity depleted to zero), the result screen should keep the chase
camera framed on the spot where the hull broke apart, so the player watches the
`mesh/explode` debris fly instead of the camera swooping away to the
planet-from-above vantage.

## Current behavior

On a crash the hull entity is given `ExplodeMesh + DespawnOnExit(Playing)`, so
it despawns the moment the state changes to `Result`, leaving only the flying
fragments. `drive_chase_camera` runs every frame in every state and, finding no
`Ship`, falls back to `ship_start_pos()` (straight above the +Y pole). That
fallback pulls the camera off the crash and up to the planet overview, hiding
the explosion. (A soft landing keeps the parked hull, so the camera already
follows it there -- that path must not change.)

## Approach

Record the crash location at the moment the hull explodes and use it as the
chase-camera anchor while the ship entity is gone. Reuse the existing
`surface_frame(up, heading)` so the framing matches the in-flight camera and
does not roll (per the orbit-rotation retro / AGENTS.md gotcha -- never a bare
`from_rotation_arc`).

## Steps

- [x] Add a `CrashSite` resource holding an `Option` of the crash world
      position and the hull's forward heading at the moment of destruction
      (derive `Resource, Default`); register it with `init_resource`.
- [x] Set the crash site in BOTH ship-destruction paths, capturing the ship's
      `transform.translation` and `transform.rotation * Vec3::NEG_Z`:
      the hard-terrain crash branch (the `ExplodeMesh` insert around line 1995)
      and `on_ship_destroyed` (integrity depleted). Do NOT set it on a soft
      landing or on an asteroid shatter (that is not the ship).
- [x] Clear the crash site (`None`) at run start (`start_run` / OnEnter
      Playing) and on leaving the result screen (OnExit Result), so a fresh run
      and the menu return to the spawn vantage.
- [x] In `drive_chase_camera`, when there is no `Ship`, anchor to the crash
      site if one is recorded (position + heading via `surface_frame`), else
      keep the `ship_start_pos()` fallback. Update the doc comments on
      `drive_chase_camera` / `ship_start_pos` to describe the crash-site anchor.
- [x] Verify: `cargo fmt`, `cargo clippy --all-targets`, `cargo test --examples`,
      `./scripts/check-ascii.sh`, and a headless run
      (`BCS_AUTOPILOT=1 cargo run --example 08_dropzone --features debug` under
      timeout) that reaches `autopilot: cycle complete, no panic`. All done.
      A live crash screenshot was attempted (throwaway edit cutting the
      autopilot thrust so the ship free-falls) but abandoned -- the harness
      window could not be captured reliably this session. Behavior is instead
      covered by the pure `camera_anchor` unit test; see the doc note.

## Notes

- The result UI is a transparent centered text overlay, so the 3D scene (the
  explosion) shows through behind it -- no UI change needed.
- The autopilot forces `Playing -> Result` on a timer without an actual crash,
  so it will NOT exercise the crash-site anchor (no `CrashSite` set); it only
  proves no panic. A small headless unit test or a manual crash confirms the
  anchor; note the autopilot's blindness here.
