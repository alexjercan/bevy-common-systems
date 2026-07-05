# breach -- juice pass (hit markers, kill/combo popups, muzzle flash, low-HP pulse)

- STATUS: OPEN
- PRIORITY: 50
- TAGS: spike,breach,example,juice

## Goal

A juice pass on `14_breach` to raise perceived quality using modules already in
the crate: a hit marker (crosshair tick / brief reticle flash on a confirmed
raycast hit), kill and combo popups (`ui/popup`, coordinated with the combos
task), a muzzle-flash entity (`camera/post` bloom + `helpers/temp`), a low-health
vignette pulse (`feedback/screen_flash`), and an enemy spawn tell (a brief
telegraph so enemies do not pop in cold). The base already has recoil shake, hit
kick, damage vignette, hit-flash, tracers and gibs -- this fills the gaps.

## Notes

- Spike: docs/spikes/20260705-132024-breach-fun-pass.md
- Reuse: `ui/popup`, `camera/post`, `camera/shake`, `feedback/flash`,
  `feedback/screen_flash`, `helpers/temp`. No new crate code expected.
- Watch the tracer/flash-vs-despawn race the example already hit: insert
  cosmetic side effects on an enemy BEFORE triggering lethal damage, or target a
  separate cosmetic entity, so nothing lands on a despawned entity.
- HDR emissive muzzle flash must NOT set `unlit: true` or it will not bloom;
  an entity with only child meshes needs an explicit `Visibility` (B0004).
- Some of this renders wrong silently (a background run cannot see the screen):
  if `$DISPLAY` is set, use `ScreenshotPlugin` / `scrot` to eyeball a frame; a
  state-entry screenshot only shows the initial scene, so drive with the
  autopilot and grab externally for mid-gameplay.
- Verify: `cargo clippy --all-targets`, headless run, then run for real.
