# Turning examples 01-05 into small games

Date: 2026-07-03
Task: 20260703-165138 (spike)

## Why

`examples/06_fruitninja` started life as the mesh-slicer demo and grew into a
real little game: main menu, playing and game-over states, score and combo UI,
a blade trail, bombs wired through `HealthPlugin`, one-shot sounds through
`SfxPlugin`, and a wasm build in the web gallery. Along the way it became the
best single showcase of the crate, because a game naturally pulls several
modules together instead of demoing one in isolation.

The other numbered examples (01-05) are still single-module tech demos. This
doc asks, for each one: what small game could it become, which crate modules
would that game exercise (ideally the ones 06 leaves untouched), what is the
crate missing to build it, and is it worth doing.

The crate's mission is "small, composable, game-agnostic building blocks that
almost every game needs." The best game candidate is therefore not the most
fun in a vacuum -- it is the one that puts the most currently-undemoed modules
under real gameplay load, because that is what shakes out API gaps and gives
the gallery something to show.

## What 06 already covers

So we can aim the new games at the gaps, here is what fruit ninja already
exercises:

- `mesh/explode` + `mesh/builder` (slicing octahedra into fragments)
- `health` (bombs deal lethal damage)
- `ui/status` (indirectly; the HUD is custom but the pattern is shared)
- `audio` (`SfxPlugin` one-shot sounds)
- `helpers/temp`, `helpers/despawn` (fragment cleanup)
- Bevy states, wasm build, the web gallery pipeline

Modules 06 does *not* meaningfully touch, and that we want a game to cover:

- `camera/chase`, `camera/skybox`, `camera/wasd` (+ `helpers/wasd`)
- `camera/post` (bloom / tonemapping)
- all of `transform/*` (the sphere-orbit and rotation drivers)
- `physics/pd_controller`
- `meth/sphere`, `meth/lerp`
- `modding/events` (the whole event-bus / handler system)

## Per-example concepts

### 01_sphere -> "Orbit Runner" (surface dodge game)

The demo is one octahedron sphere plus a free WASD camera. The obvious game is
to put something *on* the sphere's surface and move it with the orbit drivers.

Concept: you pilot a marker that rides the surface of a planet-sized sphere,
steering with `directional_sphere_orbit` (input is a direction vector, output
is a position on the surface). Obstacles and pickups wander the same surface
via `random_sphere_orbit`. A `ChaseCamera` follows the player from behind and
above, using `LerpSnap` smoothing. Survive / collect for score; a hit ends the
run.

- Modules exercised: `transform/directional_sphere_orbit`,
  `transform/random_sphere_orbit`, `camera/chase`, `meth/sphere`, `meth/lerp`,
  `mesh/builder`, `health` (the hit), `audio`, `ui/status`.
- Gaps: none blocking. The orbit drivers output a transform on the sphere but
  the player still needs an "up = surface normal" orientation; `meth/sphere`
  has the conversions, so this is game code, not a crate gap. A
  `SkyboxPlugin` backdrop is a cheap bonus.
- Effort / payoff: MEDIUM effort, HIGH payoff. This single game lights up the
  entire `transform/*` family and the chase camera -- the biggest undemoed
  surface area in the crate -- and it is visually clear in a gallery thumbnail.

Recommendation: STRONGEST candidate. Build this next.

### 02_planet -> "Dropzone" (land on the procedural planet)

The demo is the noise-displaced planet. The planet already has real terrain
relief, so the game is a landing / low-altitude challenge above it.

Concept: a small ship hovers over the noise planet. You rotate it toward a
target orientation and the `PDControllerPlugin` computes the avian3d torque to
get there (this is exactly what the PD controller is for, and nothing demos it
today). Thrust down/around to touch down softly on flat-enough terrain for
points; hit a slope too fast and you break apart (reuse `mesh/explode`). A
`SkyboxPlugin` starfield sells the "in orbit" feel, and `camera/post` bloom
makes the thrusters glow.

- Modules exercised: `physics/pd_controller` (the headline), `camera/skybox`,
  `camera/post`, `camera/chase`, `mesh/builder` (+ noise), `mesh/explode`,
  `audio`, `ui/status` (altitude / fuel / speed gauges).
