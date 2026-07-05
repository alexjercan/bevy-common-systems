# Build examples/14_breach -- grounded first-person arena shooter

- STATUS: OPEN
- PRIORITY: 80
- TAGS: spike,feature,example,fps

## Goal

Add `examples/14_breach` (name tunable): a grounded, Doom-like first-person arena
shooter, and the gallery's first first-person game. It headlines the first-person
viewpoint as a real game, the crate's first avian `SpatialQuery` raycast (hitscan),
and a new game-local first-person character controller.

Core (net-new, since `camera/wasd` is free-fly only and there is no character
controller in the crate): a `FirstPersonController` with walk + gravity + ground
check + collide-and-slide against level colliders, and always-on yaw/pitch from a
grabbed mouse (`CursorGrabMode::Locked`, released on Escape/menu) with a pitch
clamp; the `Camera3d` rides at eye height. LMB fires a hitscan ray
(`SpatialQuery::cast_ray`) on a `time/cooldown`; first enemy hit takes
`HealthApplyDamage` + `feedback/flash`, dies via `mesh/explode`; muzzle flash
(`camera/post` bloom), tracer (`helpers/temp`), recoil (`camera/shake`), gunshot
`SfxPlugin`. Waves of pathing enemies (capsule/octahedron `Health` shapes) close in
and deal contact damage (spikes `feedback/screen_flash`); player `Health` zero ends
the run. Blocky arena (floor + walls + cover) as static avian colliders. Shape:
menu/playing/game-over, `ui/status` HUD (health/ammo/wave), `ui/menu`, `audio`,
`persist`+`HighScore` (best wave), `input/state`, wasm build. Touch: dual virtual
sticks + fire button via `ui/touchpad` (clunky but touch-enterable; desktop is
primary).

Ship the controller game-local; do NOT build a crate abstraction here (that is the
follow-up 20260705-103238).

## Notes

Spike: docs/spikes/20260705-103116-grounded-fps-example.md

Stepless direction-level task -- run `/plan` before `/work`. Key facts from the
spike survey (so the implementer does not re-derive):

- `camera/wasd` is a free-fly spectator camera (accumulates `WASDCameraInput.pan`
  into yaw/pitch, integrates `wasd`/`vertical` straight into `Transform`, no
  gravity/ground/collision/pitch-clamp/smoothing) and `helpers/wasd` gates look to
  RMB-drag with NO cursor grab. It owns the whole `Transform`, so it fights a
  physics controller -- do NOT use it for the FPS; build the controller.
- No `SpatialQuery`/`RayCaster`/raycast anywhere in the crate or examples -- hitscan
  is net-new against avian's `SpatialQuery` (this is a headline: first raycast demo).
- No kinematic character controller / collide-and-slide anywhere; `physics/` is just
  `pd_controller`. avian usage to copy: `10_asteroids` kinematic bodies +
  `LinearVelocity`, `08_dropzone` dynamic body + static colliders + trimesh terrain.
- Reuse: `feedback/flash` (enemy hit-flash, needs `MeshMaterial3d<StandardMaterial>`;
  it clones the material), `feedback/screen_flash` (damage vignette -- spawn a
  persistent `despawn_on_end:false` overlay and re-insert `ScreenFlash` per hit),
  `ui/touchpad` (`stick_deflection` for virtual sticks, `button_grid_at` for the
  fire button, `RevealOnTouch` to gate touch UI), `camera/shake`, `camera/post`.
- Open unknowns to settle at impl (see spike): wasm pointer-lock flow, the
  collide-and-slide approach (kinematic shapecast vs depenetration), and how much
  touch to ship.
- Copy the Bevy 0.19 visual/HUD idioms from `11_overload`/`08_dropzone`, not memory.
  Verify with the `AutopilotPlugin`/`ScreenshotPlugin` harness; a mid-game invariant
  is better asserted as a pure-function unit test than a screenshot (state-entry
  screenshot != gameplay verification -- see the `13_glide` retro/gotcha).
