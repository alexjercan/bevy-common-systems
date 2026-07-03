# Webpack/TS showcase gallery site (GitHub Pages)

- STATUS: CLOSED
- PRIORITY: 90
- TAGS: feature,web

## Goal

A simple static showcase site (TypeScript + HTML + webpack, matching the
football-guessr conventions) with a landing gallery of the crate's games as
cards; each card opens the game's wasm build (iframe or link). Ships a combined
static `dist/` ready for GitHub Pages, plus a publish workflow (no push).

## Steps

- [x] Scaffold `web/` as a webpack + TypeScript project mirroring
      `~/personal/football-guessr` conventions: `package.json` (build/serve/
      format/lint scripts), `webpack.config.js` (ts-loader, html-webpack-plugin,
      css, `PUBLIC_PATH` env for Pages, `asyncWebAssembly` on), `tsconfig.json`,
      prettier + eslint configs. Keep deps minimal (tailwind optional).
- [x] Define a small `games` data model in TS (id, title, blurb, thumbnail,
      path to the wasm build) and render the landing page (`src/index.html` +
      `src/index.ts`) as a grid of cards. Start with one entry: Fruit Ninja.
- [x] Each card opens the game: link to `games/06_fruitninja/` (built by task
      20260703-144934) or embed it in an iframe on a per-game page. Keep the
      integration iframe-based per the chosen design.
- [x] Combined build: a top-level script (npm script or `web/scripts/build.sh`)
      that (a) runs the game trunk builds into `web/dist/games/...` and (b) runs
      webpack to emit the gallery into `web/dist/`, all `PUBLIC_PATH`-aware so it
      works under a GitHub Pages subpath.
- [x] Add a GitHub Actions workflow (`.github/workflows/pages.yml`) that builds
      the site (nix devshell for trunk + node for webpack) and publishes `dist/`
      to Pages. Do NOT trigger a deploy; committing the workflow is enough.
- [x] Add `web/README.md` documenting how to build and serve locally
      (`npm run serve`) and how to add a new game (data entry + build script
      line). Update the repo root `AGENTS.md` with a pointer to `web/`.
- [x] Verify: `npm run build` produces `web/dist/` with the gallery + the game;
      `npm run format:check` and `npm run lint` pass; the game iframe path
      resolves under the configured `PUBLIC_PATH`. Browser smoke test is the
      user's; confirm artifacts and that webpack build is clean.

## Notes

- Depends on: 20260703-144934 (needs the game's trunk build to embed).
- football-guessr reference: webpack.config.js already uses
  `experiments.asyncWebAssembly`, `PUBLIC_PATH`, html-webpack-plugin,
  copy-webpack-plugin, ts-loader, prettier/eslint/jest. Mirror the structure;
  jest/tests optional for a static gallery (a light DOM test of the card
  rendering is a nice-to-have, not required).
- Node/npm: confirm availability in the devshell; if node is not in the nix
  shell, note it and use the system node, or add node to the flake in task
  144934's scope (prefer not to widen; document the requirement).
- Keep it genuinely simple: one gallery page + one game. The value is the
  extensible scaffold, not features.
- No push/deploy without the user asking.

## Close-out

Scaffolded `web/` as a webpack + TypeScript static site (football-guessr
conventions): package.json scripts, webpack.config.js (ts-loader, css,
html-webpack-plugin, CopyPlugin from the games staging dir, PUBLIC_PATH,
asyncWebAssembly), tsconfig (strict), prettier + flat eslint. `src/games.ts`
is the data model; `src/index.ts` renders a card grid and opens the selected
game in a full-screen iframe overlay (Esc / x to close). `npm run build` builds
the games (trunk) then the gallery (webpack) into `web/dist/`. Added a
workflow_dispatch-only Pages workflow, `nodejs` to the flake, `web/README.md`,
and an AGENTS.md pointer. Verified: npm install, webpack build (exit 0), strict
TS, eslint clean, prettier clean, and a static-serve smoke test (gallery + game
page + bundle all 200).
