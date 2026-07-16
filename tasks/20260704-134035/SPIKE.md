# Spike: What library features should bevy_common_systems add next?

- DATE: 20260704-134035
- STATUS: RECOMMENDED
- TAGS: spike, features, roadmap, ergonomics

## Question

The crate's stated goal is "build games faster": a collection of small,
copy-pastable, game-agnostic building blocks. We have eleven modules and ten
example games. The fuzzy request: what should we add next to make building a
game with this crate easier? A good answer is not a wishlist -- it is a
prioritized set of *library* features backed by evidence that a real game
needs them, chosen so each one fits the crate's existing conventions and its
"no framework machinery" guardrail.

## Context

The crate today (`src/`): `audio` (SFX one-shots), `camera` (chase / post /
skybox / wasd), `debug`, `health`, `helpers` (despawn / temp / wasd), `mesh`
(builder / explode), `meth` (lerp / sphere), `modding` (events / registry),
`physics` (pd_controller), `transform` (orbit + rotation drivers), `ui`
(status bar). Every runtime module follows the same shape: one `*Plugin` per
concern, a Config / `*Input` / `*Output` / private `*State` component split,
observers for reactive setup, preludes, `Reflect` derives.

The crate has grown almost entirely by building example games (`06`-`11`) and
harvesting reusable systems out of them (that is how `audio`, `mesh/explode`,
the whole `transform` family and `ui/status` came to exist). That harvesting
is incomplete. Grepping the ten examples shows the *same* gameplay-juice and
scaffolding code hand-rolled in game after game, never promoted to the
library:

| Pattern | Examples that hand-roll it |
| --- | --- |
| Camera shake (trauma resource + offset) | 06, 07, 08, 10 |
| Floating "+N" score/damage popup | 06, 07, 08 |
| Hit / damage material flash | 06, 07, 10 |
| High-score persistence (in-memory only, resets each launch) | 06, 07, 08, 10, 11 (all) |
| Menu / Playing / GameOver states + menu buttons | 06, 07, 08, 10, 11 |
| Easing / tween of transform, color, scale | 06, 07, 08, 10, 11 |
| Timed spawner (interval + jitter) | 06, 08 |

Two facts sharpen this. First, the camera-shake copy is *bug-prone*: the
asteroids retro (`tasks/20260703-170744/RETRO.md`) records
that the shake was written as an accumulating `+=` instead of an absolute
`BASE + offset` and drifted the camera -- a bug the library would have
prevented once. Second, "high score" is a lie in every game: it is a plain
`Resource` (`06_fruitninja.rs:355  struct HighScore(usize)`) that resets on
every launch, because the crate has no persistence primitive and nobody wanted
to hand-roll native-plus-wasm storage per game.

This is the strongest possible signal for what belongs in a "build games
faster" crate: not new speculative capabilities, but the boilerplate the
crate's own games already prove they need. Two capability *gaps* are also
already logged as user-facing proposals: particle bursts
(`tasks/20260703-214928`) and background music / ambience
(`tasks/20260703-214929`).

## Options considered

Framed as candidate feature areas. Each is judged on evidence (how many games
need it), reuse, risk, and fit with the crate conventions and the explicit
"small composable blocks, not framework machinery" rule.

### A. Promote recurring juice into library modules (highest evidence)

- **`camera/shake` -- `CameraShake` trauma component.** A trauma value that
  decays per second and offsets the camera as `BASE + trauma^2 * random`;
  code adds trauma via a method or event. How it works here: slots directly
  next to `camera/chase` and `camera/post`, Config (`CameraShake`) + a
  trauma-add `*Input` + `*Output` writing the offset. Pros: 4 games need it,
  API is obvious, already burned us with a bug. Cons: must compose with the
  chase camera (shake is an offset applied *after* chase writes the transform)
  -- an ordering `SystemSet` handles it. Unknowns: whether to shake rotation
  too (kick), or translation only for v1.
