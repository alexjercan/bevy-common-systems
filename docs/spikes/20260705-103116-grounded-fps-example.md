# Spike: A grounded first-person shooter example (`14_breach`)

- DATE: 20260705-103116
- STATUS: RECOMMENDED
- TAGS: spike, examples, games, fps, camera, physics, roadmap

## Question

What is the smallest first-person shooter we should add to `examples/` that fills
the crate's first-person viewpoint gap, and what does it actually cost? The user
picked a **grounded, Doom-like** FPS (walk on a floor, gravity, collide-and-slide,
always-on mouse-look, hitscan) over a free-fly or a stationary-gallery variant. A
good answer states exactly which parts are net-new game code vs crate reuse, names
the controller/shooting approach, and gives enough shape that `/plan` can expand it
without re-deciding the feel -- plus an honest read on the touch/wasm compromise
and the crate harvest this seeds.

## Context

The gallery (`examples/01`..`13`) has no first-person *game*: `camera/wasd` +
`helpers/wasd` appear only in the `01/02/04/05` tech demos. Two prior spikes
deferred a first-person example as "lowest payoff" precisely because `camera/wasd`
was assumed to be a usable FP camera. A grounded survey of the code
(`docs/spikes/` notwithstanding) shows it is not:

- **`camera/wasd` is a free-fly spectator camera.** `WASDCamera` has only
  `look_sensitivity` / `wasd_sensitivity`; `WASDCameraInput` is `{ pan: Vec2
  (mouse delta), wasd: Vec2 (strafe/forward), vertical: f32 }`. It *accumulates*
  pan into yaw/pitch and integrates `wasd`/`vertical` straight into
  `Transform.translation` in `PostUpdate` -- with **no gravity, no ground, no
  collision, no pitch clamp, and no movement smoothing** (the source says
  smoothing is "for the future"). `helpers/wasd` binds look to **RMB-drag** (look
  only while right mouse is held) with **no cursor grab / pointer lock** anywhere
  in the crate.
- **No raycast / `SpatialQuery` anywhere.** avian3d ships `SpatialQuery` /
  `RayCaster`, but zero uses in `src/` or `examples/`. Hitscan is net-new.
- **No character controller / collide-and-slide anywhere.** `physics/` ships only
  `pd_controller` (attitude torque) + a radial-gravity recipe. avian usage in
  examples is dynamic bodies (`08_dropzone` lander) and kinematic bodies driven by
  `LinearVelocity` (`10_asteroids`), never a kinematic walk controller.

So a grounded FPS's core -- walk + gravity + collide-and-slide against level
geometry, always-on mouse-look with a grabbed cursor, and hitscan -- is **all
net-new game code**. The bundled WASD camera does not provide it and, used as-is,
would fight a physics controller (it owns the whole `Transform`).

What *is* reusable: `health` (player + enemies), `mesh/builder` (blocky level +
enemy shapes) and `mesh/explode` (enemy gib on death), `ui/status` (health/ammo
HUD), `ui/menu`, `audio` (`SfxPlugin` + `SoundBank`, reusing placeholder wavs),
`feedback/flash` (enemy hit-flash) and `feedback/screen_flash` (damage vignette),
`camera/shake` (recoil / hit kick), `camera/post` (bloom on muzzle flash /
enemies), `ui/touchpad` primitives (`stick_deflection`, `button_grid_at`,
reveal-on-touch) to back virtual sticks, `input/state`, and `persist` +
`scoring/high_score` (best wave/score). This is the same juice/scaffolding kit
every example after `05` leans on.

## Options considered

The user selected the flavor (grounded); these are the *implementation* forks that
remain, which is where the real decision now sits.

### Controller: game-local FP controller vs extend `camera/wasd`

- **Game-local `FirstPersonController` (recommended).** Build the walk controller
  inside `14_breach`: a kinematic avian body (or a transform + manual
  shapecast-based collide-and-slide), gravity, ground check, and yaw/pitch from
  an always-on grabbed mouse with a pitch clamp. The camera is a child at eye
  height. `camera/wasd` is **not** used (it is free-fly and owns the transform).
  Pros: unblocked, correct feel, and it becomes the concrete reference for a future
  harvest. Cons: it means the FP example does not literally headline `camera/wasd`
  -- but that module was never game-ready, and the honest story is "the game needs
  a controller the crate lacks", which is exactly how the crate grows.
