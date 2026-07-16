# Web builds

The [example games](../examples/) ship as playable WebAssembly builds through the
showcase site in [`web/`](https://github.com/alexjercan/bevy-common-systems/tree/master/web).
This page is how that site is put together and how to run the whole thing
locally.

## The showcase site

The site is a small TypeScript + [webpack](https://webpack.js.org) app. It has
three parts:

- the **landing page** (`/`) -- the pitch and the module grid;
- these **docs** (`/wiki/`) -- one markdown page per module, rendered to HTML at
  build time;
- the **examples gallery** (`/play/`) -- a card grid that embeds each game in a
  full-screen, itch.io-style page.

The games themselves are compiled separately by [Trunk](https://trunkrs.dev) and
copied into the site under `/games/<example>/`; the gallery iframes them.

## Building the games

`web/scripts/build-games.sh` builds each showcased example to a self-contained
static wasm page with Trunk:

```sh
cd web
npm run build:games
```

Each game builds into a staging dir (`web/build/games/<example>/`); webpack's
copy step then folds it into `dist/games/`. This needs the wasm toolchain --
Trunk and the `wasm32-unknown-unknown` target, both provided by the repo's Nix
devshell.

## Serving the site

For live site development, build the games once, then run the webpack dev server:

```sh
cd web
npm install
npm run build:games   # once (or whenever a game changes)
npm run serve         # http://localhost:8080
```

The dev server rewrites the clean directory URLs (`/play/`, `/wiki/<slug>/`) to
their generated pages, so the site behaves the same locally as when deployed.

To build the full static site into `web/dist/`:

```sh
npm run build         # build:games then build:web
```

## Deploying

The site deploys to GitHub Pages under a subpath, so every asset and inter-page
link has to resolve under `/<repo>/`. Set `PUBLIC_PATH` to that subpath for both
the game build and the webpack build so the two agree:

```sh
PUBLIC_PATH=/bevy-common-systems/ npm run build
```

Locally `PUBLIC_PATH` defaults to `/`. The [persist](../persist/) module's wasm
backend uses `localStorage`, so high scores in games like [Glide](../examples/)
survive a reload even in the browser build.
