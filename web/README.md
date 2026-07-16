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
   (`06_fruitninja` copies `assets/sounds` for its SFX). See
   `docs/wasm-web-builds.md`.

See `docs/wasm-web-builds.md` for the wasm build details (including the
`getrandom` gotcha and how assets are staged).
