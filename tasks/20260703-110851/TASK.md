# Add runnable mesh slicer / explode example

- STATUS: CLOSED
- PRIORITY: 90
- TAGS: feature

## Goal

Ship a runnable example that exercises the mesh slicer end to end: spawn a
mesh, trigger `ExplodeMesh`, and spawn the resulting `ExplodeFragments` as
separate entities that fly apart along their explosion directions. This is
both the quickstart doc for the slicer and its de facto integration test -
it must run without crashing, including under repeated explosions.

## Steps

- [x] Add `examples/05_explode.rs` following the house pattern (clap CLI
      header, `DefaultPlugins`, `PhysicsPlugins::default()`, optional
      `InspectorDebugPlugin` under the `debug` feature, a WASD camera via
      `WASDCameraControllerPlugin`, a setup system, `ExplodeMeshPlugin`).
- [x] Spawn a target mesh (e.g. `TriangleMeshBuilder::new_octahedron(3)` or
      a cone) with a `StandardMaterial`, matching example 01/02.
- [x] On a key press (e.g. Space), insert `ExplodeMesh { fragment_count }`
      on the target. Add an observer or system on `Added<ExplodeFragments>`
      (or `On<Insert, ExplodeFragments>`) that spawns one child entity per
      fragment with its mesh + material, then despawns the original, and
      pushes each fragment along `fragment.direction` (simple per-frame
      velocity component, or an avian `RigidBody` + `LinearVelocity` since
      avian is already a dep).
- [x] Make it re-runnable: pressing the key again spawns a fresh target and
      explodes it, so the example can be hammered to shake out edge cases.
- [x] Manually run it and confirm no crash: `cargo run --example 05_explode`
      (and `--features debug`). Explode several times; watch the log for the
      slicer's `error!` lines (declined slices are fine; panics or NaN
      warnings are not). Note: on NixOS this must run inside `nix develop`.
- [x] Register the example in AGENTS.md's example list and the module-map
      mention of `mesh/explode`, and (if the repo lists examples anywhere
      else) keep them consistent.

## Notes

- Depends on: 20260703-110915 (slicer hardening) - build on the hardened
  code so the example does not crash on degenerate slices.
- `ExplodeFragments` is added to the entity that received `ExplodeMesh`
  (see explode.rs handle_explosion); each `ExplodeFragment` has `origin`,
  `mesh: Handle<Mesh>`, and `direction: Dir3`. Reuse the handle; add a
  material for rendering.
- Keep it game-agnostic and small, per the crate philosophy. A single
  exploding shape that respawns is enough; no need for scenes or UI.
- This example is the integration test the slicer never had; if it surfaces
  a new crash not covered by 20260703-110915, fix it there (or file a
  follow-up) rather than papering over it in the example.
- Numbering: 05 is the next free example index (01-04 exist).

## Close-out

What shipped:
- examples/05_explode.rs: spawns an origin-centered octahedron target;
  Space inserts `ExplodeMesh`; an `On<Insert, ExplodeFragments>` observer
  spawns each fragment as a flying entity (velocity along
  `fragment.direction`, gravity + tumble), despawns the shell, and spawns a
  fresh target so it is re-runnable. Fragments auto-despawn via the crate's
  `TempEntity`, so repeated explosions do not pile up.
- A headless integration test in explode.rs
  (`test_explode_mesh_plugin_produces_fragments`) that drives the same
  `ExplodeMesh -> ExplodeFragments` observer path in a `MinimalPlugins`
  app and asserts finite fragment meshes. This is the CI-able proof the
  graphical example cannot be.
- AGENTS.md: example list + `mesh/explode` module-map mention updated.

Design choices:
- Manual velocity + `TempEntity` cleanup instead of avian rigid bodies:
  arbitrary sliced meshes have no ready collider, and the demo only needs
  fragments to fly apart and disappear. Kept it small, per crate philosophy.
- Octahedron (centered at origin) as the target so every plane through the
  origin cuts it - the example always explodes rather than sometimes
  declining.

Verification (this was a headless background session):
- Compiles (default and `--features debug`); full suite green (unit tests
  now 20, plus the headless plugin integration test).
- Booted the real app: NVIDIA RTX 3060 Ti / Vulkan, window created, ran to
  the timeout with no panic (only benign XSETTINGS / swap-chain warnings).
- Temporarily added a 2s auto-explode, ran ~14s: fragments sliced, spawned,
  uploaded to the GPU and rendered with no panic or slice error - exercising
  exactly the GPU-upload path where NaN geometry would have crashed. Reverted
  the temporary change (not committed).

Difficulties / limits:
- Could not drive a live Space keypress from the headless session, so the
  runtime explosion path was proven two other ways: the headless plugin
  integration test and the temporary auto-explode graphical run. Both
  passed; the committed example keeps the Space trigger.

Self-reflection:
- The most useful move was refusing to treat "graphical example, can't run
  it here" as unverifiable: a MinimalPlugins integration test covers the
  ECS wiring for CI, and a throwaway auto-explode run covered the GPU path
  once. Same pattern as the CI task - verify every separable piece and name
  the one irreducible gap (live input), which here was closed by the
  auto-explode run.
