# Spike: What should we build to make *developing* the next game faster -- the dev harness and the app skeleton?

- DATE: 20260704-175058
- STATUS: RECOMMENDED
- TAGS: spike, features, roadmap, devtools, scaffolding

## Question

The crate's goal is "build games faster." Two prior spikes already mined the
in-game *runtime* duplication: the juice kit
(`tasks/20260704-134035/SPIKE.md` -- shake,
popup, flash, tween, persist, spawner, menu) and the input/projection harvest
(`tasks/20260704-161210/SPIKE.md` -- pointer,
project, touchpad, scoring, gravity, progress). Between them they seeded a
backlog that is roughly half shipped.

The user's request this time stresses a different word: help me *develop* more
games. That points past runtime mechanics to the **development workflow and the
app-level scaffolding** every game copies -- the half neither prior spike
touched. A good answer is a prioritized set of *new* candidates (library
modules and dev tools) that (a) neither prior spike catalogued, (b) are backed
by cross-game evidence, and (c) fit the "small composable blocks, not framework
machinery" charter -- or, for developer tooling, sit safely behind a feature
flag where that charter does not apply.

## Context

Shipped since the prior spikes: `camera/shake`, `camera/project`, `ui/popup`,
`feedback/flash`, `feedback/screen_flash`, `input/pointer`. Still open from
them: `tween`, `persist`, `spawn`+`cooldown`, `scoring`, radial gravity,
`progress`, and the DEFERRED `ui/menu` proposal (`tasks/20260704-134800`, needs
a user call on how far to generalize a state machine).

Two fresh sweeps of examples 06-11 (evidence with file:line below) surface a
cluster the runtime-focused spikes skipped: the **app skeleton, the game-flow
lifecycle, asset loading, and -- the headline -- the throwaway verification
harness** that gets rebuilt from scratch for every game. The single strongest
signal in the whole repo lives here and has never been proposed as a feature.

| Pattern (not in the prior two spikes) | Games / occurrences |
| --- | --- |
| Env-gated **autopilot** harness, hand-rolled then deleted before commit | 7 times (dropzone x4, reactor, overload, asteroids) |
| Env-gated **screenshot** harness (force window size + auto-advance + capture) | 2 times (reactor x2) |
| ~30-line app-skeleton preamble (wasm window + diagnostics + inspector guard) | 06-11 (6/6) |
| Per-run **reset** system (zero N resources + poke `CameraShakeInput.reset`) | 06-11 (6/6) |
| Hand-rolled `SfxAssets` handle-bag loaded inline, no ready-gate | 06-11 (6/6) |
| `menu_screen` / `game_over_screen` + `centered_screen` + `screen_text` | 06-11 (6/6, two helpers copied verbatim) |
| Generic in-memory `HighScore` + `record_high_score` + "New best!" | 06-11 (6/6) |
| "advance on any press" (`mouse||keys||touches.any_just_pressed`) | ~7 copies |
| `giveup_on_escape`, `pulse_menu_title`, alarm-banner widget, glowing-material idiom | 5-6 games each |

