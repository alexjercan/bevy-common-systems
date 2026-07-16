# Spike: What else should we harvest from the games -- input, projection, progression?

- DATE: 20260704-161210
- STATUS: RECOMMENDED
- TAGS: spike, features, roadmap, input, harvest

## Question

The crate grows by building example games and promoting the reusable systems
out of them (that is how `audio`, `mesh/explode`, the whole `transform` family
and `ui/status` were born). A prior spike
(`tasks/20260704-134035/SPIKE.md`) catalogued the
gameplay-*juice* duplication (camera shake, popup, flash, tween, persist,
spawner, menu/states) and its Wave 1 has shipped (`camera/shake`, `ui/popup`,
`feedback`). The fuzzy request now: with those known, what *else* do the six
game examples (06-11) still hand-roll in game after game that the crate should
own? A good answer is a prioritized set of *new* library candidates -- ones the
prior spike did not catalogue -- each backed by cross-game evidence and each a
fit for the "small, composable, game-agnostic block, not framework machinery"
charter.

## Context

A fresh sweep of `examples/06`-`11` (evidence with file:line in the findings
below) surfaces a second cluster of duplication that the juice-kit spike did not
cover. It centers on **player input and the camera-to-screen glue around it**,
plus two smaller gameplay primitives. Crucially, the input cluster is not
speculative: mobile touch support alone already has FOUR design docs
(`docs/2026-07-0{3,4}-*touch*.md`) and THREE retros
(`tasks/<id>/RETRO.md (the *touch* tasks)`), each re-deriving the same pattern in a different
game, and the asteroids `Pointer`, fruitninja `Pointer` and dropzone
`TouchControl` are three independent re-implementations of "collapse
mouse+touch+keys into one thing this frame".

| New pattern (not in the prior spike) | Games that hand-roll it |
| --- | --- |
| Unified pointer resource (mouse + touch + cursor -> one abstraction) | 06, 08, 10 |
| Reveal-on-first-touch on-screen control pad (button grid / virtual stick) | 08, 09, 11 |
| Touch tap navigating menu / result (`Touches::any_just_pressed`) | 06, 08, 09, 11 |
| Screen<->world projection (`viewport_to_world` plane pick; `world_to_viewport`) | 06, 07, 08, 10 |
| Score + escalating combo/streak resource (decay window) | 06, 07 (07 says "modelled on 06") |
| Difficulty ramp / progression scalar over time | 06, 07, 08, 09, 10, 11 (all) |
| Radial ("point") gravity via `normalize_or` | 08, 10 |

Two of these are byte-for-byte copies, the strongest possible harvest signal.
`fn pointer_on_play_plane` (viewport ray -> intersect a Z-plane) is duplicated
verbatim in `06_fruitninja.rs:1515` and `10_asteroids.rs:1533`. And `07_orbit`'s
`Streak` carries the comment "Modelled on `06_fruitninja`'s `Combo`"
(`07_orbit.rs:288`) -- the game author literally copied the mechanic across.

Charter note: touch-pad *layouts* are game-specific and a full "input framework"
would be exactly the machinery AGENTS.md warns against. But the *primitives*
underneath every game's copy are game-agnostic: a unified pointer resource, a
reveal-on-first-touch visibility gate, a pure grid/stick hit-test function, and
the projection math. Harvest the primitives, leave the layout in the game -- the
same line `ui/status` walks (it owns `status_bar_item`, the game owns which
items).

## Options considered

Each candidate judged on evidence (games affected), reuse, risk, and charter
fit. Grouped into a recommended harvest wave, a medium-value wave, and
document-don't-build.

### A. Input & projection harvest (highest evidence, headline)

- **`input/pointer` -- unified `Pointer` resource.** One resource resolved each
  frame in `PreUpdate` that collapses `Touches` + `ButtonInput<MouseButton>` +
  `window.cursor_position()` into `{ screen_pos, pressed, just_pressed }`, an
  active touch winning over the cursor. Evidence: three independent copies
  (`10_asteroids.rs:293`, `06_fruitninja.rs:283`, `08_dropzone.rs:307`), plus
  the shared `active_pointer_pos(touch, cursor)` helper
  (`06_fruitninja.rs:1504`, unit-tested; inlined in asteroids `:325`). Pros: 3
  games, pure resolve system, no ordering hazard, obviously game-agnostic. Cons:
  fruitninja routes touch through `bevy_enhanced_input`
  (`TouchInputId(CustomInput)`), so the module must offer a plain `Touches` path
  and not force the enhanced-input dependency on users. Unknowns: whether to also
  expose per-touch tracking (drag ids) or keep it to a single logical pointer.
