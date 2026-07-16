# Spike: A UI-forward slide-merge puzzle (`13_glide`) as the next example

- DATE: 20260705-090421
- STATUS: RECOMMENDED
- TAGS: spike, examples, games, ui, tween, roadmap

## Question

Which small (~1500-2000 line) prototype game should we add to `examples/`
next, now that `12_bastion` has shipped? A good answer is one concrete concept,
chosen because it is the canonical demo for a crate capability that currently
has no headline, with enough shape (interaction loop, which modules it wires,
the fun hook) that `/plan` can expand it into steps without re-deciding what to
build. Per an explicit user steer during this spike: the pick should lean
heavily on **UI**, be a vehicle for **really good, copy-pastable UI patterns**,
and treat UI quality as a first-class goal rather than an afterthought.

## Context

The crate grows by building example games and harvesting the reusable systems
out of them; every example after `05` follows one shape (menu/playing/game-over
`States`, `SfxPlugin` one-shots, a `ui/status` HUD, a wasm/trunk build,
touch-playable controls). A fresh coverage sweep of `src/` against the twelve
examples (`01_sphere` ... `12_bastion`) shows the **module-gap** angle is now
nearly exhausted -- `12_bastion` closed the last three never-demoed modules
(`camera/project`, `transform/point_rotation`, `transform/smooth_look_rotation`;
see `tasks/20260704-220530/SPIKE.md`). What remains are
**substantial modules that are nobody's headline**, plus two genre/viewpoint
gaps:

Modules with real substance that are never the star of a numbered demo:

| Module | Uses | Note |
| --- | --- | --- |
| `feedback` (flash + screen_flash) | 4 | whole juice module, never headlined |
| `tween` | 1 (only `06`) | general A->B animation primitive, never headlined |
| `persist` (`PersistPlugin<T>`) | 1 (only `06`) | native-JSON / wasm-localStorage save, never headlined |
| `scoring/high_score` (`HighScore<T>`) | 6 | pervasive but never the point of a game |
| `time/cooldown` | 3 | never headlined |
| `camera` family (chase/skybox/post/shake) | 1-5 | no camera-centric example |

Genre/viewpoint gaps in the gallery: there is **no puzzle game** and **no
first-person game** (`camera/wasd` appears only in the `01/02/04/05` tech demos,
never in a real game).

Also relevant to the "copy-pastable UI patterns" ask: the crate's `ui/` modules
today are `status` (HUD), `menu` (overlay builders), `popup` (floating "+N") and
`touchpad` (touch gating). There is **no reusable pattern for an animated,
laid-out UI surface** -- a grid/board of cells, a tile that eases between
positions, a UI node that flashes or pops on change. Every example that wants UI
motion hand-rolls it, and the juice modules that would help (`feedback/flash`,
`feedback/screen_flash`) are **material/mesh-oriented** (`flash` clones a
`StandardMaterial`), so they do **not** apply to Bevy UI nodes at all. That is a
real, unfilled niche a UI-forward game would both exercise and expose.

## Options considered

Four concepts were sketched and weighed (full detail in the diverge notes
below); the user steered convergence toward the most UI-centric one.

### A. `13_glide` -- slide-merge (2048-style) puzzle [RECOMMENDED]

A 4x4 board of numbered tiles. Swipe (or arrow keys) slides every tile one
direction; equal neighbours merge into their sum, one new tile spawns per move,
and the board filling with no legal move ends the run. Score is the sum of all
merges; the best score persists across launches. It is almost entirely a **UI**
game: the board and tiles are Bevy UI nodes, so it is the natural home for a
polished, reusable UI layer.

- Headlines: **`tween`** (every tile slide is a `Tween<Vec2>` of the node's
  `left`/`top`; every spawn and merge is a scale/opacity pop) and **`persist` +
  `scoring/high_score`** (a 2048 without a saved best score is pointless, so the
  save primitive is load-bearing, not incidental).