- **`ui/popup` -- floating text.** `spawn_popup(text, world_or_screen_pos,
  color)` spawns a label that rises and fades over a lifetime, then
  self-despawns (builds on `helpers/temp`). Pros: 3 games, pure additive, no
  ordering hazards. Cons: world-to-screen projection needs a camera handle;
  billboarded 3D text vs screen-space UI text is a design choice. Unknowns:
  3D worldspace text vs a UI node tracked to a projected point.
- **`feedback` -- hit flash.** A `Flash { color, duration }` component that
  briefly overrides an entity's material emissive/base color and eases back.
  Pros: 3 games. Cons: has to clone-and-restore the material (or use a
  per-entity material) without leaking handles. Unknowns: how to restore
  cleanly when the base material is shared.

### B. Generalize existing math into a tween engine

- **`tween` -- easing engine.** Generalize `meth/lerp::LerpSnap` (which only
  does exponential lerp-with-snap) into a duration-based tween: animate a
  `Transform` component / material color / scale from A to B over `t` seconds
  with a Bevy `EaseFunction`, firing a completion event. Pros: 5 games ease
  *something* by hand; would also back the popup rise and the flash decay, so
  it is a foundation, not a leaf. Cons: risk of reinventing `bevy_tweening`;
  scope creep toward a general animation system (framework machinery). Fit:
  keep it deliberately narrow -- a `Tween<T>` component + a handful of
  built-in target adapters, not a keyframe timeline DSL. Unknowns: lean on
  Bevy 0.19's built-in `EaseFunction`; decide component-tween vs
  reflection-driven field-tween (favor the former for simplicity).

### C. Persistence (all games need it, no primitive exists)

- **`persist` -- save/load resource.** A small wrapper that serializes a
  `Resource` to disk on native (ron/serde under a project dir) and to
  `localStorage` on wasm, so high scores and settings survive a restart.
  Pros: every game wants it; the wasm/native split is exactly the kind of
  cross-platform plumbing a utility crate should own once. Cons: touches the
  filesystem and web APIs (the crate has no such dependency today); a mature
  option (`bevy-persistent`) exists and might be a better dependency than a
  hand-roll. Unknowns: build vs depend -- resolve with a short follow-up, but
  the wasm-web build already solved the `getrandom` wasm split, so the
  precedent for owning this plumbing exists.

### D. Spawning and timing helpers

- **`spawn` -- timed spawner + optional pool.** A `Spawner { interval, jitter
}` component that emits a spawn signal on a cadence, plus (optionally) a small
  object pool to avoid churn. Pros: 06 and 08 hand-roll interval spawning;
  wave spawners in 07/10 are adjacent. Cons: "what to spawn" is game-specific,
  so the module can only own the *timing*, not the entity -- risk of a thin
  wrapper over `Timer`. Unknowns: whether a signal-only spawner earns its
  keep vs just documenting the `Timer` pattern.
- **`time/cooldown` -- cooldown / i-frames.** A `Cooldown` component for
  weapon fire and invulnerability windows (10 hand-rolls i-frames). Small;
  could live beside `helpers/temp`.

### E. State + menu scaffolding (high reuse, high opinion)

- **`ui/menu` + game-flow states.** Every game copies a Menu / Playing /
  GameOver state enum plus button-spawning UI. A reusable `menu_button()`
  bundle builder (mirroring `status_bar_item()`) is low-risk and clearly in
  scope. A generic *state machine*, however, edges toward the framework
  machinery the crate's charter warns against. Pros: 5 games. Cons: the state
  enum is game-specific; over-generalizing it fights the "game-agnostic
  building blocks, not a game template" principle. This one needs a user
  decision on how far to go.

### F. New capabilities already proposed (need a dependency decision)

- **Particles** (`tasks/20260703-214928`) and **background music**
  (`tasks/20260703-214929`) are logged proposals awaiting a user call on
  whether to pull a crate or hand-roll. Particles could be built in-house
  (billboarded quads + velocity + fade, reusing `mesh` and `helpers/temp`,
  no new dep); music is a small manager over Bevy audio. Left as-is; this
  spike does not re-decide them, only points at them.

### Do nothing

