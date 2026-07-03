# Ship game audio assets into the wasm web build so sounds load

- STATUS: IN_PROGRESS
- PRIORITY: 90
- TAGS: feature,web,wasm,audio

## Goal

Make the sound effects in the `06_fruitninja` web (wasm) build actually load,
by shipping the game's audio assets into the trunk build output. Today the
trunk build emits only the `index.html`, the JS glue and the `.wasm`; nothing
copies `assets/sounds/*.wav` into the dist, so every
`asset_server.load("sounds/*.wav")` 404s in the browser and all SFX are silent.

"Done" = after `bash web/scripts/build-games.sh`, the game's sound files are
present in the build output at the exact URL the wasm `AssetServer` fetches, so
a browser build of `06_fruitninja` can load them (playback itself is confirmed
in task 20260703-163329). The mechanism must generalize so future
showcased games that carry assets ship them too, and the approach is
documented.

## Steps

- [x] Confirm the fetch path. `build-games.sh` builds with
      `--public-url <public>/games/06_fruitninja/`, and Bevy's wasm
      `AssetServer` uses `file_path = "assets"` by default and fetches relative
      to the page, so it requests `<public>/games/06_fruitninja/assets/sounds/*.wav`.
      The copied files must land at `<dist>/assets/sounds/*.wav` inside the
      per-game dist dir. Verify this against the actual emitted paths, do not
      assume.
- [x] Add a trunk asset-copy directive to `web/games/06_fruitninja/index.html`
      so trunk stages the crate's audio assets into the dist under `assets/`.
      Prefer a scoped copy of the sounds the game uses, e.g.
      `<link data-trunk rel="copy-dir" href="../../../assets/sounds"
      data-target-path="assets/sounds" />` (href is relative to the
      index.html; `../../../` reaches the repo root from
      `web/games/06_fruitninja/`). Copying the whole `assets/` dir is also
      acceptable if simpler and still correct; pick the option that keeps the
      dist minimal and the fetch path correct, and note the choice in the
      index.html comment next to the existing `rel="rust"` link.
- [x] Decide whether the copy belongs in the per-game `index.html` (trunk
      copy-dir, keeps each game page self-contained -- matches the existing
      "adding a game is a two-line change" ethos) or in `build-games.sh` /
      webpack. Default to the per-game `index.html` trunk directive unless
      there is a concrete reason not to; record the rationale in the Outcome.
- [x] Build and inspect the output: run `bash web/scripts/build-games.sh` (in
      `nix develop`), then confirm the wav files exist under
      `web/build/games/06_fruitninja/assets/sounds/` (list them) and that a
      trunk asset hash/rewrite, if any, still leaves them fetchable at the path
      the AssetServer uses. Redirect build output to a file and check the exit
      code -- do NOT pipe the build through `| tail` (a piped build hides the
      real exit code; see docs/retros/20260703-web-showcase-gotchas.md).
- [x] Generalize for future games: update the "adding a game" guidance so a new
      showcased game with assets does not rediscover this wall. Add an
      assets/copy-dir note to the checklist in `web/README.md` and to the
      per-game section of `docs/wasm-web-builds.md`, and if `build-games.sh`
      carries an "adding a game is a two-line change" comment, extend it to
      mention copying assets when the game loads any.
- [x] Document the web-audio asset path in `docs/wasm-web-builds.md`: a short
      "Assets (sounds, textures)" note explaining that trunk copies nothing by
      default, that assets must be staged via a `copy-dir` directive, and the
      exact fetched URL (`<public>/games/<name>/assets/...`).
- [x] Keep CI green: `cargo build`, `cargo clippy --all-targets`,
      `cargo clippy --all-targets --features debug`, `cargo fmt --check`,
      `cargo test`, `cargo test --features debug`, `./scripts/check-ascii.sh`.
      (The wasm build itself is not part of `cargo` CI, but must succeed via
      `build-games.sh`.)

## Notes