- Reuses: `feedback` (see UI-flash note below), `ui/popup` (+score on merge),
  `ui/menu` (title/game-over overlays), `input/pointer` (`UnifiedPointer` drag
  -> swipe direction), `audio` (`SfxPlugin` slide/merge/spawn/game-over),
  `input/state` (key -> state), `camera/shake` (optional, on a big merge), and
  the wasm/trunk build. Renders with a plain `Camera2d` (like `09`/`11`).
- Pros: fills the **puzzle genre gap**; swipe is a native touch fit; the tiniest
  option (no physics, no 3D, no camera rig); makes `tween` and `persist` --
  three never-headlined modules -- genuinely central; and its UI focus directly
  serves the user's ask. Strong candidate to **surface the next harvest**: the
  reusable UI patterns it forces out (see Recommendation) are exactly the
  copy-pastable building blocks the crate wants.
- Cons: the classic 2048 rule set is well-trodden, so the novelty is in the
  presentation, not the mechanic -- which is fine here, because presentation
  (UI/juice) is the point.
- Unknowns: does a UI **tile pop** scale via `Transform` on a `Node` (Bevy UI
  has historically ignored `Transform` scale), or must the pop tween node
  `width`/`height` / a child, or wrap the tile in a scalable layer? This is the
  first thing to settle at implementation (a `ScreenshotPlugin` grab answers it).

### B. `13_breaker` -- brick-breaker (Arkanoid)

Headlines the `feedback` module (flash + screen_flash) plus `tween`, in a new
paddle-arcade genre. Rejected as the primary pick: it is sprite/action-oriented,
not UI-oriented, so it does not serve the "UI a lot / copy-pastable UI patterns"
steer. Kept as the strongest runner-up if a `feedback` headline is wanted later
(it would need a UI-flash counterpart regardless -- see the harvest note).

### C. `13_glider` -- physics gate-racer

