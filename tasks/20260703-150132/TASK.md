# Fix bevy 0.19 build: FontSize enum in examples/06_fruitninja.rs

- STATUS: CLOSED
- PRIORITY: 85
- TAGS: bug,bevy-migration,example,historical

## Goal

Same bevy 0.19 `TextFont::font_size` change as the library fix, but in the
`06_fruitninja` example: `font_size` is now `FontSize` instead of `f32`. Four
sites in `examples/06_fruitninja.rs` no longer compile. Wrap each in
`FontSize::Px(..)`, preserving current sizes.

## Steps

- [x] Line ~687 and ~744: `font_size: size` where `size: f32` ->
      `font_size: FontSize::Px(size)`.
- [x] Line ~826: `font_size: 40.0` -> `font_size: FontSize::Px(40.0)`.
- [x] Line ~844: `font_size: 34.0` -> `font_size: FontSize::Px(34.0)`.
- [x] Ensure `FontSize` resolves (via `bevy::prelude::*` or an explicit import).
- [x] Verify: `cargo build --example 06_fruitninja`, then the full suite
      `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features debug`),
      `cargo test`, `./scripts/check-ascii.sh`.

## Notes

- Discovered mid-flow: the library FontSize task only covered `src/ui/status.rs`;
  the example has its own copies. Errors E0308 at lines 687, 744, 826, 844.
- No new dependencies.

## Close-out

Wrapped the four `font_size` sites in `FontSize::Px(..)` (two carrying an `f32`
`size` variable, two literals 40.0 / 34.0). `FontSize` resolves through the
example's `bevy::prelude::*`. `cargo build --example 06_fruitninja` and the full
suite are green. Did not interactively play the game (windowed, needs a human at
the swipe input); covered by compile + clippy.
