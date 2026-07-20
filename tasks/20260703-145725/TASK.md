# Fix bevy 0.19 build: FontSize enum in ui/status.rs

- STATUS: CLOSED
- PRIORITY: 90
- TAGS: bug,bevy-migration,historical

## Goal

After the bevy 0.18 -> 0.19 bump, `TextFont::font_size` is no longer an `f32`;
it is a `FontSize` enum (`FontSize::Px`, `Vw`, `Vh`, `VMin`, ...). Three
`font_size: 14.0` literals in `src/ui/status.rs` no longer compile. Make the
crate compile again by wrapping them in the pixel variant, preserving the
existing 14px behavior.

## Steps

- [x] In `src/ui/status.rs`, change the three `font_size: 14.0` sites (the
      StatusBarItem prefix, value and suffix `TextFont` at lines ~231, ~241,
      ~251) to `font_size: FontSize::Px(14.0)`.
- [x] Add the `FontSize` import if it is not already covered by the existing
      `bevy::prelude::*` glob (check whether `FontSize` is in the prelude; if
      not, import it explicitly). Let `cargo fmt` order the imports.
- [x] Verify the module compiles: `cargo build` (these three E0308 errors gone).
- [x] Full check suite: `cargo fmt --check`, `cargo clippy --all-targets`
      (+ `--features debug`), `cargo test`, `./scripts/check-ascii.sh`.

## Notes

- Errors: `E0308 mismatched types: expected FontSize, found floating-point
  number` at `src/ui/status.rs:231`, `:241`, `:251`.
- Keep the value at 14 pixels -- `FontSize::Px(14.0)` is the direct equivalent
  of the old `14.0`.
- No new dependencies; this is a pure API-rename fix.

## Close-out

Wrapped the three `font_size: 14.0` literals as `FontSize::Px(14.0)`. `FontSize`
resolves through the existing `bevy::prelude::*` glob -- no explicit import
needed. Full suite green (fmt, clippy default + debug, tests, ascii).
