# Review: Ship game audio assets into the wasm web build so sounds load

- TASK: 20260703-163328
- BRANCH: feature/web-audio

## Round 1

- VERDICT: APPROVE

Diff reviewed: `git diff master...HEAD` -- one `data-trunk rel="copy-dir"`
directive in `web/games/06_fruitninja/index.html` plus doc updates
(`docs/wasm-web-builds.md`, `web/README.md`, `assets/sounds/README.md`). No
Rust touched.

Verified independently (not trusting the implementer's summary):

- **Delivers the Goal.** Built via `web/scripts/build-games.sh` from the repo
  root AND from the `web/` subdir (npm's cwd): both exit 0 and land all eight
  WAVs at `web/build/games/06_fruitninja/assets/sounds/*.wav` with plain
  filenames (no trunk hashing).
- **End-to-end through the real entry point.** Ran `npm install && npm run
  build:web` (the webpack half that produces the served `dist/`): exit 0, and
  the sounds arrive at `dist/games/06_fruitninja/assets/sounds/*.wav`. This was
  the one real risk -- the served artifact is webpack's `dist/`, not the trunk
  staging dir -- and copy-webpack-plugin's recursive dir copy carries the
  `assets/` subtree through as expected. Confirmed, not assumed.
- **Path matches the runtime fetch.** Build-time `data-target-path=assets/sounds`
  agrees with the URL Bevy's wasm `AssetServer` requests
  (`<public>/games/06_fruitninja/assets/sounds/<file>`), and because the fetch
  is relative to the page it also resolves under a Pages subpath.
- **Docs are accurate** against observed behavior (trunk copies contents of the
  href dir into the target-path; copies nothing by default). ASCII-clean;
  `cargo fmt --check` and `check-ascii.sh` pass. Rust CI matrix is unaffected
  (no `.rs` changed).

Findings:

- [ ] R1.1 (NIT) web/games/06_fruitninja/index.html:29 - the `copy-dir` also
  ships `assets/sounds/README.md` into the dist (it lands at
  `dist/games/06_fruitninja/assets/sounds/README.md`). Harmless and tiny, but
  it is a stray non-asset in the published build. Optional: leave it (simplest,
  the directive stays a one-liner) or switch to per-file `copy-file` directives
  / an exclude if a pristine dist matters. Not blocking.
  - Response: Leaving as-is. Shipping the tiny README keeps the copy-dir a single self-documenting line; a stray 1.4 KB text file in the dist is harmless and not worth per-file directives or an exclude glob. (implementer)

No BLOCKER/MAJOR/MINOR findings. The diff is small, correct, and delivers the
task goal; the audio *audibility* check in a browser is correctly deferred to
task 20260703-163329. Approved.
