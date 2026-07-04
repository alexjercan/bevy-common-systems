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

**(Revised 20260704-2213 after user steer.)** The original recommendation was
two separate games (a top-down click-defense for `camera/project`, a
first-person turret for the two rotation modules). The user pointed out these
merge cleanly into **one** unconventional tower-defense prototype, which is a
strictly better call: a single ~2000-line game closes all three never-demoed
modules at once, avoids the two-games' redundant juice-kit boilerplate, and
carries a natural `modding` hook the separate games did not. Options A-D above
remain the research trail; this is where they converge.

### The merged game: `12_bastion` -- defend-the-core tower defense

Unconventional TD framing: instead of enemies walking a fixed lane to an exit,
a **Core** (the thing you defend, a `Health` pool) sits at the center of a
circular play plane and **enemies spawn all around the border and converge
inward** from every bearing. You earn credits from kills and spend them to
**place towers on the plane around the Core** and to **upgrade** them. An enemy
that reaches the Core damages it (`HealthApplyDamage`); Core health zero ends
the run. Waves ramp count/speed/HP.

How it homes each gap module (this is the point):

- **`camera/project` (headline).** `pointer_on_plane` turns the mouse/tap into
  the world point where a tower is placed (and shows a ghost + range ring
  there); `world_to_screen` anchors floating UI -- enemy health pips, "+N"
  credit popups on kill, the click-to-upgrade panel over a selected tower.
- **`transform/smooth_look_rotation` (tower turrets).** Each tower auto-targets
  the nearest enemy in range and its turret rotates toward it with
  `SmoothLookRotation` -- rate-limited on purpose, so a fast enemy crossing a
  tower's arc can out-slew a cheap turret until upgraded. That turn-rate is a
  real, tunable stat, not just cosmetics, which is exactly what the module is
  for.
- **`transform/point_rotation` (orbit camera).** The camera rides a rig aimed at
  the Core; mouse-drag (or A/D) feeds `PointRotationInput` to accumulate the
  rig's yaw/pitch, orbiting the view around the battlefield. This is the "camera
  with orbit and point rotation" the user described, and it gives
  `point_rotation` a genuine home without a manual-aim mechanic that would fight
  the auto-targeting towers.

Kept simple for the first cut (the "just a few towers, simple upgrades, simple
enemies" brief): 2-3 tower archetypes (e.g. fast/weak, slow/heavy, maybe a
slow-field), a one-axis upgrade per tower (damage or range or fire-rate),
one or two enemy types, hand-authored waves. Reuses the full established shape
and juice kit: menu/playing/game-over `States`, `SfxPlugin`, `ui/status` HUD
(credits/wave/Core integrity), `mesh/explode` on kills, `ui/popup`,
`camera/shake`, `feedback`, `tween`, `scoring/streak`, `time/cooldown` (per-
tower fire cadence), `input/pointer`, and a wasm/trunk build. Touch-native:
tap-to-place, drag-to-orbit.

### Follow-up: data-driven towers/enemies (the `modding` hook)

The user's stretch goal -- add new towers/enemies with different HP/damage
without recompiling -- is a real second showcase, but a different shape from the
crate's existing `modding`. Today `modding` is an **event bus** (`EventWorld` +
`EventHandler` + `registry`, demoed by 03/09); a TD wants a **stat catalog**
(`TowerSpec`/`EnemySpec`: HP, damage, range, fire-rate, turn-rate, speed, cost)
loaded from JSON and spawned by name. That is closer to how `registry` maps
name-strings to constructors than to the event bus. Recommended path: build the
catalog game-local first inside `12_bastion` (serde structs + a spawn-by-name
table), ship the game on it, then evaluate in a later spike whether it
generalizes into a crate module (a `SpecCatalog<T>` sibling to
`EventHandlerRegistry`). Do NOT block the MVP on the crate-level abstraction.

### What is dropped / deferred

- **`14_tank`** and **`15_corridors`** (Options C, D) -- still dropped/deferred
  as before; the merged game supersedes the need for a second rotation demo, and
  the FP-locomotion example remains the lowest-payoff backlog note.
- The separate `13_turret` first-person concept is **folded into** the orbit
  camera + smooth-look towers of `12_bastion`; there is no standalone turret
  game.

Line budget: the merged game is more mechanic than any single Option (placement
+ upgrades + auto-targeting + waves + orbit camera), so ~2000 lines is the
target ceiling, not a floor -- keep tower/enemy variety minimal for the first
cut and lean hard on the juice kit to stay under it.

## Open questions

- **Camera:** angled perspective looking in at the Core (reads best with
  `camera/post` bloom) vs a steeper top-down. Both support `pointer_on_plane`;
  pick with a `ScreenshotPlugin` grab at implementation. The orbit via
  `point_rotation` needs a sensible pitch clamp (`SmoothLookRotation`-style min/
  max is on that module, but `point_rotation` has none -- clamp in the game, or
  note it as a possible `point_rotation` enhancement to harvest back).
- **Placement UX under orbit:** `pointer_on_plane` picks the ground point fine
  at any camera angle, but a grazing angle makes placement imprecise. Constrain
  the orbit pitch, or snap placement to a grid/ring, to keep it usable on touch.
- **Auto-target selection:** nearest-in-range is the simple default; note if it
  feels bad (first-in-range / most-progressed toward Core are the usual
  alternatives) -- a tuning question, not a blocker.
- **Data-catalog generality:** does the `TowerSpec`/`EnemySpec` catalog want to
  become a crate module, and can it reuse anything from `modding/registry`?
  Deferred to a follow-up spike after the game-local version exists.
- **Shared wave-spawner:** edge-in wave spawning is likely another cross-game
  duplication (06/07/08/10) worth harvesting later; note it, do not build it
  here.

## Next steps

Direction-level tasks seeded for `/plan` to break into steps:

- tatr 20260704-220736: build `examples/12_bastion` -- defend-the-core tower defense; canonical demo of `camera/project` (placement + UI anchoring), with `smooth_look_rotation` tower turrets and a `point_rotation` orbit camera. MVP: a few towers, simple upgrades, simple enemies, game-local tower/enemy stat catalog.
- tatr 20260704-220719: follow-up -- data-driven tower/enemy catalog for `12_bastion` and evaluate promoting a `SpecCatalog<T>` module (the `modding` hook). Depends on the MVP shipping first.