- **Extend `camera/wasd` to a grounded mode.** Add optional gravity + ground +
  always-on look + cursor grab + pitch clamp to `camera/wasd`. Pros: the example
  would headline an upgraded crate module. Cons: collide-and-slide is inherently
  level-specific (needs the game's colliders), so it cannot live cleanly in a
  camera module; this bakes game concerns into the camera and is the wrong
  altitude. Rejected for the MVP; a *subset* (always-on look + cursor grab + pitch
  clamp) is a good later harvest (see Next steps).

### Shooting: hitscan raycast vs projectiles

- **Hitscan via avian `SpatialQuery::cast_ray` (recommended).** Fire a ray from
  the camera along its forward; the first enemy collider hit takes damage
  (`HealthApplyDamage`), spawn a brief tracer + muzzle flash, `camera/shake` a
  kick. This is the crate's **first `SpatialQuery` demo** -- a genuinely new avian
  capability shown, which adds technical value beyond "first-person". Cons: none
  material; instant-hit is the classic FPS default.
- **Projectiles (asteroids-style kinematic bullets).** Already demoed by
  `10_asteroids`; reusing it here would add nothing new and is slower/heavier for a
  hitscan gun. Rejected (keep as an option for a rocket/grenade later).

### Level: open arena vs rooms-and-corridors

- **Small blocky arena with a few cover blocks (recommended for MVP).** A floor +
  perimeter walls + a handful of cover cuboids, all static avian colliders from
  `mesh/builder`/primitives. Enough to exercise collide-and-slide and line-of-sight
  without a full level pipeline. Enemies path directly toward the player (open
  sightlines keep AI trivial).
- **Rooms + corridors (Doom-like maze).** More level and navigation work (pathing
  around walls) for little extra module payoff. Defer; the arena proves the
  controller and shooting first.

### Do nothing

Cost: the first-person genre stays unrepresented, `camera/wasd` stays a tech-demo
curiosity, avian `SpatialQuery` stays undemoed, and the crate never grows a FP
character controller. The user explicitly asked for this, so deferring has low
upside.

## Recommendation

Build **`examples/14_breach`** (name tunable): a grounded, Doom-like first-person
arena shooter.

### Shape (first cut)

- **Player / controller (net-new, the technical core).** A game-local
  `FirstPersonController`: a kinematic-ish body with gravity and a ground check;
  WASD gives a horizontal move intent that is resolved by **collide-and-slide**
  against the level colliders (avian shapecast, or move-then-depenetrate); the
  `Camera3d` is a child at eye height. Look is **always-on** yaw/pitch from mouse
  motion with the **cursor grabbed** (`CursorGrabMode::Locked`, released on Escape
  / menu / focus loss) and a **pitch clamp** (+/- ~89deg). This is the piece the
  crate cannot supply today.
- **Gun (net-new: first avian raycast).** LMB (with a fire cooldown via
  `time/cooldown`) casts a ray with `SpatialQuery::cast_ray` from the camera
  forward; first enemy hit takes `HealthApplyDamage`, gets a `feedback/flash`, and
  on death `mesh/explode` gibs it. Muzzle flash (bloom via `camera/post`), a short
  tracer (`helpers/temp`), a recoil `camera/shake`, and a gunshot `SfxPlugin`
  one-shot. Optional ammo + reload.
- **Enemies.** Capsule/octahedron shapes with `Health`, spawned at the arena edge
  in ramping waves; they path toward the player and deal contact damage (drains
  player `Health` -> `HealthApplyDamage` -> `HealthZeroMarker` ends the run). A
  hit spikes the `feedback/screen_flash` damage vignette.
- **Level.** Floor + perimeter walls + a few cover cuboids as static avian
  colliders; `camera/skybox` optional for backdrop.
- **HUD / shape.** `ui/status` health + ammo + wave/score; `ui/menu`
  menu/game-over; `persist` + `HighScore` best wave; `input/state` for Escape;
  `SfxPlugin` throughout. Follows the `06_fruitninja` shape.
- **Touch (the charter compromise).** Dual virtual sticks via `ui/touchpad`
  `stick_deflection` (left = move, right = look) plus a fire button
  (`button_grid_at`), gated on `RevealOnTouch`. Honestly clunky -- an FPS is the
  hardest genre for touch -- but it keeps the wasm gallery build touch-*enterable*.
  The desktop mouse+keyboard path is the primary one.

### What this example is really the headline of

Not `camera/wasd` (it is superseded here), but: **the first-person viewpoint as a
real game**, the crate's **first avian `SpatialQuery`/raycast** use, and a **new
first-person character controller** (collide-and-slide + grabbed-mouse look) that
is the reference for a harvest.

## Open questions

- **Cursor grab on wasm.** Pointer lock needs a user gesture and can behave
  differently across browsers; verify the grab/release flow in the trunk build
  (grab on click-to-start, release on Escape / pointer-lock-exit). May need a
  small wasm-specific path.
- **Collide-and-slide approach.** Kinematic avian body with manual
  shapecast-and-slide vs a dynamic body with frozen rotation vs transform +
  depenetration query. Pick at implementation with a boot test; the arena is
  simple enough that any works. Note it as the main tuning risk.
- **Touch viability.** Dual-stick FPS on a phone is marginal; decide during impl
  whether the wasm build ships the touch controls or is labelled desktop-first in
  the gallery blurb. Do not over-invest in touch feel.
- **Harvest boundary.** How much of the controller (look+grab+pitch-clamp vs the
  whole walk+collide-and-slide) should become a crate module, and does any of it
  belong as a `camera/wasd` grounded mode? Deferred to the follow-up.

## Next steps

Direction-level tasks seeded for `/plan` to break into steps:

- tatr 20260705-103236: build `examples/14_breach` -- a grounded first-person arena
  shooter. Game-local `FirstPersonController` (walk + gravity + collide-and-slide +
  grabbed-mouse always-on look + pitch clamp), hitscan gun via avian
  `SpatialQuery::cast_ray` (the crate's first raycast), waves of pathing enemies
  with `health` + `mesh/explode`, blocky arena, `ui/status` HUD, `feedback` hit-
  flash + damage vignette, `camera/shake` + `camera/post` juice, `ui/menu`,
  `audio`, `persist`+`HighScore`, `ui/touchpad` dual-stick touch, wasm build.
  Follows the `06_fruitninja` shape.
- tatr 20260705-103238: follow-up -- evaluate harvesting from `14_breach`: a reusable
  first-person character controller (walk/gravity/collide-and-slide), and/or
  `camera/wasd` upgrades (optional always-on look, a cursor-grab helper, pitch
  clamp), and/or a hitscan/`SpatialQuery` helper. Depends on the MVP shipping so
  there is a concrete reference. Do NOT block the MVP on any crate abstraction.