- **`camera/project` -- screen<->world helpers.** Two pure-ish helpers over a
  `Camera` + `GlobalTransform`: `pointer_on_plane(camera, gt, viewport_pos,
  plane) -> Option<Vec3>` (the byte-for-byte-duplicated `viewport_to_world` +
  `InfinitePlane3d` intersect from `06:1515` and `10:1533`) and
  `world_to_screen(camera, gt, world_pos) -> Option<Vec2>` (the popup-anchoring
  glue in `06:1304`, `07:1042`, `08:1863`, all guarding "off-screen / behind
  camera"). Pros: 4 games, one is a verbatim dupe, tiny surface, and it directly
  resolves the prior spike's open "popup rendering: worldspace vs projected UI
  node" question by giving `ui/popup` a blessed `world_to_screen` to track a
  world entity. Cons: needs a camera handle (helper functions, not a plugin);
  decide `meth` vs `camera` home (favor `camera`, it is camera-coupled).
- **`ui/touchpad` -- reveal-on-first-touch pad primitives.** NOT a fixed layout:
  a `TouchSeen` resource that flips true on first `any_just_pressed` and a helper
  that toggles a tagged HUD root's `Visibility`, plus pure window-fraction
  hit-test helpers (`button_grid_at(point, window, cols, rows, zone) ->
  Option<usize>` generalizing `vent_button_at` at `11:822`; and the
  `deflection -> stick offset` map from `touch_lean` at `08:1424`). Both hit-tests
  are already unit-tested in their games (`11:1162`, `08:2596`), matching the
  crate's pure-logic-tested convention. Evidence: 08, 09, 11 all reveal-on-touch
  and hit-test by window fraction; the reveal-on-first-touch rationale (runtime
  detection beats `#[cfg(wasm32)]` and `navigator.maxTouchPoints`) is identical
  across all the touch docs. Pros: 3 games, kills the most-documented
  duplication in the repo. Cons: highest charter tension -- must ship
  primitives (gate + pure hit-test) not an opinionated pad widget; the
  `just_pressed`-vs-held frame-derivation lesson must be baked in so ports do not
  reintroduce the held-finger leak both touch retros record.

### B. Gameplay primitives (medium evidence)

- **`scoring` -- Score + Combo/Streak.** A `Score` resource and a
  `Combo`/`Streak { window, count, timer }` that decays after a quiet window and
  scales a multiplier. Evidence: `06_fruitninja` `Combo` and `07_orbit` `Streak`
  (`:285`) which the code comments say is modelled on 06; plain `Score` in 5
  games. Pros: cross-referenced copy is strong signal; small, `Reflect`-friendly.
  Cons: "what a point is worth" is game-specific, so the module owns the
  decay/multiplier bookkeeping only, not the scoring rule -- risk of a thin
  wrapper; sketch and confirm it beats a raw `Timer + usize` before committing
  (a negative result is fine, like the prior spike's `spawn` caveat).
- **`transform`/`physics` radial gravity.** A `RadialGravity { strength }` (or a
  documented recipe) for the `gravity = -position.normalize_or(up) * strength`
  idiom that `08_dropzone` (`:1636`, and again at `:1668,1763,2184`) and
  `10_asteroids` set up around `Gravity(Vec3::ZERO)`. Pros: 2 games, clean fit
  next to `physics/pd_controller`. Cons: only 2 games and avian-coupled; might be
  better as a documented snippet than a module. Decide at sketch time.
- **`progress` -- difficulty ramp.** All six games ramp difficulty, but via
  genuinely different idioms: time-lerp (`06 ramp_t`), discrete `Level(n)`
  (`07`, `11`), `Wave(n)` (`10`), and log-scaled tiers (`09 tier_for_score`).
  The only truly shared core is "elapsed -> normalized t -> ease a value from
  start to cap". Pros: universal. Cons: the shared core is nearly a one-liner
  over Bevy's `EaseFunction` (and the Wave-2 `tween` task already brings easing);
  a `Level`-timer helper is more substantial but each game's level *effects*
  differ. Honest read: likely document the two idioms (time-ramp helper + level
  timer) rather than a heavy module. Lowest priority; may fold into `tween`.

### C. Document, do not build (weak / divergent evidence)

- **In-game text HUD helper.** `07:353` and `10:820` both redefine a local
  `hud_text` closure + marker component + `update_hud`. Only 2 games and the
  values differ; `ui/status` already covers most HUD needs. Note the pattern,
  do not module-ize yet.
- **Bounded arena / wall bounce.** Only `10_asteroids` truly reflects at bounds
  (`:952`); `08` clamps spawns instead; no game does toroidal wrap. Two games,
  two different approaches -- document the reflect-and-clamp recipe, do not build.
- **Trail rendering.** `06` blade-trail ring buffer (`:368`), `08` wind streaks,
  `10` a child flame mesh -- three different things called "trail". Defer to the
  already-logged particles proposal (`tasks/20260703-214928`).
- **"Reset the run" system + progression SFX ping.** All six games have an
  `OnEnter(Playing)` reset and most ping `level_up.wav` on a difficulty step.
  This is game-flow scaffolding, already owned by the deferred `ui/menu` +
  states proposal (`tasks/20260704-134800`); do not re-seed it here.

### Do nothing

Keep writing games and let each re-hand-roll the pointer, the projection math
and the touch pad. Cost: the input duplication is already the single
most-documented copy-paste in the repo (4 docs, 3 retros), each re-derivation
has re-hit the same held-finger and frame-derivation bugs the retros record, and
the verbatim `pointer_on_play_plane` dupe will drift. This is precisely the tax
the crate exists to stop.

## Recommendation

Run a second harvest wave -- **"Input & Projection"** -- as the direct sequel to
the juice kit, sequenced *after* the juice kit's Wave 2 foundations (`tween`
lands the easing that `progress` would lean on). Prove each module the crate way:
refactor the existing games onto it so the refactor is both the integration test
and the payoff (net line reduction, one correct copy).

Wave A -- do first, highest evidence, clean charter fit:

1. `camera/project` -- `pointer_on_plane` + `world_to_screen` helpers (4 games;
   one verbatim dupe; also unblocks `ui/popup` world-entity tracking). Smallest,
   least controversial, unblocks others -- start here.
2. `input/pointer` -- unified `Pointer` resource (3 independent copies). Keep a
   plain `Touches` path; do not force `bevy_enhanced_input`.
3. `ui/touchpad` -- reveal-on-first-touch gate + pure grid/stick hit-test
   primitives (3 games; the most-documented duplication). Ship primitives, not a
   fixed pad; bake in the `just_pressed`-not-held frame-derivation lesson.

Wave B -- medium value, sketch-then-commit (each may downgrade to a doc):

4. `scoring` -- Score + Combo/Streak decay (cross-referenced copy in 06/07).
5. radial gravity -- `RadialGravity` component or documented recipe (08, 10).
6. `progress` -- difficulty-ramp helper; likely a small helper or a doc, decide
   after sketching against `tween`.

Why this beats the runners-up: like the prior spike it is grounded in the
crate's own demonstrated duplication, but it covers the cluster that spike
missed -- and that cluster (input/touch) is the *most* documented and
*most* re-buggy duplication in the whole repo, so its expected payoff is the
highest of any remaining harvest. Wave A items each fit an existing module
directory (`camera/`, a new `input/`, `ui/`), carry no new hard dependency, and
land with an example refactor as their test. Wave B is deliberately second
because each item carries a real "is this more than a thin wrapper?" question
that should not block Wave A's clean wins. "Do nothing" loses for the usual
reason plus a sharper one here: the duplication it preserves is the one the
project has already written seven documents about.

## Open questions

- **`input/pointer` and `bevy_enhanced_input`.** Fruitninja routes touch through
  enhanced-input; asteroids/dropzone read raw `Touches`. Does the module offer
  only the raw path (simplest, no new dep in the core) and leave enhanced-input
  bridging to `helpers/`? Decide when planning task 2.
- **`ui/touchpad` scope.** How much layout is a "primitive"? A bare hit-test +
  visibility gate (recommended) vs a ready-made bottom-strip button-row builder
  mirroring `status_bar_item`. The latter is more useful but more opinionated --
  weigh at planning; may need a user call, like the prior spike's `ui/menu`.
- **`progress` earns its keep?** Is the ramp core more than a one-line
  `EaseFunction` call once `tween` exists? Sketch first; a doc is an acceptable
  outcome.
- **Home of the projection helpers.** `camera/project` (camera-coupled, favored)
  vs `meth` (pure math). Pick at planning task 1.

## Next steps

Direction-level tasks seeded for `/plan` to break into steps; each links back to
this doc and carries the `spike` tag. Sequence them after the juice-kit Wave 2
`tween` task (`tasks/20260704-134630`), which task 6 leans on.

- tatr 20260704-161502: `camera/project` -- screen<->world helpers (Wave A)
- tatr 20260704-161508: `input/pointer` -- unified pointer resource (Wave A)
- tatr 20260704-161513: `ui/touchpad` -- reveal-on-first-touch + hit-test primitives (Wave A)
- tatr 20260704-161518: `scoring` -- Score + Combo/Streak decay (Wave B)
- tatr 20260704-161522: radial gravity -- RadialGravity component or documented recipe (Wave B)
- tatr 20260704-161526: `progress` -- difficulty-ramp helper, sketch-then-decide (Wave B)
