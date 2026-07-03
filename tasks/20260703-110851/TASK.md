# Add runnable mesh slicer / explode example

- STATUS: OPEN
- PRIORITY: 90
- TAGS: feature

## Goal

Ship a runnable example that exercises the mesh slicer end to end: spawn a
mesh, trigger `ExplodeMesh`, and spawn the resulting `ExplodeFragments` as
separate entities that fly apart along their explosion directions. This is
both the quickstart doc for the slicer and its de facto integration test -
it must run without crashing, including under repeated explosions.

## Steps

- [ ] Add `examples/05_explode.rs` following the house pattern (clap CLI
      header, `DefaultPlugins`, `PhysicsPlugins::default()`, optional
      `InspectorDebugPlugin` under the `debug` feature, a WASD camera via
      `WASDCameraControllerPlugin`, a setup system, `ExplodeMeshPlugin`).
- [ ] Spawn a target mesh (e.g. `TriangleMeshBuilder::new_octahedron(3)` or
      a cone) with a `StandardMaterial`, matching example 01/02.
- [ ] On a key press (e.g. Space), insert `ExplodeMesh { fragment_count }`
      on the target. Add an observer or system on `Added<ExplodeFragments>`
      (or `On<Insert, ExplodeFragments>`) that spawns one child entity per
      fragment with its mesh + material, then despawns the original, and
      pushes each fragment along `fragment.direction` (simple per-frame
      velocity component, or an avian `RigidBody` + `LinearVelocity` since
      avian is already a dep).
- [ ] Make it re-runnable: pressing the key again spawns a fresh target and
      explodes it, so the example can be hammered to shake out edge cases.
- [ ] Manually run it and confirm no crash: `cargo run --example 05_explode`
      (and `--features debug`). Explode several times; watch the log for the
      slicer's `error!` lines (declined slices are fine; panics or NaN
      warnings are not). Note: on NixOS this must run inside `nix develop`.
- [ ] Register the example in AGENTS.md's example list and the module-map
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