- Gaps: the planet mesh needs a collider. avian3d can build a trimesh collider
  from a `Mesh`, so this is wiring in the example, but a small crate helper
  ("collider from `TriangleMeshBuilder`") could be worth extracting if two
  games end up needing it. Worth a note, not a blocker.
- Effort / payoff: MEDIUM-HIGH effort (physics tuning is fiddly), HIGH payoff.
  It is the only credible way to demo the PD controller under gameplay, and it
  pulls in skybox + post + physics in one shot.

Recommendation: SECOND strongest. Build after Orbit Runner; it is the natural
home for the PD controller, skybox and post-processing.

### 03_modding -> "Reactor" (rules-as-gameplay incremental)

The demo is the event bus printing to the console. It is the odd one out: its
"output" is data, not a rendered scene, so the game has to make the event
system itself the thing you play with.

Concept: a small idle / incremental game where the *rules are the mod system*.
The world state (`EventWorld`) holds a few resources (energy, heat, credits).
Ticks and player clicks `fire` events. The player builds their machine by
placing `EventHandler` entities -- each is a filter (e.g. "energy >= 100") plus
an action (e.g. "convert energy to credits, add heat") -- exactly the
filter+action pairs `03_modding` already shows. Progress = composing handlers
into a stable, escalating loop without letting heat run away.

- Modules exercised: `modding/events` end to end (the only game that would),
  `ui/status` (the gauges are a perfect fit for the status bar's
  value_fn/color_fn thresholds), `audio`.
- Gaps: to be a *game* rather than a demo you want handlers to be
  data-authored (JSON), which is precisely the serde-friendly direction the
  module was built for but does not yet ship -- there is no built-in registry
  mapping event-name/filter/action strings back to trait objects. That
  registry is a real, well-scoped `feature` candidate and would make the whole
  modding module dramatically more useful. Also see the known gotcha: the
  `EventKind` default `event_info` path is stale (`tasks/20260703-095509`).
- Effort / payoff: MEDIUM effort for a thin version, but HIGH design risk (it
  is the least visual, hardest to make legibly fun). Payoff is high *if* we
  also build the JSON registry, because that turns modding from "interesting
  primitive" into "you can actually mod a game with this."

Recommendation: DEFER the game, but file the JSON handler-registry feature now
-- it is valuable independent of any game and unblocks a real modding demo.

### 04_status_item -> "Overload" (dashboard survival)

The demo is a screen-corner status bar with FPS, version and a shell-command
item. The status bar is HUD, not a game core, but a game *about reading gauges*
makes it the star.

Concept: you run a failing machine (reactor, ship, life-support -- pick a
skin). Several gauges climb and drift on their own; each is a `status_bar_item`
whose `color_fn` goes green -> amber -> red at thresholds (already supported).
Periodic events (drive them with the `modding` bus or plain timers) push gauges
around; you press keys to vent / cool / patch and pull them back to green. Let
any gauge sit red too long and you lose. It is essentially a reaction game
whose entire display is the status bar.

- Modules exercised: `ui/status` (as the core mechanic, not decoration),
  `health` (the lose condition), `audio` (alarms), optionally `modding` for
  the event-driven upsets.
- Gaps: the status value closures run in an exclusive system every frame and
  are meant to be cheap reads; a game would want to read a game-state resource
  from them, which already works. No blocking gap. A minor ergonomic want: the
  status bar is output-only, so all input/logic is game code -- fine.
- Effort / payoff: LOW effort, MEDIUM payoff. Cheapest game on the list and a
  genuinely different genre (no 3D scene needed), but it demos the fewest new
  modules and overlaps 06 on health/audio.

Recommendation: GOOD warm-up / low-risk build if we want a quick second game,
or fold its gauge idea into "Dropzone" (fuel/altitude/hull) rather than
shipping standalone.

### 05_explode -> "Asteroids" (slice-on-hit shooter)

The demo is the mesh slicer. 06 already turned slicing into a game (fruit
ninja), so a second slicer game has to be a clearly different genre to be worth
it.

