# Spike: make 08_dropzone more fun + mobile-playable

- STATUS: OPEN
- PRIORITY: 5
- TAGS: spike,dropzone,research

## Goal

`examples/08_dropzone.rs` is a solid physics demo (PD attitude control, radial
gravity, trimesh planet, crash-by-explode) but it is thin as a *game*: one ship,
one landing pad-less planet, land-slow-and-upright-once, score, done. This spike
is research only -- no gameplay code yet. It answers two questions and leaves
behind a ranked, codebase-grounded design menu that a follow-up implementation
task (or several) can pull from:

1. How do we make the landing game more FUN (more mechanics, more replay, more
   moment-to-moment decisions) without breaking the "small, copy-pastable,
   crate-demo" spirit of the example?
2. How do we make it PLAYABLE ON MOBILE / touch (wasm showcase runs in a phone
   browser today with keyboard-only controls, i.e. unplayable on a phone)?
   Thrust-on-tap is easy; leaning the ship is the hard part.

Deliverable of the spike: this document (the ranked menu + recommendations
below) plus a short "recommended first slice" the team can turn into a task.

## Current baseline (what exists today, for grounding)

From `examples/08_dropzone.rs` as of this branch:

- Controls: Space/ArrowUp = thrust (boolean, along ship local +Y); W/S = pitch
  lean, A/D = roll lean. Lean is an absolute *target attitude* offset (max
  `MAX_LEAN` 0.45 rad) that eases toward the steered target (`LEAN_RATE` 2.5) and
  self-levels on release (`LEAN_DECAY` 4.0). A PD controller (`PD_FREQUENCY` 2.2,
  `PD_DAMPING` 1.0, `PD_MAX_TORQUE` 4000) torques the avian body toward it.
- Flight budget: `START_FUEL` 100, `FUEL_BURN` 14/s, `GRAVITY` 5.5,
  `THRUST_ACCEL` 13. Play-tested winnable with ~40% fuel to spare.
- Win/lose: touchdown with speed <= `LAND_SPEED_MAX` 4.5 m/s and tilt <=
  `LAND_TILT_MAX` 0.35 rad scores; otherwise the hull explodes
  (`mesh/explode`). Game ends on FIRST contact -- there is no post-landing sim.
- Scoring: `landing_score(fuel, speed, tilt)` = `100 + fuel*3 + (4.5-speed)*40 +
  (0.35-tilt)*200`. So fuel and being upright dominate; there is no positional /
  target component at all.
- States: Menu / Playing / Result. One planet, ship spawns near the +Y pole at
  `START_ALTITUDE` 22. No landing target, no pickups, no obstacles, no music.

Two structural facts shape everything below:

- "Game ends on first contact" means anything that should happen *after* landing
  (multi-leg refuel runs, taxiing) needs a state-machine change, not just a new
  entity. Cheaper mechanics avoid touching this.
- Scoring is a pure function of `(fuel, speed, tilt)`. Adding a *positional*
  term (distance, target zone, pickups collected) is a tiny, well-isolated
  change -- the highest fun-per-line lever we have.

---

## Part 1 -- Making it more FUN

Ideas are grouped by how much they cost to build and rated on Fun (player value)
vs Cost (code + tuning + risk), plus which crate systems they exercise (this is
a crate demo, so "shows off more of the library" is a real bonus).

### Tier A -- cheap, high leverage (do these first)

**A1. Landing target(s) with a positional score.** Mark one (or a few) surface
spots as pads/beacons. Score scales with how close the touchdown is to the
nearest target -- `proximity_bonus = (radius - dist).max(0) * k`. This turns
"land anywhere, softly" into "land softly *there*", which is the single biggest
missing decision. Reuses the existing glowing-marker look from `07_orbit`.
- Fun: high. Cost: low (a target entity + one extra scoring term + a HUD hint).
- Note: solves "landing further away = more points" but INVERTED and better: the
  user's "further = more" rewards raw distance, which perversely rewards flying
  off to nowhere. "Closer to a designated hard-to-reach target = more" gives the
  same "go far" thrill with an actual goal. Can still layer a difficulty scalar:
  targets placed further from spawn are worth a bigger base multiplier.

