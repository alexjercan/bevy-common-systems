# bevy_common_systems web showcase

A small static site (TypeScript + HTML + webpack) that showcases the crate. It
has three parts:

- **Landing** (`/`) -- the pitch, an install line, and a module grid rendered
  from the docs manifest so it never drifts.
- **Docs** (`/wiki/`) -- a "bevy book" style handbook: one page per module, each
  written in markdown under `src/wiki/` and rendered to HTML at build time
  (`markdown.js`), with a manifest-driven sidebar, search and see-also
  (`src/wiki.ts` + `src/wiki-pages.ts`). No-JS readers still get the full
  article; the JS only adds the chrome.
- **Examples** (`/play/`) -- a gallery of cards; clicking one opens the game's
  wasm build in a full-screen iframe overlay (`src/games-page.ts` + `games.ts`).

The shared header/footer live in `src/_header.html` / `src/_footer.html` and are
injected into every generated page by `webpack-partials.js`. The visual language
is a sharp, Bevy-inspired engine/tooling theme in `src/style.css`.

## Adding a docs page

1. Drop `src/wiki/<slug>.md` (start with a `# Title` H1).
2. Add a `{ slug, title }` entry to `WIKI_DOC_PAGES` in `webpack.config.js`.
3. Add a matching `WikiPage` entry to `src/wiki-pages.ts` (category, tags,
   summary, related, headings) so the sidebar, search and index pick it up.

Cross-link between docs pages with relative links (`[mesh](../mesh/)`) so they
resolve under any `PUBLIC_PATH`.

## Build

Requires the repo's nix devshell (for `trunk` + the wasm target) and Node.js
(added to the devshell in `flake.nix`).

```sh
cd web
npm install
npm run build        # builds the games (trunk) then the gallery (webpack)
```

The combined static site lands in `web/dist/` (landing at the root, docs under
`dist/wiki/`, the gallery at `dist/play/`, each game under `dist/games/<name>/`).
Serve `dist/` with any static server, or use the dev server:

```sh
npm run serve        # http://localhost:8080  (run `npm run build:games` once first)
npm run serve:lan    # same, bound to 0.0.0.0 so other devices on the LAN can reach it
```

`serve` binds to localhost only; `serve:lan` binds to `0.0.0.0` (and sets
`--allowed-hosts all` so requests by LAN IP are not rejected) -- reach it from
another device at `http://<your-LAN-IP>:8080`.

`npm run build:games` and `npm run build:web` run the two halves separately.

## GitHub Pages

Set `PUBLIC_PATH` to the Pages subpath so all links (and the game wasm) resolve:

```sh
PUBLIC_PATH=/bevy-common-systems/ npm run build
```

The `.github/workflows/pages.yml` workflow does this and publishes `dist/`. It
is `workflow_dispatch` only (manual), so it never deploys on its own.

## Adding a game

1. Add a trunk source `web/games/<name>/index.html` (copy `06_fruitninja`'s).
2. Add the example to the `games` array in `web/scripts/build-games.sh`.
3. Add a `Game` entry to `web/src/games.ts`.
4. If the game loads assets (sounds, textures, fonts), add a
   `data-trunk rel="copy-dir"` link to its `index.html` so the files ship into
   the build -- trunk copies nothing by default and the fetches would 404
   (`06_fruitninja` copies `assets/sounds` for its SFX). See "Assets (sounds,
   textures, ...)" below.
5. If the game plays sound, also add the two audio-unlock lines from "Audio and
   the autoplay policy" below.

The rest of this file documents the wasm build itself: how an example becomes a
static page, and the browser-side gotchas that bit us getting there.

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

### trunk must run from the repo root

`trunk` resolves its target and the cargo project relative to the current
directory, and fails with `Unable to find any Trunk configuration` when run from
a subdirectory like `web/` -- even if you pass a correct absolute path to the
`index.html`. `build-games.sh` therefore `cd`s to the repo root before invoking
trunk, so it works no matter where it is launched from (in particular
`npm run build:games`, which npm runs from `web/`). Keep that `cd` if you edit
the script.

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
comes up suspended.

The browsers then split on how the context comes back:

- **Chrome and Firefox** auto-resume a suspended context once two things are
  true -- the user has interacted with the document, and a source node's
  `start()` has been called (which rodio/cpal do on every sound). No explicit
  `resume()` is needed there.
