# Enable wasm example builds + trunk build of fruit ninja

- STATUS: OPEN
- PRIORITY: 100
- TAGS: feature,web

## Goal

Make the crate's examples buildable for `wasm32-unknown-unknown` and produce a
self-contained static wasm page for `06_fruitninja` using trunk. This is the
foundation the showcase site embeds.

## Steps

- [ ] Fix the `getrandom` wasm backend so examples compile for wasm. getrandom
      is 0.3.4, which needs BOTH the `wasm_js` feature and the
      `--cfg getrandom_backend="wasm_js"` rustflag:
  - add `getrandom = { version = "0.3", features = ["wasm_js"] }` to
    `[dependencies]` (or `[target.'cfg(target_arch = "wasm32")'.dependencies]`)
    so the feature is enabled;
  - append `--cfg getrandom_backend="wasm_js"` to the existing
    `[target.wasm32-unknown-unknown] rustflags` in `.cargo/config.toml`.
- [ ] Confirm `cargo build --example 06_fruitninja --target wasm32-unknown-unknown`
      succeeds (capture the real exit code -- do not pipe through `tail`, which
      hides it).
- [ ] Add a trunk entry for the game under `web/games/06_fruitninja/`:
      an `index.html` with the trunk rust link building the example
      (`<link data-trunk rel="rust" data-bin="..."/>` or the `--example` form),
      a canvas, and minimal page CSS. Use `trunk build --example 06_fruitninja`
      (trunk bundles wasm-bindgen; no separate CLI needed).
- [ ] Add a build script (e.g. `web/scripts/build-games.sh`) that runs trunk for
      each game into the site's games output dir, parameterized by a public base
      path (for GitHub Pages). Start with just 06_fruitninja; make adding a game
      a one-line change.
- [ ] Verify the trunk build produces a working `dist` for the game (wasm + JS
      glue + index.html). A real browser run needs the user; at minimum confirm
      the artifacts exist and the wasm is non-trivial in size.
- [ ] Document the wasm build in `docs/` (how to build a game for the web, the
      getrandom gotcha).

## Notes

- Tooling present in the nix devshell: `trunk`, `wasm-pack`, wasm32 target,
  `.cargo/config.toml` already sets `--cfg=web_sys_unstable_apis`.
- `trunk build --example <EXAMPLE>` is supported (verified via `trunk build
  --help`).
- Bevy wasm canvas: the app needs a canvas; Bevy 0.18 picks up a canvas via the
  window settings or a default `#bevy` canvas -- confirm the selector during
  work and wire the index.html canvas id to match, or set
  `Window { canvas: Some("#...".into()), .. }` in a wasm-only tweak if needed.
- Only 06_fruitninja for now (user scope); keep the script/layout ready for
  01/02/03/05 later. 04_status_item shells out to `uname` and is intentionally
  excluded.
- This is Rust/build-side only; the webpack gallery is task 20260703-144936.
- No changes to the game's gameplay code.
