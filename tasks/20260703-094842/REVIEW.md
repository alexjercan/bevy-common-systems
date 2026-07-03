# Review: Write AGENTS.md and CLAUDE.md describing the crate for agents

- TASK: 20260703-094842
- BRANCH: docs/agents-claude-md

## Round 1

- VERDICT: APPROVE

Verified independently: `git diff master...docs/agents-claude-md` touches
only docs and task files; `cargo build`, `cargo fmt --check`,
`cargo clippy --all-targets` (clean), `cargo test --all-targets` (13/13)
all pass as the document claims; the known-issue notes (10 failing
doctests, 1 clippy warning under `--features debug`) reproduce exactly;
the module map, feature flags, dependency versions, toolchain facts,
example descriptions and all four gotchas were spot-checked against the
code and are accurate. CLAUDE.md uses the supported `@AGENTS.md` import
syntax. TASK.md checkboxes match what was actually done.

The findings below are wording overclaims in the Conventions section:
statements of the form "every X" where the codebase has real exceptions.
None is blocking, but this document is ground truth for future agents, so
absolutes should only be used where they are literally true.

- [ ] R1.1 (MINOR) AGENTS.md:118 - "Every module has a `pub mod prelude`"
  is false: src/debug/wireframe.rs, src/debug/inspector.rs,
  src/meth/lerp.rs and src/meth/sphere.rs have none; their parents
  re-export items directly (src/debug/mod.rs:33, src/meth/mod.rs:14).
  Suggested change: "Modules expose their public API through preludes:
  most files define `pub mod prelude`, and parent modules aggregate child
  preludes (a few leaf files are re-exported directly by their parent)."
  - Response:
- [ ] R1.2 (MINOR) AGENTS.md:107 - "one plugin per concern" is false for
  pure-utility modules: src/meth/ and src/mesh/builder.rs ship no plugin
  at all. An agent following this literally would wrap pure math in an
  empty plugin. Suggested change: "Modules that add runtime behavior ship
  one plugin per concern, named `*Plugin`; pure utility modules (meth,
  mesh/builder) export plain types and functions instead."
  - Response:
- [ ] R1.3 (MINOR) AGENTS.md:126-127 - "Module-level `//!` doc comment ...
  at the top of every file; doc comments on every public item" - both
  absolutes have exceptions: src/physics/mod.rs, src/transform/mod.rs,
  src/meth/lerp.rs and src/meth/sphere.rs have no `//!` doc, and
  src/mesh/explode.rs uses `///` attached to a `use` item instead (its
  module doc does not even render); several public items in
  src/ui/status.rs (e.g. `StatusBarItemMarker`, `StatusBarRootConfig`)
  are undocumented. Suggested change: state it as the target convention,
  e.g. "Most files carry a module-level `//!` doc with a usage snippet
  and public items carry doc comments; match that standard in new code
  even though a few existing files fall short."
  - Response:
- [ ] R1.4 (MINOR) AGENTS.md:72 - "`LerpSnap` ... used by every smoothing
  system in the crate" is false: `lerp_and_snap` is used only by
  camera/chase, transform/sphere_orbit and
  transform/directional_sphere_orbit; camera/wasd and
  transform/smooth_look_rotation smooth by other means. Suggested change:
  "used by the chase camera and the sphere orbit systems for smoothing".
  - Response:
- [ ] R1.5 (NIT) AGENTS.md:109 - "(a few older ones use `*PluginSystems`)"
  claims an age difference that git history cannot support (everything
  dates to the initial commit). Suggested change: drop "older": "(a few
  use `*PluginSystems`)".
  - Response:

Verdict is APPROVE per the severity rules (no BLOCKER or MAJOR). The
MINOR findings are one-line wording fixes in a document whose value is
accuracy; recommended to apply before merge.