Keep growing by writing more example games and let each re-hand-roll the
juice. Cost: the crate's own examples keep duplicating shake / popup / flash /
persistence, the duplication keeps re-introducing bugs (the shake drift), and
the crate under-delivers on its one stated goal -- the reusable blocks that
would make the *next* game faster to build never get harvested.

## Recommendation

Pursue a themed **"Game Juice & Scaffolding Kit"**: systematically promote the
boilerplate the crate's own games already prove they need, harvesting from the
examples exactly as `audio` and `transform` were harvested before. Do it in
waves, and prove each module by refactoring the existing games onto it -- the
refactor is both the test (ECS behavior is validated by the examples, per the
testing convention) and the payoff (net line reduction, one bug-free copy).

Wave 1 -- low risk, high evidence, clean Config/Input/Output fit, no new
dependency (do these first):

1. `camera/shake` -- `CameraShake` trauma component (4 games; fixes the
   known drift bug once).
2. `ui/popup` -- floating "+N" text (3 games).
3. `feedback` hit flash -- `Flash` component (3 games).

Wave 2 -- high value, a little more design:

4. `tween` -- narrow easing engine generalizing `LerpSnap`; back the popup
   rise and flash decay with it so it earns its foundation status.
5. `persist` -- cross-platform save/load for high scores and settings
   (resolve build-vs-`bevy-persistent` first; see open questions).
6. `spawn` / `time/cooldown` -- timed spawner and cooldown/i-frames, only if
   they prove to be more than a `Timer` wrapper once sketched.

Deferred, needs a user decision (do not start blind):

7. `ui/menu` + game-flow states -- the `menu_button()` bundle builder is in
   scope; a generic state machine may be framework machinery. Ask before
   building the state layer.
8. Particles / music -- already logged as proposals
   (`tasks/20260703-214928`, `tasks/20260703-214929`); this spike defers to
   those, it does not re-open them.

Why this beats the runners-up: it is the only option grounded in the crate's
own demonstrated needs rather than guessed ones, every Wave-1 item fits an
existing module directory and the established component-split convention, and
each lands with a built-in integration test (the example refactor). "Do
nothing" loses because it keeps paying the duplication-and-bugs tax on the one
thing the crate exists to prevent. The tween/persist wave is sequenced second
because it carries a genuine build-vs-depend question that should not block the
obvious wins.

## Open questions

- **Persistence: build or depend?** Hand-roll a thin native+wasm wrapper, or
  pull `bevy-persistent`? Resolve with a 30-minute follow-up spike or a user
  call before starting task 5.
- **Tween scope.** Component-tween only (recommended) vs reflection-driven
  field tween; and how much overlap with `bevy_tweening` is acceptable before
  we should just depend on it. Decide at planning time for task 4.
- **Popup rendering.** Worldspace billboarded 3D text vs a screen-space UI
  node tracked to a projected point. Pick when planning task 2.
- **Menu scaffolding depth (needs user).** Just a `menu_button()` builder, or
  a reusable state machine too? The latter risks the "no framework machinery"
  line -- needs an explicit user decision.

## Next steps

Direction-level tasks seeded for `/plan` to break into steps (each links back
to this doc and carries the `spike` tag). Particles and music are intentionally
*not* re-seeded -- they already live as `tasks/20260703-214928` and
`tasks/20260703-214929`.

- tatr 20260704-134500: `camera/shake` -- CameraShake trauma module (Wave 1)
- tatr 20260704-134530: `ui/popup` -- floating text module (Wave 1)
- tatr 20260704-134600: `feedback` -- hit-flash module (Wave 1)
- tatr 20260704-134630: `tween` -- easing engine generalizing LerpSnap (Wave 2)
- tatr 20260704-134700: `persist` -- cross-platform save/load resource (Wave 2)
- tatr 20260704-134730: `spawn` / `time/cooldown` -- timed spawner + cooldown (Wave 2)
- tatr 20260704-134800: PROPOSAL (needs user) -- `ui/menu` button builder + optional game-flow state scaffolding (Wave 3)
