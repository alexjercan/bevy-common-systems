# breach -- juice pass (hit markers, kill/combo popups, muzzle flash, low-HP pulse)

- STATUS: CLOSED
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

- Spike: tasks/20260705-132024/SPIKE.md
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

## Steps

- [x] **Hit marker.** `HitFlash(f32)` resource (seconds of marker visibility) + a
  `HitMarker` bordered box centred on the crosshair. `player_shoot` sets `HitFlash` on a
  confirmed enemy hit; `update_hit_marker` fades the border alpha from the timer. Reset
  in `start_run`.
- [x] **Muzzle flash.** In `player_shoot`, on each shot spawn a small bright emissive
  sphere at the muzzle with `TempEntity` (blooms via `camera/post`), alongside the
  existing tracer.
- [x] **Low-health pulse.** `low_health_warning` reads `PlayerHp`; while
  `is_low_health(current, max)` (pure, `< LOW_HP_FRAC`), pulse the `DamageVignette`
  `ScreenFlash` every `LOW_HP_PULSE` seconds so the screen throbs red near death.
- [x] **Enemy spawn tell.** In `spawn_wave`, at each spawn position drop a short-lived
  emissive beacon (`TempEntity`) in the archetype colour, so enemies telegraph instead
  of popping in cold.
- [x] **Tests + verify.** Unit-test the pure `is_low_health`. `cargo fmt`, `cargo clippy
  --all-targets`, `cargo test --example 14_breach`, ascii, headless `BCS_AUTOPILOT`,
  and a real run (screenshot if `$DISPLAY`, since this is a visual task). Update the
  `//!` header and AGENTS note.
