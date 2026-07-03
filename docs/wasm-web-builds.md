# Building the examples for the web (wasm)

The crate's examples can be built for `wasm32-unknown-unknown` and shipped as
static pages. This is what the showcase site under `web/` embeds.

## The getrandom gotcha

`rand` 0.9 pulls in `getrandom` 0.3, which has **no default backend on
`wasm32-unknown-unknown`** -- a plain wasm build fails with
`could not compile getrandom`. Two things are needed, and both are already
configured in-repo:

1. The `wasm_js` feature on `getrandom`, enabled only for wasm via a
   target-scoped dependency in `Cargo.toml`:

   ```toml
   [target.'cfg(target_arch = "wasm32")'.dependencies]
   getrandom = { version = "0.3", features = ["wasm_js"] }
   ```

2. The matching backend cfg in `.cargo/config.toml`:

   ```toml
   [target.wasm32-unknown-unknown]
   rustflags = [
       "--cfg=web_sys_unstable_apis",
       "--cfg=getrandom_backend=\"wasm_js\"",
   ]
   ```

Native builds are unaffected (the dependency and rustflag are wasm-only).

## Building a game with trunk

`trunk` (in the nix devshell) compiles an example to wasm, runs `wasm-bindgen`,
optimizes with `wasm-opt`, and emits a static `dist/` (an `index.html`, a JS
glue file, and the `.wasm`). The per-game trunk source lives in
`web/games/<name>/index.html`.

Build all showcased games:

```sh
PUBLIC_PATH=/ bash web/scripts/build-games.sh
```

This writes each game to `web/dist/games/<name>/`. `PUBLIC_PATH` must match the
gallery's base path (use the repo's Pages subpath in CI). The default is `/`
for local serving.

Adding a game later is a two-line change: a `web/games/<name>/index.html` (copy
06_fruitninja's) and one entry in the `games` array in
`web/scripts/build-games.sh`.

### trunk must run from the repo root

`trunk` resolves its target and the cargo project relative to the current
directory, and fails with `Unable to find any Trunk configuration` when run from
a subdirectory like `web/` -- even if you pass a correct absolute path to the
`index.html`. `build-games.sh` therefore `cd`s to the repo root before invoking
trunk, so it works no matter where it is launched from (in particular
`npm run build:games`, which npm runs from `web/`). Keep that `cd` if you edit
the script.

## Notes

- Bevy creates its own canvas on wasm; the page CSS stretches it to fill the
  frame. If precise canvas fitting is ever needed, set
  `Window { fit_canvas_to_parent: true, canvas: Some("#...".into()), .. }` in a
  `#[cfg(target_arch = "wasm32")]` tweak -- not needed for the current embed.
- `04_status_item` shells out to `uname` (`std::process::Command`) and cannot
  run in a browser, so it is intentionally excluded from the web builds.
- The dev-profile wasm is huge (~380 MB, unoptimized + debuginfo); always build
  the site with `--release` (as `build-games.sh` does) for a shippable size.
