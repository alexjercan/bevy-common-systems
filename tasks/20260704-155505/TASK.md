# feedback: promote the full-screen damage overlay (06/07/10) into a screen-flash

- STATUS: CLOSED
- PRIORITY: 34
- TAGS: feature,feedback,cleanup


> Split out of tasks/20260704-134600 (feedback material Flash). While building
> the material Flash it turned out the spike's "material flash | 06,07,10"
> premise was wrong: those games do NOT flash a material, they hand-roll a
> full-screen red UI *overlay* (this is the real, higher-evidence duplication).

## Goal

Promote the full-screen damage/hit overlay that three games duplicate into the
`feedback` module (e.g. `feedback/screen_flash.rs`), and refactor the three
copies onto it (real dedup, deleting local code):

- `examples/06_fruitninja.rs` -- `RedFlash { age, lifetime }` + `fade_red_flash`,
  a full-screen `BackgroundColor` overlay shown on a bomb death.
- `examples/07_orbit.rs` -- `DamageFlash(f32)` intensity resource +
  `DamageFlashOverlay` node + `fade_damage_flash`, spiked on a hazard hit.
- `examples/10_asteroids.rs` -- `RedFlash` full-screen overlay on ship destroyed.

## Design sketch (decide at planning)

The three differ: 06/10 spawn a short-lived overlay entity that fades over a
lifetime (like `ui/popup`); 07 keeps a persistent overlay whose alpha is a
resource spiked to a peak and decayed each frame. Pick one API that covers both,
e.g. a `ScreenFlash { color, peak_alpha, decay }` component (or a
`commands.screen_flash(color)` helper) that spawns/updates a full-screen overlay
and fades it. Consider whether it belongs in `feedback` or `ui` (it is a UI
overlay, but conceptually hit feedback -- lean `feedback` to sit beside the
material flash).

## Steps

- [x] Design the screen-flash API (one shape covering the spike-to-peak-then-decay
      and the spawn-and-fade variants). See
      docs/2026-07-04-feedback-screen-flash-module.md: a `ScreenFlash { peak_alpha,
      decay, despawn_on_end }` component (color lives in `BackgroundColor`), plus
      `screen_flash()` (one-shot builder) and `screen_flash_node()` (shared node).
- [x] Add `src/feedback/screen_flash.rs` (+ prelude wiring), with tests (4:
      alpha scale/clamp, insert-spike + tint preserved, spawn-and-fade despawn,
      persistent re-spike).
- [x] Refactor 06, 07 and 10 onto it, deleting their local overlay copies
      (RedFlash/fade_red_flash, DamageFlash resource + fade_damage_flash).
- [x] Verify: full check suite (fmt/clippy/test/examples/ascii, all green) +
      booted all three examples to the render loop, no panics.