The autopilot/screenshot evidence is decisive and worth stating in full. AGENTS.md
already codifies the harness as a *standing gotcha* (`AGENTS.md`, "Running
examples" and "Seeing the screen"): to exercise a stateful example headlessly
you "add a TEMPORARY env-gated autopilot system that drives the state machine
and controls the ship, run it under `timeout` ... then remove the harness before
commit." It has been re-invented under a different env-var name every single
time -- `DROPZONE_AUTOPILOT`, `DROPZONE_SMOKE`, `OVERLOAD_SMOKE`, `REACTOR_SHOT`,
plus unnamed ones -- across `tasks/20260703-213510/RETRO.md`,
`tasks/20260704-103544/NOTES.md`, `tasks/20260704-103544`,
`tasks/20260704-103553`, `tasks/20260704-103517`,
`tasks/20260704-170738/NOTES.md`, `tasks/20260704-130314`,
`tasks/20260704-130316`, and `tasks/20260704-142016`. The reactor mobile-touch
retro names it outright: "A temporary env-gated window-size + auto-start harness
... is the single highest-leverage technique from this cycle"
(`tasks/20260704-143000/RETRO.md`). The screenshot
harness caught a real layout regression (four of six reactor shop buttons below
the fold at phone width) that `cargo build`, clippy, and the boot check all
missed. One task even records *skipping* verification because "adding/removing a
temporary harness to an unchanged example is pure churn"
(`tasks/20260704-130316`) -- the duplication tax made visible.

Crucially, a dev harness is not shipped game machinery: it lives behind the
existing `debug`/`dev` feature, exactly like `debug/inspector`. So the "no
framework machinery" charter line -- which rightly constrains the app-skeleton
and menu candidates below -- simply does not bind the highest-value item here.

## Options considered

Each candidate judged on evidence (occurrences), reuse, risk, and charter fit.
Grouped into the recommended dev-tools wave, an app-scaffolding wave that
carries real charter tension, and a harvest wave of clean leaf helpers.

### A. Developer tooling behind the `dev` feature (highest evidence, no charter tension) -- HEADLINE

- **`dev/autopilot` -- scripted state-driver harness.** An env-gated plugin,
  generic over the game's `States` type, that force-advances the state machine
  on a schedule (`Menu -> Playing -> ... -> GameOver`), optionally runs a small
  per-frame `*Input` script the game supplies, logs each transition and a final
  "cycle complete, no panic" line, then can exit cleanly. Turned on by an env
  var (`BCS_AUTOPILOT=...`) so it is inert in normal runs and needs no
  add/remove churn. How it works here: a `AutopilotPlugin::<S: States>` plus a
  builder for the transition schedule and an optional
  `Fn(&mut World, elapsed)` input hook; reads the same env-gate pattern the
  games already use. Pros: hand-rolled 7 times, AGENTS.md already documents the
  exact shape, zero charter tension (it is `debug`-feature dev tooling like the
  inspector), and it makes the crate's own "an example is not done until it has
  been run once" rule cheap to honour. Cons: must stay generic over the user's
  `States` and input resources without knowing them -- solved with a generic
  param plus a closure hook. Unknowns: how much of the "control the ship" half
  generalizes vs stays a game-supplied closure (favor: the plugin owns the
  state clock and logging; the game passes a one-line input closure); whether to
  fold the clean-exit (the `std::process::exit` wgpu-teardown segfault AGENTS.md
  warns about) into the plugin as a `AppExit` instead.
- **`dev/screenshot` -- viewport-capture harness.** A sibling env-gated plugin
  (`BCS_SHOT="WxH@state"`) that overrides `WindowResolution`, auto-advances to a
  named state, waits N frames, writes a PNG, and exits. How it works here: reuses
  the autopilot's state-driver, adds a `WindowResolution` override at startup and
  a Bevy screenshot capture. Pros: proven twice, caught a real responsive-layout
  regression invisible to every other check, and directly serves the "Seeing the
  screen" workflow AGENTS.md already blesses. Cons: capture-to-file plumbing and
  a deterministic "settled" frame count; the crop/`xdotool` step stays external
  (the plugin only needs to force size + capture full frame). Unknowns: rely on
  Bevy 0.19's built-in screenshot API vs a manual render-target readback.

  These two share a state-driver core, so they are one module (`dev/harness`)
  with two entry points, not two separate builds.

### B. App skeleton + game-flow scaffolding (high evidence, real charter tension)

- **`game_app()` / `GameAppPlugin` -- the boot preamble.** The ~30-line
  identical `main()` head -- the wasm-canvas `Window` block (copied
  character-for-character, same comment, in all six: `06_fruitninja.rs:139`,
  `08_dropzone.rs:546`), the `PhysicsPlugins`-only-for-the-inspector line with
  its apologetic comment (`09:235`, `11:152`), the `FrameTimeDiagnosticsPlugin`
  add-guard, and the `#[cfg(feature="debug")] InspectorDebugPlugin` -- becomes
  one plugin or builder, plus a `game_camera_2d()/3d()` helper for the
  camera+light+`AmbientLight`-per-camera split (the exact 0.19 footgun AGENTS.md
  flags). Pros: 6/6, the single largest copy-paste, removes a footgun-prone
  block. Cons/charter: a "boot my game for me" plugin edges toward the game
  template the charter warns against; must stay a thin, opt-out-friendly
  aggregation (each line is independently available), not a mandatory frame.
  Unknowns: how much to bundle vs expose as a `prelude` of one-liners.
- **`OnEnter(Playing)` run-reset helper.** Every game hand-writes a
  `start_run`/`start_game` that resets a pile of per-run resources to defaults
  and sets `CameraShakeInput.reset = true` (`06:846`, `08:1077`, `09:787`,
  `11:610`). The despawn half is already solved by Bevy's `DespawnOnExit(state)`
  marker (used ~9x per game), so only the reset half is duplicated. A
  registered-resource "reset to `Default` on state-enter" helper would absorb
  most of it. Pros: 6/6, purely mechanical. Cons: "which resources" is
  game-specific; risk of a thin wrapper over "call `Default`" -- sketch before
  committing. This is the concrete half of the DEFERRED `ui/menu` state-machine
  question (`tasks/20260704-134800`); it should sharpen that proposal, not race
  it.

### C. Screen / flow leaf helpers + asset registry (clean harvest, fits existing dirs)

- **`ui/menu` screen builders.** `centered_screen() -> Node` and
  `screen_text(text, size, color)` are duplicated *verbatim* across five files
  (`06:774,788`, `07:686,700`, `09:671,684`, `10:609,623`, `11:501,514`); on top
  of them every game builds a `menu_screen` (title + "tap to play" + best +
  controls) and a `game_over_screen` (title + score + the identical `new_best`
  branch + "tap to return"). This is the low-risk, in-scope half the deferred
  `ui/menu` proposal already named (mirroring `status_bar_item()`); this spike
  adds the verbatim-dupe evidence. Plus a `TitlePulse` component for the
  `pulse_menu_title` sine breathe copied in 5/6.
- **Generic `HighScore<T: PartialOrd>` + `record_high_score` + "New best!".**
  6/6 hand-roll an in-memory best-score resource and the new-best branch
  (`06:351,984`, `07:297`, `09:511`, `10:303`, `11:297`); the value type varies
  (`usize`/`f64`/`f32`) so the API must be generic. Distinct from the open
  `persist` task -- `persist` would *save* this resource; this is the resource
  itself and its update rule. Small, `Reflect`-friendly.
- **`assets` sound/handle registry (+ optional ready-gate).** 6/6 hand-roll a
  flat `SfxAssets` bag of named `Handle<AudioSource>` loaded inline in `setup`
  (`06:558`, `08:439`, `09:514`, `11:308`), sharing the same placeholder `.wav`
  files, with no loading-state gate anywhere -- every game boots straight to
  `Menu` and trusts async handles are ready. A named-sound enum + loader (and an
  optional `AssetsReady` state gate) would deduplicate the struct and the
  file-path strings. Fits beside `audio`. Unknowns: whether a ready-gate earns
  its keep given the games get away without one today (favor: ship the registry,
  make the gate opt-in).
- **`AnyStartPress` helper on `input/pointer`.** The keyboard-inclusive "advance
  on any press" check (`mouse.just_pressed || keys.any || touches.any_just_pressed`)
  is copy-pasted ~7x for menu/game-over dismissal (`07:677`, `08:1067`, `09:774`,
  `11:599`, ...). `UnifiedPointer` already exists but only 10_asteroids uses it;
  the win is a `just_started()`-style helper plus migrating the other five onto
  the existing resource. Tiny, extends a shipped module.
- **Leaf one-liners:** `giveup_on_escape` (5 games), a `status_bar_with_fps()`
  convenience (the identical 8-line FPS `status_bar_item` block, 6/6), a
  `glowing_material(base, emissive)` helper (the emissive-blooms-never-`unlit`
  idiom retyped 4-5x, a recurring footgun). Trivial, high footgun-avoidance.

### Do nothing

Keep writing games and re-hand-rolling the harness, the boot preamble, the
reset systems and the menu screens. Cost: the verification harness -- the
crate's *own documented* highest-leverage technique -- stays a copy-paste that
gets rebuilt and deleted every game, so verification stays expensive enough that
tasks skip it as "pure churn"; the boot footguns (`unlit` emissive, per-camera
`AmbientLight`) keep recurring; and the crate keeps under-delivering on "develop
games faster" for the development loop specifically, not just the runtime.

## Recommendation

Run a **"Dev Loop & App Scaffolding"** wave, sequenced as the sequel to the two
runtime harvests, and led by the developer tooling because it is both the
highest-evidence and the lowest-risk item in the entire remaining backlog.

Wave 1 -- do first, headline, zero charter tension (behind the `dev`/`debug`
feature):

1. `dev/harness` -- the `autopilot` + `screenshot` state-driver plugins. This is
   the single clearest reusable-tool candidate in the repo: hand-rolled 9 times
   total, already documented in AGENTS.md as the technique to use, and it makes
   the crate's own "run the example once before it's done" rule cheap. Build it,
   then delete the AGENTS.md "add a temporary harness" gotcha in favour of "add
   `DevHarnessPlugin` and set `BCS_AUTOPILOT`."

Wave 2 -- clean leaf harvest, fits existing module dirs, low charter risk:

2. `assets` -- sound/handle registry (+ opt-in ready-gate).
3. Generic `HighScore<T>` + `record_high_score` + "New best!" (complements, does
   not overlap, the open `persist` task).
4. `ui/menu` screen builders (`centered_screen`, `screen_text`, `menu_screen`,
   `game_over_screen`, `TitlePulse`) -- the low-risk half of the deferred
   `ui/menu` proposal; fold this evidence into `tasks/20260704-134800` rather
   than opening a rival task for the same builders.
5. `input` `AnyStartPress` helper + migrate the five games onto the existing
   `UnifiedPointer`; leaf one-liners (`giveup_on_escape`, `status_bar_with_fps`,
   `glowing_material`).

Deferred -- needs the user decision the prior spike already surfaced:

6. `game_app()`/`GameAppPlugin` and the run-reset helper. Both are 6/6
   duplication, but both edge toward the "boot/template my game" machinery the
   charter warns against, and the reset helper is the concrete half of the
   already-deferred state-machine question. Do NOT build these blind -- roll
   them into the same user call as `tasks/20260704-134800` (how far to
   generalize the app/state layer).

Why this beats the runners-up: it is the first candidate set grounded in the
crate's own *development* pain rather than its runtime mechanics, so it is purely
additive to the two prior spikes instead of a third rehash of the same juice.
The headline item is unique in the whole backlog for being simultaneously the
highest-evidence (9 hand-rolls, self-documented in AGENTS.md) and the
lowest-charter-risk (dev-only, feature-gated, like the inspector) -- there is no
reason to sequence it behind anything. The leaf harvest in Wave 2 fits existing
directories and lands with the usual example-refactor-as-test payoff. The
genuinely opinion-heavy items (app builder, state/reset layer) are correctly
deferred to the one user decision the project has already flagged, so this spike
does not pre-empt it. "Do nothing" loses hardest here because the thing it
preserves is the crate's own documented single-highest-leverage technique.

## Open questions

- **Autopilot generality.** How much of "drive the game" can the plugin own
  generically (state clock + logging + clean `AppExit`) vs must stay a
  game-supplied input closure? Resolve by sketching `dev/harness` against
  06_fruitninja and 08_dropzone (one 2D-ish, one physics) before finalizing the
  API for task 1.
- **Screenshot "settled frame" determinism.** How many frames to wait after a
  state transition before capture, and Bevy 0.19 built-in screenshot API vs a
  manual readback? Decide at planning task 1.
- **Ready-gate worth it?** The games ship with no asset-ready gate and get away
  with it. Ship the registry unconditionally and make the gate opt-in; confirm
  when planning task 2.
- **App/state layer depth (needs user -- shared with `tasks/20260704-134800`).**
  Whether to build `game_app()` and the run-reset helper at all, or leave them
  as the game's own boilerplate to preserve the "not a game template" line. This
  is the same decision the deferred `ui/menu` proposal is waiting on; answer it
  once, for both.

## Next steps

Direction-level tasks seeded for `/plan` to break into steps; each links back to
this doc and carries the `spike` tag. The already-open runtime tasks
(`tween`, `persist`, `scoring`, `progress`, touchpad, gravity) and the deferred
`ui/menu` proposal are NOT re-seeded here.

- tatr 20260704-175421: `dev/harness` -- autopilot + screenshot state-driver plugins (Wave 1, headline)
- tatr 20260704-175422: `assets` -- sound/handle registry + opt-in ready-gate (Wave 2)
- tatr 20260704-175423: generic `HighScore<T>` + record_high_score + "New best!" (Wave 2)
- tatr 20260704-175424: `ui/menu` screen builders -- centered_screen/screen_text/menu_screen/game_over_screen/TitlePulse (Wave 2; fold into tasks/20260704-134800)
- tatr 20260704-175425: `input` AnyStartPress helper + adopt UnifiedPointer in 5 games; leaf one-liners giveup_on_escape / status_bar_with_fps / glowing_material (Wave 2)
- (deferred, no task) `game_app()`/GameAppPlugin + run-reset helper -- route into the user decision on tasks/20260704-134800, do not build blind
