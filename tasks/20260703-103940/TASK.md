# Add non-ASCII guard (pre-commit or CI) enforcing plain-ASCII rule

- STATUS: CLOSED
- PRIORITY: 10
- TAGS: chore

## Goal

AGENTS.md mandates plain ASCII in all code, comments and docs, but nothing
enforces it - non-ASCII typographic chars sat in chase.rs from the initial
commit until a grep happened to find them (20260703-101712). Add mechanical
enforcement. Per the maintainer's decision, this is the repo's first CI
workflow (GitHub Actions), running the full verified suite - fmt, clippy
(both feature configs), test (both feature configs) - plus the non-ASCII
guard, on push and pull_request.

## Steps

- [x] Add `scripts/check-ascii.sh`: fail with a listing when
      `grep -rnP '[^\x00-\x7F]'` matches under `src/`,
      `bevy_common_systems_macros/src/` and `examples/`. Reusable by CI and
      humans. `chmod +x`.
- [x] Add `.github/workflows/ci.yml`: on push + pull_request, ubuntu-latest,
      install the Bevy Linux build deps (`libasound2-dev`, `libudev-dev`),
      install nightly with rustfmt+clippy, cache with Swatinem/rust-cache,
      then run the documented suite: `cargo fmt --check`,
      `cargo clippy --all-targets`, `cargo clippy --all-targets --features
      debug`, `cargo test`, `cargo test --features debug`, and
      `./scripts/check-ascii.sh`.
- [x] Verify locally what is verifiable without pushing: the script passes
      on the clean tree and fails on a deliberately injected en-dash; every
      cargo command in the workflow passes; the workflow YAML parses.
- [x] Document in AGENTS.md: mention `scripts/check-ascii.sh` in the
      build/verify list and note that CI runs the suite on push/PR.

## Notes

- Discovered during 20260703-101712; see
  docs/retros/20260703-101712-chase-ascii.md.
- Maintainer chose full GitHub Actions CI over an ASCII-only workflow or a
  local hook (AskUserQuestion, this session). That widened the task from a
  single grep to standing up CI; scope updated accordingly.
- HONEST LIMIT: the runner environment (apt deps, nightly install, Bevy
  link on a headless ubuntu-latest) cannot be fully verified without
  pushing, which is the maintainer's call. Every *command* the workflow
  runs is verified locally; the workflow structure follows Bevy's
  documented Linux CI pattern (libasound2-dev + libudev-dev). First push
  is the real confirmation.
- Bevy needs system libs to build/link on Linux (alsa, udev); examples that
  open windows are compiled by `cargo test` but never run, so vulkan/x11/
  wayland are not needed in CI.

## Close-out

What changed:
- scripts/check-ascii.sh (executable): greps src/,
  bevy_common_systems_macros/src/ and examples/ for any non-ASCII byte,
  fails with a file:line:text listing. Shared by CI and humans.
- .github/workflows/ci.yml: the repo's first CI. On push + pull_request,
  ubuntu-latest: apt installs libasound2-dev + libudev-dev, installs
  nightly (rustfmt+clippy), caches with Swatinem/rust-cache, then runs the
  AGENTS.md suite (fmt, clippy x2, test x2) and the ascii guard. Adds a
  concurrency group to cancel superseded runs.
- AGENTS.md: added the ascii script to the verify list and a note that CI
  runs the suite on push/PR.

Verification (what a headless environment allows):
- actionlint 1.7.12 clean (validates the Actions schema and shellchecks the
  embedded run: blocks); shellcheck clean on the standalone script.
- All six workflow commands pass locally on this branch.
- ascii guard: passes on the clean tree, fails (exit 1, correct listing) on
  a deliberately injected en-dash in src/lib.rs, then passes after revert.
- Could NOT verify without pushing: the runner installing apt deps + nightly
  and Bevy linking on ubuntu-latest. Documented as first-push confirmation;
  pushing is the maintainer's call.

Alternatives considered (maintainer decided via AskUserQuestion):
- ASCII-only workflow: cheaper, but leaves fmt/clippy/test ungated by CI.
- Local pre-commit hook + install script: no CI infra, but hooks are not
  shared through the repo and must be installed per clone. Rejected.
- Full GitHub Actions CI (chosen): standing up CI was larger scope than the
  original "add a grep", but it enforces the whole documented suite, not
  just ASCII, which is the stronger guarantee.

Difficulties:
- No YAML/Actions validator was installed (no pyyaml, actionlint, yamllint,
  ruby, node). Resolved by running actionlint and shellcheck through
  `nix run nixpkgs#...`, since the repo already ships a Nix flake - so the
  workflow got real schema + shell validation, not just a hand read.

Self-reflection:
- The honest limit here is real: CI correctness is only fully provable by
  running it on the platform. The right move was to verify every separable
  piece locally (commands, YAML schema, shell) and state the one
  irreducible unknown plainly, rather than claim "CI works".
- Standing up the first CI workflow is a maintainer-level decision;
  stopping to ask before building (rather than defaulting to the task's
  "CI preferred" hint) was correct - the choice materially changed scope.
