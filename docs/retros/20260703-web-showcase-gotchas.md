# Retro: web showcase - avoidable gotchas

- SCOPE: the wasm game showcase feature (tasks 20260703-144934 / -144936 /
  -153128, plus the build-games.sh fix in commit 64a3d7e)
- BRANCH: web-showcase (merged to master)

This is a focused retro on the small issues that cost time and would have been
avoidable with foreknowledge, per the request. Process notes for the individual
tasks live in their own retros (wasm-builds, showcase-site, embed-size-title).

## The gotchas (and how to avoid them next time)

1. **trunk must run from the repo root, or it can't find the project.** The
   showcase shipped a broken build: `npm run build` failed for the user with
   `Unable to find any Trunk configuration`. `build-games.sh` was correct except
   that trunk resolves its target/cargo project relative to the *current
   directory*, and npm runs the script from `web/`, not the repo root. Root
   cause: I only ever verified the script by running it *myself from the repo
   root* (where it happened to work), never through the actual user entry point
   `npm run build`. Fix was one line (`cd "$repo_root"`); the miss was
   testing-the-wrong-entry-point, not the code. Lesson: **verify a script/tool
   through the exact command a user runs it with (the npm script, the CI step),
   not a hand-rolled invocation from a convenient directory.** Documented in
   docs/wasm-web-builds.md.

2. **A piped `| tail` hides a build's real exit code.** The first de-risk wasm
   build reported "exit 0" while actually failing on getrandom, because it was
   `cargo build ... | tail`. This is the *second* time this bit me this session
   (also in the wasm-builds retro). It graduates from a one-off to a rule -
   proposed as an AGENTS.md verification note. Rule: for any build/command whose
   pass/fail matters, redirect to a file and check `$?`; reserve `| tail` for
   interactive peeking.

3. **`rand` on wasm needs the getrandom JS backend, in two places.** getrandom
   0.3 has no default backend on `wasm32-unknown-unknown`; a plain wasm build of
   any example using `rand` fails to compile. It needs BOTH a wasm-only
   `getrandom = { features = ["wasm_js"] }` dep AND `--cfg
   getrandom_backend="wasm_js"` in `.cargo/config.toml`. Predicted, but still
   cost a failed build to confirm. Now captured in docs/wasm-web-builds.md so
   the next wasm example does not rediscover it.

4. **Node was not in the flake devshell.** I had to drive `npm` through a
   hardcoded `/nix/store/...nodejs.../bin` path, then added `nodejs` to the
   flake afterward. The store path is fragile (changes on a flake update).
   Lesson: when a task needs a tool that is not in the devshell, add it to the
   flake *first*, then use `nix develop`, rather than reaching for a store path.

5. **Bevy wasm is big and slow, know the numbers up front.** dev-profile wasm is
   ~380 MB (unoptimized + debuginfo); release + wasm-opt is ~42-50 MB and takes
   a multi-minute compile. Knowing this earlier would have set build-time
   expectations and avoided a couple of "is it hung?" checks. Documented.

## Action items

- [x] Added the trunk-must-run-from-repo-root gotcha to
  docs/wasm-web-builds.md (getrandom + wasm sizes were already there).
- [x] Proposed AGENTS.md verification note: never judge a build by a piped
  `| tail`'s exit code (second occurrence this session).
- [ ] If a script's real entry point is an npm/CI wrapper, add a note or a
  smoke step that runs *that* wrapper, not just the underlying tool.
