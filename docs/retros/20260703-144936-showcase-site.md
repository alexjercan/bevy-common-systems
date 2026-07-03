# Retro: webpack/TS showcase gallery site

- TASK: 20260703-144936 (committed on web-showcase, 1 self-review round, APPROVE)

## What went well
- Reused football-guessr's conventions wholesale (webpack + ts-loader +
  html-webpack-plugin + CopyPlugin + prettier/eslint, asyncWebAssembly,
  PUBLIC_PATH), so the scaffold was a known-good shape rather than invented.
- Solved the dist-clean-vs-games conflict up front with a staging dir
  (web/build/games) that webpack copies from, so `clean: true` can't wipe the
  wasm - decided during task 1 planning, paid off here.
- Kept a single base-path source of truth: gallery links, webpack publicPath,
  and the trunk --public-url all key off PUBLIC_PATH, so a Pages subpath works
  without per-link fixups.
- Ran node from the nix store path to drive npm even though node wasn't on the
  devshell PATH yet, then added nodejs to the flake for reproducibility - didn't
  block on the missing tool.

## What went wrong
- Nothing blocking. Could not do a real browser render test of the 42 MB wasm
  headlessly; verified the static site serves (all routes 200) and left the
  visual game smoke test to the user.

## Improve next time
- For heavy wasm (42 MB), consider noting load-time expectations in the UI (a
  loading state in the iframe overlay) - a future polish, not filed.
