# Harvesting 13_glide's UI juice into the crate

- TASK: tasks/20260705-090557 (spike / harvest-after-proof)
- SPIKE INPUT: docs/spikes/20260705-090421-ui-forward-slide-merge-puzzle.md
- REFERENCE: examples/13_glide.rs

`13_glide` is the crate's first all-UI game: the board is a `bevy_ui` tree and
every animation drives a plain `Node`/`BackgroundColor` field from a `Tween`
(never `Transform` scale, which UI layout owns). This note records which of its
game-local juice patterns generalized into the crate and which stayed local.

## What shipped: `ui/animate`

A new leaf module, the UI-node counterpart to the material-only
`feedback/flash`. It builds on `tween` and provides:

- **Colour <-> Vec4 helpers** (`color_to_vec4` / `vec4_to_color`). A `Tween<Vec4>`
  is how you animate a colour (Bevy's `Color` enum does not lerp component-wise),
  and every game doing a colour tween re-derived this pair. Small but load-bearing
  and exact.
- **Three opt-in marker components + `UiAnimatePlugin`**, which copy a tweened
  value into a UI field each frame after `TweenSystems::Advance`:
  - `TweenNodeOffset` -> `Tween<Vec2>` into `Node.left/top` (px) -- a slide;
  - `TweenNodeScale` -> `Tween<f32>` into `Node.width/height` (percent) -- a pop;
  - `TweenNodeBackground` -> `Tween<Vec4>` into `BackgroundColor` -- a flash/fade.
- **`node_flash(to, duration)`**: a `Tween<Vec4>` from bright white to `to`,
  encoding the "just changed" flash convention plus the colour conversion.

**Why markers, not a plugin that grabs every `Tween<Vec2>`.** The `tween` module
is deliberately target-agnostic (its own doc shows the game writing the applier).
Auto-applying every `Tween<Vec2>` to `Node.left/top` would steal the component
from a game using it for something else. Opt-in markers keep tween generic and
make the target explicit -- the same shape as the rest of the crate (`feedback`'s
`Flash`, `ui/status`'s items). `13_glide` now spawns the tweens + markers and
deletes its `apply_slide`/`apply_pop`/`apply_flash`/`flash_tween`/colour helpers
(and its now-purposeless `TileFace` marker, which only existed to scope those
appliers): ~45 lines of glue gone, replaced by three marker tags.

## What stayed game-local: the rolling-number readout

`13_glide`'s score readout rolls the displayed integer old -> new on a
`Tween<f32>`, retargeting mid-roll when the score changes again. This did **not**
promote, on purpose:

- Its source of truth is the game's `Score` resource, and what it renders is a
  game-specific `Text` format (`u32::to_string()` here, but "Score: {}",
  currency, "x1.5", etc. elsewhere). A crate `RollingNumber` would have to either
  take a format closure (framework machinery) or expose only a `shown: f32` the
  game still has to read and format -- at which point it saves nothing over the
  ~10 lines of retarget-on-change logic, which the `tween` module already
  enables directly.
- The genuinely reusable core ("retarget a `Tween<f32>` when the target changes,
  read it back") is a *pattern on top of tween*, not a new type. Promoting it now
  would be building an abstraction before a second concrete user exists -- the
  same rule the bastion-catalog spike enforced.

If a second game wants an animated numeric readout with the same shape, revisit a
`RollingNumber` component then; until then it is a documented recipe in this
example, not a crate API.

## Verdict per the task's three candidates

1. **UI-node feedback sibling (BackgroundColor flash + UI pop): PROMOTED** as
   `ui/animate` (`TweenNodeBackground` + `node_flash`, and `TweenNodeScale` for
   the pop). Generalizes cleanly, reuses `tween`.
2. **Tween a `Node`'s left/top from `Tween<Vec2>`: PROMOTED** as
   `TweenNodeOffset`. The canonical animated-UI-surface glue.
3. **Animated-number readout: KEPT LOCAL** -- format + source are game-specific
   and the reusable core is a thin tween pattern, not a type.

No follow-up task seeded.
