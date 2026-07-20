# Review: harvest 13_glide UI juice into ui/animate

- TASK: 20260705-090557
- BRANCH: spike/glide-ui-juice-harvest

## Round 1

- VERDICT: APPROVE

Reviewed `git diff master...HEAD`: the new `src/ui/animate.rs`, the prelude
wiring, the 13_glide refactor, and the decision doc. Ran the full suite in the
worktree: `cargo fmt --check`, `cargo clippy --all-targets` (clean),
`--features debug` (clean), `cargo test` (96 unit + 52 doctests, incl. 2 new
animate tests), `cargo test --example 13_glide` (11 pass), `check-ascii.sh`.
Verified behaviour: the 13_glide autopilot cycle completes with no panic, and a
`ScreenshotPlugin` framebuffer grab shows the board rendering with both seed
tiles correctly positioned and at full size -- i.e. the crate `apply_offset` /
`apply_scale` appliers drive the Node fields.

Both halves of the spike are delivered: the two generalizable patterns promoted
as `ui/animate`, and the rolling-number readout kept local with a documented
reason. Notes are informational.

- [x] R1.1 (NIT) Verified: Behaviour is preserved: `apply_offset` / `apply_scale` /
  `apply_background` are line-for-line the old `apply_slide` / `apply_pop` /
  `apply_flash`, and `node_flash` reproduces `flash_tween` (white -> colour,
  QuadraticOut, Keep). The refactor is a pure move-to-crate, confirmed by the
  screenshot + autopilot.
- [x] R1.2 (NIT) Verified, good call: Markers over an auto-applier. Auto-applying
  every `Tween<Vec2>` to `Node.left/top` would steal the component from any game
  using `Tween<Vec2>` otherwise; the opt-in markers keep `tween` generic and match
  the crate's other opt-in shapes. The design doc explains this.
- [x] R1.3 (NIT) The crate appliers run every frame unconditionally, where
  13_glide gated the old ones on `in_state(Playing)`. Safe: outside Playing the
  board is despawned so the queries are empty, and the cost is negligible --
  the same argument the ui/menu `TitlePulse` harvest used. Called out for the
  record.
- [x] R1.4 (NIT) `apply_scale` bakes the "value * 100 as percent, clamped at 0"
  convention (grow-from-centre within a positioned parent). This is opinionated
  but is the common UI-pop case and matches 13_glide; a game wanting px sizing
  applies its own `Tween<f32>` (the tween module already supports that). Documented
  in the module doc.
- [x] R1.5 (NIT) Verified: `node_flash`'s end value (`color_to_vec4(target)`) is not
  unit-tested because advancing a `Tween` is private to the tween module; the test
  asserts the start-white convention instead, and the end is exercised by
  13_glide's merge flash. Reasonable given the API surface. New prelude names
  (`TweenNode*`, `UiAnimate*`, `node_flash`, `*_to_*`) are all novel -- no
  `bevy::prelude` collision.
- [x] R1.6 (NIT) Verified: Removing the `TileFace` marker is correct: after the
  appliers moved to the crate, nothing queried it, so it was dead weight.
