# Spike: A few new small prototype games to fill example coverage gaps

- DATE: 20260704-220530
- STATUS: RECOMMENDED
- TAGS: spike, examples, games, roadmap

## Question

Which small (~2000 line) prototype games should we add to `examples/` next?
They are not meant to be polished games -- they are proofs of concept that
double as the crate's integration tests and quickstart docs, in the same spirit
as `06_fruitninja` ... `11_overload`. A good answer is a short, prioritized set
of concrete game concepts, each chosen because it is the *canonical demo* for a
crate module that currently has none, plus enough shape (interaction loop,
which modules it wires, the fun hook) that `/plan` can expand one into steps
without re-deciding what to build.

## Context

The crate grows by building example games and promoting the reusable systems
out of them; every example after `05` follows one shape (menu/playing/game-over
`States`, `SfxPlugin` one-shots, a `ui/status` HUD, a wasm/trunk build,
touch-playable controls). Coverage today, from a sweep of `src/` against
`examples/` (grep of the public type of each module):

Well-covered modules: `audio`, `mesh/explode`, `mesh/builder`, `health`,
`ui/status`, `camera/chase`, `camera/skybox`, `camera/post`, `camera/shake`,
`ui/popup`, `feedback`, `scoring/streak`, `time/cooldown`, `tween`, `persist`,
`input/pointer` (UnifiedPointer), `ui/touchpad`, the `modding` bus, and the
`transform` sphere-orbit family (`sphere_orbit`,
`directional_sphere_orbit`, `random_sphere_orbit`).

Modules exercised by **no** example -- the real gaps:

| Module | Public API | What it is for |
| --- | --- | --- |
| `camera/project` | `pointer_on_plane`, `world_to_screen` | mouse/touch -> world point on a play plane; world entity -> screen pixel for anchoring UI |
| `transform/point_rotation` | `PointRotation` + In/Out | accumulate an orientation from mouse-delta input (mouse-look / free aim) |
| `transform/smooth_look_rotation` | `SmoothLookRotation` + Target/Out | rate-limited, optionally clamped rotate-toward-a-target-angle around an axis |

Two nuances sharpen the priority:

1. `camera/project` was *harvested* from the inline projection code in
   06/07/08/10 (see `docs/spikes/20260704-161210-input-and-projection-harvest.md`),
   but no example imports the promoted module -- they still carry their own
   `viewport_to_world` glue. So the extracted module has **never had a canonical
   demo, and has never been validated in use.** A new game that is built on it
   both showcases and battle-tests it.
2. `point_rotation` and `smooth_look_rotation` are the two "aim/track" halves of
   the transform family. The existing games only ever orbit a sphere surface;
   nothing in the examples does first-person free-aim or a turret that tracks a
   moving target. That is a whole interaction genre (aim-and-shoot) the crate
   claims to support but never demonstrates.

Also worth noting: `camera/wasd` + `helpers/wasd` only appear in `01_sphere` /
`02_planet`, which are free-fly tech demos, not games -- so there is no
first-person *game* example either, though the module itself is technically
"used".

The charter constraint: each new example should be the smallest game that makes
one gap module obviously useful, reuse the established shape, and stay
touch-playable for the wasm showcase.

## Options considered

Four concrete concepts, each anchored on a gap module. Prior art for all of
these is well-trodden (tower defense, shooting galleries, twin-stick tanks),
so the research is about fit-to-module, not novelty of genre.

### A. `12_warden` -- top-down click-defense (headline for `camera/project`)

Top-down camera looking down at a `y = 0` play plane. A central core has a
`Health` pool. Octahedron enemies spawn at the arena edge and crawl inward
toward the core. You **click/tap anywhere on the ground** -- `pointer_on_plane`
turns the pointer into the world point -- to fire a shot (or drop a mine) at
that spot; a hit inserts `ExplodeMesh` and pops a `ui/popup` "+N" anchored over
the enemy via `world_to_screen`. Core integrity, wave and score ride a
`ui/status` HUD; the core reaching zero (`HealthPlugin` -> `HealthZeroMarker`)
ends the run. Difficulty ramps enemy count/speed per wave.

- Modules: **`camera/project` (both functions, the headline)**, `input/pointer`,
  `health`, `mesh/explode`, `mesh/builder`, `ui/popup`, `ui/status`,
  `scoring/streak`, `time/cooldown` (fire cooldown), `audio`, `tween`,
  `feedback`, `camera/shake`.
- Pros: fills the single biggest, never-demoed gap and validates a harvested
  module in the process; tap == click makes it touch-native for free; reuses
  nearly the whole juice kit so most of the ~2000 lines is well-worn shape.
- Cons: top-down "click the ground" is close in feel to `10_asteroids`' pointer
  control -- must differentiate with the placement/defense loop, not the input.
- Unknowns: orthographic vs a high perspective camera for the plane pick
  (both work with `pointer_on_plane`; perspective reads better with `camera/post`
  bloom). Small.

### B. `13_turret` -- first-person turret defense (headline for `point_rotation` + `smooth_look_rotation`)