**A2. Fuel pickups (fuel cans).** Floating fuel cells on the descent path;
flying through one adds fuel (and enables a longer, riskier route). Detect with
an avian sensor collider or a simple distance check; on pickup add to `Fuel`,
play a sound (`pickup` already exists in the shared set), pop a "+FUEL" like
`07_orbit`'s "+N". Directly the user's fuel-can idea.
- Fun: high (creates a risk/reward routing choice: detour for fuel vs direct
  descent). Cost: low-medium (spawn set + pickup detection + HUD popup). Reuses
  `helpers/temp`, the `ui` popup pattern, `audio`.
- Design lever: place cans slightly off the efficient line so grabbing them
  costs time/altitude control -- that is what makes them a decision, not free
  candy.

**A3. Time / efficiency pressure as an explicit score term.** The fuel bonus
already rewards efficiency implicitly; making a visible timer or a "descent
score" that ticks down encourages committing to a line instead of hovering.
Nearly free (one resource + HUD line via `ui/status`).
- Fun: medium. Cost: very low.

**A4. Juice pass (feel, not mechanics).** The `07_orbit` polish cycle
(`feature/07-orbit-polish`) is the template: screen-relative popups, a landing
"stuck!" flash, camera punch on touchdown, thruster bloom ramp with throttle,
dust/impact particles on contact (reuse the fragment system at low intensity for
a soft landing too). No new rules, big perceived-fun gain.
- Fun: medium-high (perception). Cost: low-medium. Reuses `camera/post`,
  `mesh/explode`, the `07_orbit` juice patterns.

### Tier B -- medium cost, adds depth

**B1. Obstacles / hazards.** The user's "things you can hit". Options ranked by
cost:
- *Static terrain hazards*: spires/arches on the planet the trimesh already
  supports; crank `TERRAIN_AMPLITUDE` locally or add rock meshes with colliders.
  Hitting them at speed = crash (existing explode path). Cheap, uses existing
  collision classification.
- *Moving hazards*: drifting debris/asteroids using the `transform/*` orbit
  family (exactly what `07_orbit` does with `RandomSphereOrbit`). Medium cost.
- *No-fly zones / radiation belts*: a region that drains fuel or health while
  you are in it (ties in `HealthPlugin`). Medium cost, strong "find the gap"
  gameplay.
- Fun: high. Cost: medium. Risk: physics tuning (fast collisions -- the doc
  notes CCD was not needed at current speeds; obstacles could change that).

**B2. Multi-leg mission (refuel-and-continue).** Change "ends on first contact"
to "land, refuel/score, take off again, land on the next pad". Turns a
20-second run into a short course. This is the biggest FUN multiplier but needs
the state machine reworked (a `Landed` sub-state, takeoff detection, per-pad
scoring). Highest cost in this tier.
- Fun: very high. Cost: high (touches the Playing/Result flow, not just
  entities). Good candidate for its own dedicated task, not the first slice.

**B3. Cargo / weight mechanic.** Pick up cargo at one pad, deliver to another;
cargo adds mass (avian mass already drives inertia, though thrust/gravity use
mass-independent acceleration today, so this needs a deliberate mass coupling).
Ties into the PD controller feel (heavier = more sluggish attitude).
- Fun: medium-high. Cost: medium-high. Nice PD-controller showcase.

**B4. Wind / lateral force gusts.** A time-varying `ConstantLinearAcceleration`
tangential to the surface that the player must fight with lean. Trivial to add
(one more acceleration component, already the pattern for gravity/thrust), adds
real skill expression, and demos that the force-composition approach scales.
- Fun: medium-high. Cost: low-medium. Strong showcase-per-line.

### Tier C -- bigger, "if this becomes its own game"

- **C1. Multiple planets / level progression** (like `07_orbit`'s difficulty
  ramp): each level a new noise seed, tighter fuel, more hazards, smaller pads.
- **C2. Leaderboard / combo-style scoring** across a run of landings.
- **C3. Ship upgrades** between runs (more fuel, stronger thrusters, better PD
  tune) -- meta-progression.
- **C4. Two-phase flight**: orbital insertion then landing. Probably out of
  scope for a crate example.

### Recommendation for FUN

Ship Tier A as one "make it a game" slice: **landing target + positional score
(A1), fuel pickups (A2), and a juice pass (A4)**. That trio adds a real goal, a
routing decision, and game-feel with almost no change to the physics/state
machine, and it exercises more of the crate (`ui`, `audio`, `helpers/temp`, the
orbit markers, `camera/post`). Then B1 (obstacles) and B4 (wind) as a second
slice, and B2 (multi-leg) only if the example is being promoted into a fuller
game. Distance-for-points from the prompt is folded into A1 as "reward reaching
a far/hard target" rather than raw distance, which avoids the fly-off-to-nowhere
degenerate.