Gives the single-use physics modules (`pd_controller`, `camera/skybox`,
`camera/chase`, all only in `08_dropzone`) a second, validating home. Rejected:
`pd_controller` is **already** `08`'s headline, so a second flyer is a redundant
demo (the exact trap `12_bastion`'s spike avoided), it overlaps `08`'s stack
heavily (reskin risk), and it is the opposite of UI-forward.

### D. `13_corridors` -- first-person collectathon

Fills the only viewpoint gap (`camera/wasd` has no real game). Rejected here (and
already ranked lowest-payoff by the prior spike): `wasd` is not a gap module,
level collision is the most net-new code for the least new-module payoff, and it
is not a UI showcase.

### Do nothing

Cost: the puzzle genre stays unrepresented, `tween`/`persist` keep having no
headline, and the crate never grows a reusable animated-UI-surface pattern --
which the user has now explicitly asked for. Low upside to deferring.

## Recommendation

Build **`examples/13_glide`**: a UI-forward, `tween`-headlining slide-merge
(2048-style) puzzle, with UI quality treated as a first-class goal.

### Shape (first cut)

- **Board:** a 4x4 grid. Draw the static cell backing with a real CSS-`grid`
  `Node` (`display: Display::Grid`) so the layout is responsive at any width
  (per the reactor mobile lesson: percentage-based grid, not fixed px +
  `flex_wrap`). Draw the tiles as a **separate, absolutely-positioned layer** on
  top, one `Node` per tile with `position_type: Absolute` and tweened
  `left`/`top` -- a pure grid layout cannot animate a tile moving between cells,
  so the moving layer must be absolute and positioned by the game each frame from
  its `Tween<Vec2>`. This split (static responsive grid underlay + animated
  absolute tile layer) is the core reusable UI pattern the example teaches.
- **Input:** `UnifiedPointer` drag resolved to the dominant axis -> a swipe
  direction; plus arrow keys / WASD. One move = slide + merge + spawn.
- **Rules:** standard 2048 (slide to the wall, merge equal pairs once per move,
  spawn a `2`/`4` in a random empty cell). Game over when no legal move remains
  -> game-over state. Reaching a `2048` tile is an optional win banner with
  continue.
- **Scoring / save:** score = running sum of merge values; `HighScore<u32>`
  wired through `PersistPlugin` so the best survives a relaunch (this is the
  `persist` headline -- make the "new best!" edge visible).
- **Juice (the UI focus):** tile **slide** = `Tween<Vec2>` on node position;
  tile **spawn** and **merge** = a scale/opacity pop; **score** = `ui/popup`
  "+N" plus an animated number roll; `ui/menu` title pulse and game-over overlay;
  optional `camera/shake` on a large merge. `SfxPlugin` one-shot per event.

### The harvest this surfaces (the "copy-pastable patterns" payoff)

The user's real ask is reusable UI. Building this well forces out patterns the
crate does not yet have; capture them for a follow-up harvest spike rather than
gold-plating the example:

1. **UI-node juice** parallel to the mesh-oriented `feedback` module: a
   node-`BackgroundColor` **flash** (`Tween<Vec4>`) and a node **pop** that
   works within UI layout constraints. Today `feedback/flash` only touches
   `StandardMaterial`, so UI has no equivalent -- a `feedback`-sibling for UI is
   a clean, on-charter promotion.
2. **Animated UI surface pattern:** the static-grid-underlay + absolute-tween-
   layer split, and a small "animate a `Node`'s `left`/`top` from a `Tween<Vec2>`"
   glue system, are reusable by any board/inventory/card UI.
3. **Animated number** (roll a displayed integer from old to new via a tween) --
   recurs in HUDs and score readouts.

Do NOT build these as crate modules inside this task; ship them game-local in
`13_glide` first (the crate's harvest-after-proof rule), then evaluate promotion
in a follow-up spike once there is a concrete, working reference.

## Open questions

- **UI pop scaling:** can a tile `Node` be scaled for the pop via `Transform`,
  or must the pop animate `width`/`height` or a wrapping layer? Settle first at
  implementation with a `ScreenshotPlugin` grab; it drives how the tile entity is
  structured.
- **Swipe resolution on touch/wasm:** map a `UnifiedPointer` drag to a single
  discrete direction (dominant axis past a distance threshold). Verify with the
  `AutopilotPlugin` input closure, not synthetic clicks (per the input-refactor
  lesson).
- **Tween coordination for a whole-board move:** many tiles tween at once and
  some despawn on merge-completion. Decide whether merges resolve on
  `TweenFinished` (an `On<Add, TweenFinished>` observer) or on a fixed
  move-duration timer; the former composes with the module, the latter is
  simpler. A tuning/architecture call for `/plan`, not a blocker.
- **Harvest generality:** do the three UI patterns above want to become crate
  modules (a UI-`feedback` sibling, a `Node`-position tween helper), and can they
  reuse anything from `tween`/`feedback`? Deferred to a follow-up spike after the
  game-local versions exist.

## Next steps

Direction-level tasks seeded for `/plan` to break into steps:

- tatr 20260705-090624: build `examples/13_glide` -- a UI-forward slide-merge (2048-style)
  puzzle. Headline `tween` (tile slide/pop animations on Bevy UI nodes) and
  `persist` + `scoring/high_score` (saved best score). 4x4 board, swipe + arrow
  input, standard 2048 rules, menu/playing/game-over states, `ui/popup` /
  `ui/menu` / `audio`, `Camera2d`, wasm/trunk build. Treat UI quality as a
  first-class goal: static responsive grid underlay + absolute tweened tile
  layer, game-local UI-juice helpers (node flash/pop, animated number).
- tatr 20260705-090557: follow-up -- evaluate promoting the game-local UI patterns
  from `13_glide` into the crate: a UI-node `feedback` sibling (node color flash
  / pop, paralleling the material-only `feedback/flash`), a "tween a `Node`'s
  `left`/`top` from `Tween<Vec2>`" helper, and an animated-number readout.
  Depends on the MVP shipping first.
