# Add non-ASCII guard (pre-commit or CI) enforcing plain-ASCII rule

- STATUS: IN_PROGRESS
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
