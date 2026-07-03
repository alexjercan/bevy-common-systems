# Review: webpack/TS showcase gallery site

- TASK: 20260703-144936
- BRANCH: web-showcase (sprout worktree)

## Round 1

- VERDICT: APPROVE

Self-review. The site is intentionally small: one gallery page, a typed `games`
data model, and an iframe overlay per game (Esc/close). Build is PUBLIC_PATH-
aware end to end - the gallery links, webpack publicPath, and the per-game trunk
`--public-url` all key off the same base, so it works under a Pages subpath.
Games stage to web/build/games and are copied into dist by webpack (its clean
can't wipe them). Verified: npm ci/install, webpack build (exit 0), strict TS
compile, eslint + prettier clean, and a Node static-serve smoke test (/,
/games/06_fruitninja/, /index.js all 200). Pages workflow is workflow_dispatch
only, so nothing deploys without the user. Browser render of the 42 MB wasm is
the user's smoke test.

- Extensible as designed: adding a game is 3 small edits (games.ts, the build
  script array, a trunk index.html).
