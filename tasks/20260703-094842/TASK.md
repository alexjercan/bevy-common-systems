# Write AGENTS.md and CLAUDE.md describing the crate for agents

- STATUS: CLOSED
- PRIORITY: 100
- TAGS: docs

## Goal

Give any agent session a fast, accurate orientation in this repo: what the
crate is for (a collection of Bevy utilities to build games faster), how the
code is organized, the conventions it follows, and the commands to build,
lint, test and run examples. AGENTS.md holds the content; CLAUDE.md imports
it so both agent ecosystems read the same document.

## Steps

- [x] Skim every module not yet read so descriptions are accurate, one line
      each: src/camera/{chase,post,skybox}.rs, src/mesh/{builder,explode}.rs,
      src/meth/{lerp,sphere}.rs, src/physics/pd_controller.rs,
      src/transform/*.rs, src/helpers/{despawn,temp,wasd}.rs,
      src/ui/status.rs, examples/02-04, bevy_common_systems_macros/Cargo.toml.
- [x] Verify the actual commands work before documenting them: `cargo build`,
      `cargo fmt --check`, `cargo clippy`, `cargo test`, and how examples are
      run (`cargo run --example 01_sphere`, `--features debug`).
      Result: fmt/clippy/test --all-targets pass; plain `cargo test` fails on
      10 illustrative doctests (follow-up task 20260703-095339); clippy with
      --features debug has 1 warning (same task). Macro default-path bug
      found while reading: backlog task 20260703-095509.
- [x] Write AGENTS.md at the repo root covering: crate purpose and philosophy
      (copy-pastable utilities, plugin-per-concern), workspace layout (main
      crate + bevy_common_systems_macros proc-macro subcrate), module map
      with one line per module, conventions (per-module `prelude`, `*Plugin`
      structs, `*Systems`/`*PluginSystems` system sets, module-level `//!`
      docs, `debug!`/`trace!` logging, Reflect derives), feature flags
      (`debug`, `dev`), key dependencies (bevy 0.18, avian3d 0.6,
      bevy_enhanced_input 0.25, noise, serde), toolchain (nightly, rustfmt
      import grouping, relaxed clippy lints, nix flake dev shell, wasm
      target), and build/test/example commands.
- [x] Write CLAUDE.md at the repo root as a one-line import of AGENTS.md
      (`@AGENTS.md`), matching the user's own global convention.
- [x] Cross-check AGENTS.md claims against the code once more (module names,
      feature names, dependency versions, commands) so nothing is invented.
      Caught and fixed: wrong "no unit tests" claim (13 exist and pass),
      PD controller writes torque to Output instead of applying it,
      03_modding opens a window, status bar closures run in an exclusive
      system, F11 toggles for debug plugins, modding handler wording.
- [x] Write docs/ note per global guidelines: decision record for why
      CLAUDE.md imports AGENTS.md instead of duplicating content
      (tasks/20260703-094842/NOTES.md).

## Notes

- Relevant files: Cargo.toml, src/lib.rs, src/*/mod.rs, rust-toolchain.toml,
  rustfmt.toml, flake.nix, .cargo/config.toml, examples/*.rs,
  bevy_common_systems_macros/src/lib.rs, README.md.
- Facts already gathered:
  - Crate: bevy_common_systems 0.0.1, edition 2021, "Common systems and
    utilities for Bevy-based projects". lib.rs doc: "Fully copy-pastable
    crate for common gameplay components and systems."
  - Modules: camera (chase, post, skybox, wasd), debug (wireframe, inspector;
    cfg(feature = "debug")), health (Health, HealthApplyDamage entity event,
    HealthZeroMarker), helpers (despawn, temp, wasd controller), mesh
    (TriangleMeshBuilder, explode), meth (LerpSnap, sphere math), modding
    (generic GameEvent system: EventWorld/EventKind/EventHandler with
    filters+actions, GameEventsPlugin, Commands::fire ext), physics
    (pd_controller), transform (sphere_orbit, directional_sphere_orbit,
    random_sphere_orbit, point_rotation, smooth_look_rotation), ui (status).
  - Macros subcrate: derive(EventKind) with #[event_name]/#[event_info]
    attributes; re-exported from lib.rs and via prelude.
  - Crate-level prelude aggregates all module preludes; every module exposes
    its own prelude.
  - Features: debug = [avian3d/diagnostic_ui, bevy/track_location,
    bevy-inspector-egui]; dev = [debug]. Clippy allows type_complexity and
    too_many_arguments (Cargo.toml [lints.clippy]).
  - Toolchain: nightly (rust-toolchain.toml); rustfmt.toml: reorder_imports,
    imports_granularity = "Crate", group_imports = "StdExternalCrate"; nix
    flake dev shell with wasm32-unknown-unknown target, trunk, wasm-pack;
    .cargo/config.toml sets web_sys_unstable_apis for wasm.
  - Examples use clap CLI structs: 01_sphere, 02_planet, 03_modding,
    04_status_item; dev-dependency clap 4.5.
  - README.md is minimal (3 lines) and slightly stale ("collection of bevy
    examples"); AGENTS.md becomes the real orientation doc. Leave README
    alone unless trivial to align; out of scope here.
- Assumption: CLAUDE.md should be `@AGENTS.md` (import), not duplicated
  content. Matches the user's own ~/.claude/CLAUDE.md which imports
  ~/AGENTS.md.
- Assumption: tests - crate has no #[test] functions today; `cargo test`
  still compiles examples and doctests. Document that examples are the de
  facto integration tests. (Verify doctest state during work; several module
  docs have ```rust blocks that may not compile as doctests - e.g. helpers
  mod doc has a typo `heleprs` and health doc uses bare `commands`. Do NOT
  fix code in this task; just document how to run checks accurately.)
  CORRECTED during work: the crate HAS 13 unit tests (meth/sphere,
  mesh/builder, physics/pd_controller, transform/point_rotation), all
  passing. The doctest prediction held: 10 fail, 2 pass.

## Close-out

What changed and why:
- Added AGENTS.md (repo root): crate purpose, layout, module map with one
  accurate line per module, conventions (plugin per concern,
  Config/Input/Output/State split, preludes, observers, Reflect, logging,
  rustfmt/clippy, ASCII style), features, dependencies with roles,
  environment/toolchain, verified command suite with known issues,
  examples, workflow, gotchas.
- Added CLAUDE.md containing only `@AGENTS.md`. Alternative (symlink,
  duplicated content) rejected; see
  tasks/20260703-094842/NOTES.md for the decision record.
- Filed follow-up tasks instead of widening the diff: 20260703-095339
  (10 failing doctests + 1 clippy warning under --features debug),
  20260703-095509 (stale default event_info path in EventKind derive).

Difficulties:
- `cargo test | tail` truncated the lib test results, which briefly led to
  a false "no unit tests" claim in the draft; caught in the cross-check
  step by re-running without truncation.
- Doctest failures were predicted at planning time and confirmed; handled
  by documenting the working suite (`cargo test --all-targets`) and
  pointing at the fix-up task.

Self-reflection:
- Do not derive negative claims ("there is no X") from summarized command
  output; re-run and read the full section.
- 50-60 line module reads were fine for the module map but not for
  behavioral sentences; those needed whole-file reads (PD controller
  Output, status bar exclusive system).
- The planned steps survived contact with reality unchanged, which
  suggests the up-front code reading during planning was the right depth.