- **WebKit (iOS + macOS Safari)** does NOT auto-resume on `start()`. It only
  resumes a context when `resume()` is called synchronously inside a real
  user-gesture handler. Since Bevy never calls `resume()` (and does not expose
  its `AudioContext` from Rust), on Safari the context stays suspended and
  every SFX is silent -- even though the in-canvas start click is a real
  gesture. This bit mobile Safari specifically (task 20260703-200005).

Because Bevy hides its `AudioContext`, the fix lives in the host page, not in
Rust. It is a single shared script, `web/games/_shared/audio-unlock.js`, that
every game loads (before trunk's injected wasm loader, so it installs first):

- it wraps the `AudioContext` / `webkitAudioContext` constructor to record
  every context Bevy/cpal builds;
- on the first `pointerdown` / `touchend` / `mousedown` / `keydown` it calls
  `resume()` on each and starts a 1-sample silent buffer through it (WebKit's
  stricter unlock wants an actual node start inside the gesture), then detaches
  its listeners once the context reaches `running`.

It is a no-op on Chrome/Firefox (resuming a running context is harmless, the
silent buffer is inaudible), so desktop audio is unchanged.

Each game wires in the shared script with exactly two lines in its
`index.html`'s `<head>`:

```html
<script src="audio-unlock.js"></script>
<link data-trunk rel="copy-file" href="../_shared/audio-unlock.js" />
```

The `copy-file` link stages the shared file into the game's dist root (so the
runtime `src="audio-unlock.js"` resolves under the game's `--public-url`), and
the `<script>` must stay a plain, non-module, non-`defer` element: that runs
synchronously before trunk's deferred `<script type="module">` wasm loader, so
the constructor wrap is installed before Bevy builds its context. Any future
web game with sound adds those same two lines -- do NOT re-inline the shim.
It used to be copy-pasted into each `index.html` and drifted (06_fruitninja
gained the iOS media-channel fix below while 07_orbit and 08_dropzone kept an
older copy and stayed silent on iPhone, task 20260704-101920); the shared file
is what stops that from recurring.

**iOS ringer channel (WebKit bug 237322).** Resuming the context is necessary
but not sufficient on iOS. WebKit routes Web Audio output to the *ringer*
channel, which the iPhone's physical Ring/Silent switch mutes even at full
media volume -- so after the resume the tab shows a "playing audio" indicator
but stays silent when the switch is on Silent. The tell is exactly that: audio
indicator present, no sound. HTML5 `<audio>` elements play on the *media*
channel (which ignores the switch), and while one is playing iOS promotes the
whole session -- Web Audio included -- to media. So on iOS the shim also starts
a continuous looping, inaudible `<audio>` (a tiny base64 silent-WAV data URI)
on the first gesture, and pauses it while the tab is hidden. This is gated to
iOS (UA + `maxTouchPoints`, to catch iPadOS-as-Mac) so desktop browsers do not
grow a spurious "now playing" widget. Task 20260703-212303; refs
`swevans/unmute`, `feross/unmute-ios-audio`.

Two more things must also hold, both already in place:

- The gesture must happen inside the iframe's document. Clicking a gallery card
  in the parent page only sets the iframe `src`; it does not unlock the child's
  audio. The in-canvas start click does.
- The game iframe carries `allow="autoplay; fullscreen; gamepad"`
  (`web/src/index.html`), which delegates autoplay to the frame -- necessary on
  WebKit and relevant if a game is ever served cross-origin (same-origin frames
  allow it by default).

A game that needs sound *before* any user gesture (menu music on load, say)
cannot be unlocked at all until the first interaction -- the context stays
suspended regardless. Gate the first sound behind a click/keypress (as
fruitninja's menu does).

Known quirk: bevyengine/bevy#15273 (0.14 era) reports a Bevy app embedded in an
iframe occasionally dropping the very first sound -- a loading/timing issue, not
the autoplay policy. The shim's silent-buffer kick on the first gesture also
helps here.

## Wasm notes

- Bevy creates its own canvas on wasm; the page CSS stretches it to fill the
  frame. If precise canvas fitting is ever needed, set
  `Window { fit_canvas_to_parent: true, canvas: Some("#...".into()), .. }` in a
  `#[cfg(target_arch = "wasm32")]` tweak -- not needed for the current embed.
- `04_status_item` shells out to `uname` (`std::process::Command`) and cannot
  run in a browser, so it is intentionally excluded from the web builds.
- The dev-profile wasm is huge (~380 MB, unoptimized + debuginfo); always build
  the site with `--release` (as `build-games.sh` does) for a shippable size.