You are a **stationary** turret at the origin. Mouse movement feeds
`PointRotationInput` to swing your barrel (snappy, free 360 aim -- exactly what
`point_rotation` is for); click fires a ray/tracer along the barrel forward.
Enemies advance from all bearings, so you rotate to prioritize and shoot them
before they reach the base (base `Health`). Distant enemy "sniper" emplacements
use `SmoothLookRotation` to *slowly, visibly* track you and fire telegraphed
shots -- giving the second module a natural, contrasting home (rate-limited,
clamped tracking vs the player's instant aim). Kills `ExplodeMesh`; `ui/status`
shows base integrity/wave/score.

- Modules: **`transform/point_rotation` (player aim)** + **`transform/smooth_look_rotation`
  (enemy tracking)**, `health`, `mesh/explode`, `helpers/wasd`-style binding for
  mouse capture (or reuse the mouse-delta plumbing), `ui/status`, `audio`,
  `ui/popup`, `camera/shake`, `feedback`.
- Pros: lands BOTH remaining transform gaps in one game, and the
  instant-vs-smooth contrast is a genuinely instructive side-by-side of the two
  APIs; first-person is a viewpoint no example has.
- Cons: mouse-look wants pointer lock, which is fiddly on wasm and awkward for
  touch -- needs a touch fallback (drag-to-aim, or on-screen look pad via
  `ui/touchpad`). Aiming a ray in 3D is more finicky to tune than a plane pick.
- Unknowns: how cleanly `point_rotation`'s mouse-delta input maps to a
  wasm/touch drag; the touch story is the main risk (mitigated by `ui/touchpad`,
  already proven in 08/11).

### C. `14_tank` -- top-down twin-control tank (combined `camera/project` + `smooth_look_rotation`)

Top-down. WASD (or click-to-move) drives a tank *hull*; the *turret* is a child
that `SmoothLookRotation`s toward the mouse's world point from
`pointer_on_plane` -- so the barrel visibly lags a fast mouse flick, which is
the whole point of the rate-limited rotate-to-angle. Click fires along the
turret facing. Enemies chase; `ExplodeMesh` on kill.

- Modules: `camera/project` (aim point) + **`smooth_look_rotation` (turret lag)**,
  plus the usual kit.
- Pros: the "turret lags the crosshair" feel is the single clearest
  demonstration of `smooth_look_rotation`; combines two gap modules elegantly.
- Cons: overlaps A (plane pick) and B (smooth-look) -- building all three would
  demo `smooth_look_rotation` twice and `camera/project` twice. Redundant if A+B
  ship.
- Unknowns: none material.

### D. `15_corridors` -- first-person WASD collectathon (fills the "no FP game" gap)

A real first-person *game* on `camera/wasd` + `helpers/wasd`: walk a small
blocky arena/maze, collect pickups against a timer, dodge a wandering hazard.

- Pros: turns the free-fly tech demo into an actual game; only FP-locomotion
  example.
- Cons: `camera/wasd` is not a *gap* module (01/02 use it), so this is the
  lowest-value of the four -- it demos polish, not a never-shown capability.
  Level geometry/collision is the most net-new code for the least new-module
  payoff.
- Unknowns: collision approach (avian vs hand-rolled AABB) drives the line count.

### Do nothing

Always a candidate. Cost: `camera/project` (a shipped, harvested module) keeps
having zero validated users and zero demo, and the aim-and-shoot genre stays
unrepresented. Low upside to deferring; these are cheap, on-charter examples.

## Recommendation

Build **two** games, in this order:

1. **`12_warden`** (Option A) -- top-down click-defense. It is the canonical,
   first, validating demo of `camera/project`, the biggest never-showcased gap;
   it is touch-native; and it reuses the whole juice kit, so the risk and the
   net-new code are both low. This is the clear #1.

2. **`13_turret`** (Option B) -- first-person turret defense. It is the only
   concept that homes BOTH `point_rotation` and `smooth_look_rotation`, and the
   instant-aim-vs-smooth-track contrast teaches the two APIs against each other
   in one screen. Accept the touch/pointer-lock work as the cost of closing two
   gaps at once; `ui/touchpad` (proven in 08/11) is the mitigation.

Together these two close all three never-demoed modules. **Drop `14_tank`
(Option C)** as a redundant second demo of modules A+B already cover, and
**defer `15_corridors` (Option D)** as the lowest-payoff (its module isn't a
real gap) -- keep it as a backlog note, not a task.

Each stays within the ~2000-line budget by cloning the `06_fruitninja` shape
(states, sounds, wasm, HUD) and spending the new lines only on the one
interaction primitive being showcased.

## Open questions

- **Warden camera:** orthographic vs high-perspective for the plane pick.
  Resolve at implementation with a `ScreenshotPlugin` grab; both are supported
  by `pointer_on_plane`. Non-blocking.
- **Turret touch/pointer-lock:** does `point_rotation`'s mouse-delta input feel
  right under a wasm drag / `ui/touchpad` look pad? This is the one real risk;
  prototype the aim control first and fall back to drag-to-aim if lock is bad.
- **Shared spawner:** both games spawn waves of edge-in enemies; check whether
  the wave/spawner logic is yet another cross-game duplication worth harvesting
  (a future spike, not this one).

## Next steps

Direction-level tasks seeded for `/plan` to break into steps:

- tatr 20260704-220736: build `examples/12_warden` -- top-down click-defense, canonical demo of `camera/project`
- tatr 20260704-220719: build `examples/13_turret` -- first-person turret defense, demo of `point_rotation` + `smooth_look_rotation`