- Root-cause and file map from the investigation:
  - `web/games/06_fruitninja/index.html` -- only has
    `<link data-trunk rel="rust" href="../../../Cargo.toml" data-wasm-opt="z" />`;
    no asset copy directive. This is where the fix most likely lands.
  - `web/scripts/build-games.sh` -- `trunk build --release --example
    06_fruitninja --public-url <public>/games/06_fruitninja/ --dist
    web/build/games/06_fruitninja <html_dir>/index.html`.
  - `web/webpack.config.js` -- CopyPlugin copies `web/build/games` into
    `dist/games` afterwards (webpack owns/cleans `dist/`).
  - `assets/sounds/*.wav` -- eight committed placeholder WAVs (bomb, slice,
    combo, golden, menu_select, game_over, launch, splat) plus a README.
  - `examples/06_fruitninja.rs` ~L515-528 -- loads the eight handles via
    `asset_server.load("sounds/<name>.wav")`.
  - `Cargo.toml` L24 -- `bevy` dev-dep has the `wav` feature, so the decoder is
    compiled into the example's wasm; format decoding is NOT the problem.
- fruitninja is otherwise fully procedural (no textures/fonts), so sounds are
  the only asset it loads -- which is why the web build works today except
  audio.
- trunk `copy-dir` with `data-target-path` places files under `<dist>/<target>`
  regardless of `--public-url`; the AssetServer's runtime URL is
  `<public-url><file_path>/...`. Make sure `data-target-path` (build-time dir)
  and the fetched URL (runtime) agree on `assets/sounds/...`.
- Do not push. Work happens on branch `feature/web-audio` in this worktree.
- This is the must-fix; task 20260703-163329 depends on it and
  confirms audio actually plays in a browser.

## Outcome

Added a single `data-trunk rel="copy-dir"` directive to
`web/games/06_fruitninja/index.html` (with an explanatory comment beside the
existing `rel="rust"` link) that stages `assets/sounds/` into the build:

```html
<link data-trunk rel="copy-dir" href="../../../assets/sounds"
      data-target-path="assets/sounds" />
```

Approach chosen: the per-game `index.html` trunk directive, not a
`build-games.sh` / webpack change. Rationale -- it keeps each game page
self-contained (the game and everything it needs to load live in one place),
matches the existing "adding a game" ethos, and lets each game declare exactly
the assets it uses. `06_fruitninja` copies only `assets/sounds` because that is
the sole thing it loads (the game is otherwise fully procedural); the docs note
that copying the whole `assets/` dir is the option for a game that loads more.

Verification (build output inspected, exit code checked from a file -- not
piped through `| tail`, per the web-showcase retro):
- `PUBLIC_PATH=/ bash web/scripts/build-games.sh` from the repo root: exit 0,
  and all eight WAVs land at `web/build/games/06_fruitninja/assets/sounds/*.wav`
  with plain filenames (no trunk hashing/rewrite).
- Re-ran from the `web/` subdir (`cd web && ... bash scripts/build-games.sh`),
  the exact cwd `npm run build:games` uses: exit 0, sounds present. This is the
  precise gotcha that shipped a broken build before (the retro's
  "verify through the real entry point" lesson); the script's own `cd
  "$repo_root"` makes it cwd-independent and the copy-dir works either way.
- Confirmed the build-time destination (`assets/sounds/`) matches the runtime
  URL Bevy's wasm `AssetServer` fetches: `<public>/games/06_fruitninja/`
  (from `--public-url`) + `assets/` (default `file_path`) + `sounds/<file>`.

Note on trunk copy-dir semantics (empirically confirmed, since the docs are
terse): with `href=".../assets/sounds"` and `data-target-path="assets/sounds"`,
trunk copies the *contents* of the source dir into `<dist>/assets/sounds/` --
it does not re-nest the leaf dir name -- so files land at
`assets/sounds/*.wav`, not `assets/sounds/sounds/*.wav`.

Docs: added an "Assets (sounds, textures, ...)" section to
`docs/wasm-web-builds.md` (trunk copies nothing by default, the copy-dir
directive, the exact fetched URL, and a pointer that web audio needs a user
gesture); extended the "adding a game" checklist in `web/README.md` with an
assets step; and added a "Web (wasm) builds" note to `assets/sounds/README.md`.

No Rust changed (HTML + Markdown only), so the Rust CI suite is unaffected;
`cargo fmt --check` and `./scripts/check-ascii.sh` pass and all edited docs are
plain ASCII. The full `cargo clippy`/`cargo test` matrix is identical to master
since no `.rs` file was touched.

Not done here: confirming sound is actually *audible* in a browser (needs a
graphical session and covers the autoplay policy) -- that is task
20260703-163329.

Difficulties: none of note. The one real unknown was trunk's copy-dir
target-path nesting behavior, resolved by building and listing the output
rather than guessing.
