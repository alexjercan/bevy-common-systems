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

Adding a game later is a small change: a `web/games/<name>/index.html` (copy
06_fruitninja's) and one entry in the `games` array in
`web/scripts/build-games.sh`. If the game loads any assets (sounds, textures,
fonts), also add a `copy-dir` directive to its `index.html` so the files ship
into the build -- see "Assets (sounds, textures, ...)" below.

### Assets (sounds, textures, ...)

Trunk copies **nothing** into the build by default: a plain build emits only
`index.html`, the JS glue and the `.wasm`. Any asset the example loads at
runtime -- for example `06_fruitninja` calling
`asset_server.load("sounds/menu_select.wav")` -- must be staged into the dist
explicitly, or the browser fetch 404s and (for audio) every sound is silent.

Stage assets with a `data-trunk rel="copy-dir"` link in the game's
`index.html` (alongside the `rel="rust"` link). `06_fruitninja` copies the
crate's sound directory:

```html
<link
  data-trunk
  rel="copy-dir"
  href="../../../assets/sounds"
  data-target-path="assets/sounds"
/>
```

- `href` is relative to the `index.html`; `../../../` reaches the repo root
  from `web/games/<name>/`.
- `data-target-path` is the destination **inside the dist dir**; trunk copies
  the *contents* of `href` into it, so this lands the files at
  `web/build/games/<name>/assets/sounds/*.wav`.
- The path must match what Bevy's wasm `AssetServer` fetches at runtime. Bevy
  uses `file_path = "assets"` by default and fetches relative to the page, so
  with the game served at `<public>/games/<name>/` it requests
  `<public>/games/<name>/assets/sounds/<file>`. Keep the build-time
  `data-target-path` (`assets/sounds`) and this runtime URL in agreement.

Copying the whole `assets/` dir (`href="../../../assets"`,
`data-target-path="assets"`) also works and generalizes to games that load more
than sounds; `06_fruitninja` copies only `assets/sounds` because that is the
sole thing it loads (it is otherwise fully procedural).

Web audio additionally needs a user gesture before it will play; the showcase
satisfies this via the in-canvas click that starts a run. See "Audio and the
autoplay policy" below.

### Audio and the autoplay policy

Browsers block Web Audio until the user interacts with the page: an
`AudioContext` created before any user gesture starts in the `suspended` state.
Bevy creates its audio context eagerly at startup (before any gesture), so it
comes up suspended. Chrome and Firefox then auto-resume it once two things are
true -- the user has interacted with the document, and a source node's
`start()` has been called (which rodio/cpal do on every sound) -- so no
explicit `resume()` call is needed.

For the showcase this is satisfied for free: `06_fruitninja` plays its first
sound (`menu_select`) on the in-canvas click that starts a run, which is a real
user gesture inside the iframe's own document, so the context resumes on that
click and every later sound is audible.

Two things make this work, both already in place:

- The gesture must happen inside the iframe's document. Clicking a gallery card
  in the parent page only sets the iframe `src`; it does not unlock the child's
  audio. The in-canvas start click does.
- The game iframe carries `allow="autoplay; fullscreen; gamepad"`
  (`web/src/index.html`), which delegates autoplay to the frame -- relevant if
  a game is ever served cross-origin (same-origin frames allow it by default).

A game that needs sound *before* any user gesture (menu music on load, say)
cannot rely on this -- the context stays suspended until the first interaction.
Bevy does not expose its `AudioContext`, so the practical options are to gate
the first sound behind a click/keypress (as fruitninja's menu does) or to
resume the context from JS in the host HTML on a canvas `pointerdown`.

Known quirk: bevyengine/bevy#15273 (0.14 era) reports a Bevy app embedded in an
iframe occasionally dropping the very first sound -- a loading/timing issue, not
the autoplay policy. If a sound rarely fails to fire on the first click, that is
the likely cause, and a JS `resume()` shim on the canvas gesture is the cheap
insurance.

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