---

## Part 2 -- Making it MOBILE / touch playable

### The core insight

Our lean is a **target attitude**, not a rate: W/S/A/D set an absolute lean the
PD controller chases, and releasing self-levels (`LEAN_DECAY`). So the touch
scheme should express an *absolute, self-centering magnitude* (deflection = lean
amount), NOT a rate ("hold to keep tilting further"). This matters: research on
tilt/steering (Teather and MacKenzie, York U) finds **position-control** (input
magnitude -> target directly) is ~16% faster than **velocity-control** (input ->
rate of change) at equal accuracy, because position-control lets the player use
proprioception and rest instead of watching constantly. Our PD model is already
position-control, so the input mapping should match it. (Players often *say*
they prefer rate control even when they perform worse -- so offer it only as an
opt-in, never the default.)

Thrust is trivial on touch (boolean = tap/hold a zone); the whole problem is
lean. The candidate schemes, ranked for our model + a wasm/mobile-browser build:

### Ranked options

**1. (Recommended default) Two-thumb layout: left thrust hold-zone + right-side
floating stick / origin-drag for lean, deflection-to-position mapping.**
- The steer touch captures its origin on touch-down; the finger offset
  `(dx, dy)` from that origin maps directly to `(lean_roll, lean_pitch)` target,
  magnitude clamped to a max radius = `MAX_LEAN`. Lift finger -> target goes to
  level (exactly today's decay behaviour). This *is* a 2-axis absolute
  controller and maps 1:1 onto our existing `ShipInput.lean_pitch/lean_roll`.
- Use a **floating/draggable** stick (re-centers on first touch, slides at the
  extents) so the thumb never runs out of room during a tense flare; draw a
  ghost ring at the origin for discoverability + feedback even in the drag
  variant. Add a small dead zone.
- Pure touch: no permissions, identical on iOS and Android browsers. Low
  fatigue (thumb rests in the dead zone), good precision (visible extent).
- Implementation fit: minimal. A new touch-reading system writes the same
  `ShipInput` the keyboard path writes today (`read_input`), so the physics /
  PD / scoring code is untouched. Thrust = a left-side zone; route each touch by
  the zone it *started* in, tracked by pointer id, and never re-evaluate a
  moving touch against zone boundaries (a lean drag crossing into the thrust
  zone must not fire thrust). This also cleanly solves thrust/lean conflict.
- Bevy: use `Touches` (`Res<Touches>`) for multi-touch with stable ids; it
  works under wasm. Keep desktop keyboard input as-is; add touch as an
  additional writer of `ShipInput` so both platforms share the physics path.

**2. (Opt-in mode) Device tilt / accelerometer.** Phone tilt angle -> lean
target is the most literal mapping to an absolute attitude and feels great where
available. BUT it cannot be the only scheme in a browser: on iOS Safari 13+,
`DeviceOrientationEvent.requestPermission()` must be called over HTTPS *from
inside a user gesture (a tap)* and can be denied; Android/Chrome auto-grants. So
it must be gated behind an explicit "Enable tilt controls" button and always
have a touch fallback. Also needs neutral-orientation calibration (capture the
resting pose at start, steer relative to it) and a dead zone, and it couples the
viewing angle to the control (bad when you are staring at the surface on
approach). Verdict: nice bonus mode, never the default. On wasm this needs a
small JS/`web-sys` bridge for the DeviceOrientation events -- more plumbing than
option 1.

**3. (Accessibility fallback) On-screen buttons / split zones (A/D + W/S
arrows).** The literal port of the keyboard and the most self-explanatory (zero
tutorial). But a button is binary, so it naturally gives {full-left, neutral,
full-right} unless you add a hold-duration ramp (which reintroduces a rate feel
and its attention cost). Coarse for a precision flare. Keep as a beginner /
accessibility option, not the primary scheme.

**4. (Avoid as default) Rate / velocity mappings** ("hold to keep leaning
further"). Mismatches our target-attitude abstraction, does not self-level,
costs more attention. Offer only as an optional "flight-sim" toggle if asked.

### Recommendation for MOBILE

Ship **option 1** (two-thumb: left thrust zone, right floating-stick /
origin-drag lean with deflection-to-position) as the default and only required
scheme -- it needs no permissions, maps 1:1 to the existing `ShipInput` target
model, and is a small additive system that leaves the physics untouched. Add
**option 2** (tilt) later as an opt-in toggle behind an iOS-safe permission
button, and **option 3** (buttons) as an accessibility fallback; all three can
share the same "released = level" convention and the same two-thumb layout, so
they differ only in how the steer zone reads input.

Also add a `WindowMode`/canvas note for the showcase: the `web/` build should
expose a mobile-friendly viewport and the touch HUD only when a touch device is
detected (or always, since a drawn stick is harmless on desktop). Keep desktop
keyboard controls working unchanged.

### Key sources (from the research pass)

- Teather and MacKenzie, "Position vs. Velocity Control for Tilt-Based
  Interaction" (York U): yorku.ca/mack/gi2014.html, yorku.ca/mack/ieeegem2014a.html
- MDN, "Mobile touch controls":
  developer.mozilla.org/en-US/docs/Games/Techniques/Control_mechanisms/Mobile_touch
- Aaron Bell, "Mobile touch controls from scratch in HTML5" (floating d-pad,
  dead zones): aaronbell.com/mobile-touch-controls/
- Lee Martin, iOS 13 DeviceMotion/Orientation `requestPermission()` (HTTPS +
  user-gesture requirement): leemartin.dev/how-to-request-device-motion-and-orientation-permission-in-ios-13-74fc9d6cd140
- James Hague, "Virtual Joysticks and Other Comfortably Poor Solutions"
  (no-tactile-feedback critique): prog21.dadgum.com/124.html
- Cownado, "Using the accelerometer in games" (calibration / dead zone):
  cownado.com/posts/2016/05/accelerometer.html

---

---

## Recommended first slice (the spike's conclusion)

One implementation task, small and self-contained, that makes the biggest
difference on both axes at once:

**"08_dropzone: landing target + fuel pickups + touch controls"**
- FUN: add A1 (a glowing landing target with a positional score term) and A2
  (fuel-can pickups on the descent line). Both are additive entities + one
  scoring term + reused `ui`/`audio`/`helpers/temp`; no state-machine change.
- MOBILE: add option-1 touch input (left thrust zone + right floating-stick /
  origin-drag lean) as an additional writer of the existing `ShipInput`, so the
  physics/PD/scoring path is untouched and desktop keyboard keeps working.
- Optionally fold in a light A4 juice pass (popup on pickup, camera punch on
  touchdown) if cheap.

This gives the example a real goal, a routing decision, and phone playability in
one cycle, while still reading as a small crate demo. Larger items -- obstacles
(B1), wind (B4), multi-leg refuel runs (B2), tilt/accelerometer mode -- are
follow-on slices, each its own task.

Follow-up implementation tasks filed (per user direction, split into three
focused tasks rather than one combined slice):
- `tasks/20260704-103544` -- Tier-A fun pass (all A proposals: landing pad +
  positional score, fuel pickups, optional time term, landing/crash visuals).
- `tasks/20260704-103553` -- hazards pass (Tier B obstacles + wind + asteroids +
  rough terrain, with ship structural integrity via `HealthPlugin`).
- `tasks/20260704-103517` -- mobile virtual-pad touch controls (Part 2 option 1).
The C proposals and the bigger B items (pickups-beyond-fuel, upgrades, cargo,
multi-leg refuel) are intentionally left out for now.

## Steps

- [x] Read the example + doc; capture the current controls, scoring, flight
      constants and state machine as the grounding baseline (above).
- [x] Enumerate and rank fun mechanics (Tier A/B/C), including the user's fuel
      cans, distance scoring and obstacles, plus added ideas (targets, wind,
      cargo, multi-leg, juice), with Fun/Cost and which crate systems each
      exercises.
- [x] Research mobile/touch control schemes for the lean problem; rank them for
      this game's absolute-target-attitude model and note wasm/browser
      accelerometer constraints.
- [x] Distil a "recommended first slice" and file the follow-up implementation
      task(s) (out of spike scope to implement; spike ends at the plan). Filed
      `tasks/20260704-102342`.

## Notes

- Spike only -- no gameplay code changes on this branch. Output is the design
  menu above and a follow-up task recommendation.
- Keep any eventual implementation faithful to AGENTS.md: one concern per
  slice, examples double as integration tests, run the example (not just build)
  before calling it done, keep it wasm/showcase-friendly.
