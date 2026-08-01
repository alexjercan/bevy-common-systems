# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The version tracks Bevy's minor rather than classic Semantic Versioning:
`0.19.x` targets Bevy `0.19.x`, so a consumer reads the compatible Bevy line
straight off the crate version.

## [Unreleased]

## [0.19.6] - 2026-08-01

A maintenance release: the public API is unchanged from `0.19.5`. Everything
below is comments, documentation, internal structure and repository tooling.

### Added

- `scripts/check-comment-tags.sh`: enforces the own-line comment convention (a non-doc comment opens with `NOTE:` / `FIXME:` / `BUG:` / `TODO:`), plus a second rule for `#[cfg(test)]` modules - a numeric literal written in BOTH a test fn's `///` and its body is reported, since a doc spelling out a number the body uses is guarding a value. Probed by `scripts/test-check-comment-tags.sh` against `scripts/fixtures/comment_tags/`.
- `scripts/check-stale-refs.sh <needle>...`: a fail-loud staleness gate to run after any file move, rename or deletion. `git grep`s the tracked tree for each literal needle (excluding `tasks/`, plus any `-x <pathspec>`) and exits 1 listing every survivor.
- `scripts/sample-peak-rss.sh`: samples peak resident memory of a build or test command, used to size the link-concurrency caps below.

### Changed

- Library-wide comment and structure pass over every module (the `v0.19.x` KISS epic). Explanatory prose moved out of the sources into task records, own-line comments now earn their keep under the tag convention, and rustdoc was tightened - so the rendered docs read differently even though no item changed.
- Two large modules split along their internal seam, both into PRIVATE submodules with an identical public surface: the triangle-vs-plane geometry kernel behind `mesh::builder`'s `slice()` moved to `mesh::slice`, and the `integrity` collision half (impact and blast overlap -> `HealthApplyDamage`, with its three damage constants and the avian dependency) moved out of `integrity::plugin` into `integrity::damage`.
- `nix develop` now derives `CARGO_BUILD_JOBS` and `RUST_TEST_THREADS` from the machine's RAM rather than leaving `cargo test` to link one full Bevy+avian binary per core. Sized against `cargo test --features debug`, the heaviest configuration.

### Removed

- The `docs/` directory. Its content was folded into `examples/README.md` (headless harness), `web/README.md` (wasm/trunk showcase builds) and the per-task `NOTES.md` records, so there is one home per topic instead of two.

## [0.19.5] - 2026-07-20

### Fixed

- A looping autopilot (`loop_while_pending`) now reports done the MOMENT the other collectors finish, mid-cycle, instead of waiting for the current cycle's end - a slow cycle otherwise wasted up to its full length after a capture completed, and could straddle the completion deadline into a false "autopilot" laggard.

## [0.19.4] - 2026-07-20

### Added

- `AutopilotPlugin::loop_while_pending()`: at the timeline's end, while OTHER completion collectors are still pending (a frame capture mid-window), the autopilot writes an `AutopilotLoop` message (the game observes it to reset its scene/script), zeros the cycle clock, and keeps driving - so a capture measures repeated ACTIVITY instead of an idle tail. Reports done normally once nothing else is pending; ignored (with a warning) combined with `self_completing()`.
- `HarnessCompletion::others_pending(name)`: the loop condition - whether any collector other than `name` is still pending.

## [0.19.3] - 2026-07-20

### Changed

- Harness completion protocol (`debug::harness::completion`): armed harness collectors now REGISTER and report DONE instead of writing `AppExit` themselves - the app exits `Success` only when every registered collector (autopilot, screenshot, an external frame capture) has finished, so a wall-clock timeline can no longer end the app under a frame-count capture mid-window. A deadline backstop (`BCS_HARNESS_DEADLINE`, default 120s) error-exits NAMING the still-pending collectors. Success exits negotiate; failures still abort immediately. Single-collector behavior is identical to before.
- `AutopilotPlugin::self_completing()`: for staged scripts that end their own run - the timeline becomes a runway, the script reports done via `HarnessCompletion` when its final stage lands, and a timeline expiry with the script still pending is an error exit (a stalled script can never pass as a finished cycle).

## [0.19.2] - 2026-07-19

### Added

- `modding::events::GameEvent` gained public read accessors `name()` and `info()`, so an external observer on `On<GameEvent>` (a run recorder, a debug overlay) can see which event passed by and read its payload. Construction is unchanged (`GameEvent::new`); the fields stay module-private.

## [0.19.1] - 2026-07-17

### Fixed

- `ui::health_display` no longer misreads a health pool: a barely-alive sliver below 1% now ceils to 1% (an alive, targetable ship never displays "0%"), and a non-positive `max` renders 0% instead of the `NaN%` a divide-by-zero produced during a section-less death window.

## [0.19.0] - 2026-07-16

First tagged release. The version scheme starts at `0.19.0` to match Bevy 0.19.

### Added

- The module library, each a small game-agnostic concern exposed through a single `*Plugin` or plain types: `audio` (one-shot SFX), `camera` (chase, WASD, post-processing, skybox, shake, projection), `feedback` (hit-flash and screen-flash juice), `health` (pool + propagating damage event), `helpers`, `input` (unified pointer + cursor grab), `material` (blooming emissive), `mesh` (procedural `TriangleMeshBuilder` + mesh-explode), `meth` (vector math), `modding` (a serde-friendly event bus), `persist` (cross-platform save/load), `physics` (avian3d PD attitude and Doom-style controllers), `scoring` (streak + high score), `time` (cooldowns), `transform` (motion drivers), `tween`, and `ui` (status HUD, menus, popups, touchpad)
- A `debug` feature module: wireframe/inspector toggles plus a headless verification harness (`AutopilotPlugin`, `ScreenshotPlugin`) used by the example smoke tests
- Runnable example games (`examples/NN_name.rs`) that double as integration tests and quickstart documentation, plus a WebAssembly web showcase (`web/`) that serves them
- The `#[derive(EventKind)]` procedural macro (in the re-exported `bevy_common_systems_macros` subcrate) backing the modding event bus
- Release tooling mirroring nova-protocol: a cargo-about third-party license gate (`about.toml`, `about.hbs`, `scripts/gen-licenses.sh`, and a `licenses` CI job that fails on any non-permissive dependency license), and a tag-triggered `release-flow` workflow that builds the web showcase, bundles the generated third-party license manifest, and attaches the zip to the matching GitHub Release

[unreleased]: https://github.com/alexjercan/bevy-common-systems/compare/v0.19.6...HEAD
[0.19.6]: https://github.com/alexjercan/bevy-common-systems/compare/v0.19.5...v0.19.6
[0.19.5]: https://github.com/alexjercan/bevy-common-systems/compare/v0.19.4...v0.19.5
[0.19.4]: https://github.com/alexjercan/bevy-common-systems/compare/v0.19.3...v0.19.4
[0.19.3]: https://github.com/alexjercan/bevy-common-systems/compare/v0.19.2...v0.19.3
[0.19.2]: https://github.com/alexjercan/bevy-common-systems/compare/v0.19.1...v0.19.2
[0.19.1]: https://github.com/alexjercan/bevy-common-systems/compare/v0.19.0...v0.19.1
[0.19.0]: https://github.com/alexjercan/bevy-common-systems/releases/tag/v0.19.0
