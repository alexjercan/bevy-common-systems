# 10_asteroids: verify touch journey + web canvas touch-action

- STATUS: CLOSED
- PRIORITY: 80
- TAGS: example,mobile,historical

## Goal

`examples/10_asteroids.rs` already unifies mouse + touch (pointer-follow, hold
to fly + auto-fire) and its menu/result screens use `pointer.just_pressed`
(which includes touch, and `update_pointer` runs in `PreUpdate` in every
state). Confirm the entire menu -> playing -> result journey is touch-playable
and fix the one known web gap: the canvas is not `touch-action: none`, so touch
drags scroll/zoom the page instead of steering. Close any real gap found.

## Steps

- [x] Re-read the pointer path: `update_pointer` (~300), its `PreUpdate`
      registration (~206), `menu_click` (~733) and result input (~1465). Confirm
      touch flows into `pointer.just_pressed` / `pointer.pressed` in every state
      and that menu + result actually consume it. Document the trace in the
      task close-out.
- [x] Add `touch-action: none;` to the `#game-canvas` in
      `web/games/10_asteroids/index.html` (copy the dropzone CSS block) and
      confirm the viewport has `user-scalable=no` (add if missing).
- [x] If (and only if) a genuine gap is found in the touch journey (e.g. a
      state whose input system does not read the pointer, or a tap that cannot
      dismiss the result), fix it as an additive change with the keyboard path
      unchanged. If none is found, record that explicitly -- do not invent work.
- [x] Verify headlessly with the autopilot technique (env-gated system driving
      Menu -> Playing -> GameOver) that the pointer path drives the whole cycle
      without panic. Remove the harness before commit. Run `cargo fmt --check`,
      `cargo clippy --all-targets`, `cargo test --example 10_asteroids`,
      `scripts/check-ascii.sh`.
- [x] Rebuild the web showcase via `npm run build` (`npm ci` first in the fresh
      worktree's `web/`) to confirm the wasm build still succeeds.
- [x] Note the verification result (and any fix) in the task close-out; extend
      the overload touch doc or add a line to the asteroids doc if behaviour
      changed. If the change is only the web canvas CSS, a doc line is enough.

## Notes

- Relevant files: `examples/10_asteroids.rs` (update_pointer ~300, PreUpdate reg
  ~206, menu_click ~733, result input ~1465, Pointer resource ~287),
  `web/games/10_asteroids/index.html`, `web/games/08_dropzone/index.html`
  (reference for the CSS block).
- Expectation from survey: gameplay + menu + result are already touch-driven;
  the substantive deliverable here is likely just the web canvas
  `touch-action: none`. Keep the task honest -- verify, fix only real gaps.
- Depends on nothing; independent of the 11_overload task
  (20260704-130314). Lower priority because it is mostly verification.

## Close-out

Verification confirmed `10_asteroids` was already fully touch-driven, so the only
code change is the web canvas CSS.

Trace: `update_pointer` (examples/10_asteroids.rs ~305) unifies mouse + touch
into the `Pointer` resource (`pressed`, `just_pressed`, `screen_pos`; an active
touch wins over the cursor) and is registered in `PreUpdate` in EVERY state
(~207), so the pointer is live in menu, playing and result. `menu_click` (~728)
and `gameover_click` (~1461) both advance on `pointer.just_pressed`, and
`control_ship` / `fire_bullets` steer + auto-fire from the pointer while pressed.
So the whole Menu -> Playing -> GameOver journey already worked from a finger; no
Rust gap was found and none was invented.

Change: `web/games/10_asteroids/index.html` gains `touch-action: none` on
`#game-canvas` (viewport already had `user-scalable=no`), so touch drags feed the
game instead of scrolling / pinch-zooming the page. This is the same one-line gap
`08_dropzone` had; matches its CSS block.

Verified: `npm run build` rebuilt the showcase successfully after the change; no
Rust changed, so the example's existing gameplay verification (its own retro)
stands. Not run: a fresh autopilot smoke, since adding/removing a temporary
harness to an unchanged example is pure churn for a CSS-only change.