Concept: 3D asteroids. A ship (WASD or chase camera) fires at drifting
octahedron asteroids; a hit inserts `ExplodeMesh` and the asteroid slices into
fragments that keep drifting as new, smaller hazards (avian physics carries the
fragments, unlike 06 where they just despawn). Clear the field without getting
hit; `HealthPlugin` on the ship. `camera/post` bloom on the shots.

- Modules exercised: `mesh/explode` (but 06 already does), `camera/chase` or
  `camera/wasd`, `camera/post`, `physics` (fragment drift as real bodies),
  `health`, `audio`, `ui/status`.
- Gaps: none blocking; it is mostly recombining existing pieces.
- Effort / payoff: MEDIUM effort, LOW-MEDIUM payoff. It overlaps 06 on the
  headline module (slicing) and the gallery would show two "slice stuff"
  games. Its only new coverage is fragments-as-physics-bodies and the cameras,
  both of which Orbit Runner / Dropzone cover better.

Recommendation: SKIP for now. Redundant with 06 on its main mechanic; the
camera/physics coverage it adds is better served by the two picks above.

## Coverage matrix

Modules each proposed game would newly exercise (X = core to the game,
. = incidental/reused, blank = not touched). "New?" marks modules no current
example (01-06) demos under gameplay.

| module                     | New? | 01 Orbit | 02 Dropzone | 03 Reactor | 04 Overload | 05 Asteroids |
|----------------------------|------|----------|-------------|------------|-------------|--------------|
| transform/sphere_orbit*    | yes  |    X     |      .      |            |             |              |
| camera/chase               | yes  |    X     |      X      |            |             |      .        |
| camera/skybox              | yes  |    .     |      X      |            |             |              |
| camera/post                | yes  |          |      X      |            |             |      X        |
| camera/wasd + helpers/wasd | .    |    .     |             |            |             |      X        |
| physics/pd_controller      | yes  |          |      X      |            |             |              |
| modding/events             | yes  |          |             |     X      |      .       |              |
| ui/status                  | .    |    X     |      X      |     X      |      X       |      X        |
| mesh/explode               | .    |          |      .      |            |             |      X        |
| health                     | .    |    X     |      X      |     .      |      X       |      X        |
| audio                      | .    |    X     |      X      |     X      |      X       |      X        |

`transform/sphere_orbit*` stands in for the whole `directional_` /
`random_sphere_orbit` family plus `meth/sphere`.

## Recommendation

Two builds, in order:

1. **07_orbit (Orbit Runner)** -- highest new-module coverage per unit effort.
   Lights up the entire `transform/*` orbit family, the chase camera and
   `meth`, none of which any example demos under gameplay today. Visually
   legible for the gallery.
2. **08_dropzone (Dropzone)** -- the only credible gameplay demo of
   `physics/pd_controller`, and it pulls in `camera/skybox` and `camera/post`
   at the same time. Higher physics-tuning risk, so it goes second.

Independently of the games, file one feature task now:

- **modding: JSON-authored handler registry** -- a registry mapping
  event-name / filter / action strings to registered trait-object
  constructors, so `EventHandler`s can be built from `serde_json` data. This is
  the missing half of the modding module's stated purpose (crossing a
  scripting boundary), it is useful with or without a game, and it is the
  precondition for a satisfying "03 Reactor" later.

Defer 03 Reactor (needs the registry to be a game, not a demo), 04 Overload
(cheap but low new coverage; fold its gauges into Dropzone), and 05 Asteroids
(redundant with 06 on slicing).

## Follow-up tasks to file

- `feature,example`: 07_orbit "Orbit Runner" -- surface-dodge game on a sphere
  using the orbit drivers + chase camera (states, sounds, wasm gallery).
- `feature,example`: 08_dropzone "Dropzone" -- land a ship on the noise planet
  with the PD controller, skybox and post-processing (states, sounds, wasm).
- `feature,modding`: JSON-authored `EventHandler` registry for the event bus.
- `idea` (backlog, low priority): 04 Overload dashboard-survival, or fold its
  gauge mechanic into Dropzone's HUD.
