# Add fruit ninja style slicing example

- STATUS: CLOSED
- PRIORITY: 80
- TAGS: feature,example

## Goal

Add a runnable `examples/06_fruitninja.rs` that plays like Fruit Ninja using
only procedural shapes (no external assets) and no new dependencies. Shapes
are launched up in a parabolic arc; the player slices them by dragging the
mouse cursor across them, which explodes the shape into flying fragments and
scores a point. Missed shapes fall away. Score is shown via the status bar.

## Steps

- [x] Create `examples/06_fruitninja.rs` with the standard header: clap `Cli`
      struct, `main()` adding `DefaultPlugins`, `#[cfg(feature = "debug")]`
      `InspectorDebugPlugin`, `FrameTimeDiagnosticsPlugin` (for status FPS),
      `ExplodeMeshPlugin`, `TempEntityPlugin`, `StatusBarPlugin`.
      (Also adds `PhysicsPlugins` so `--features debug` boots -- see Notes.)
- [x] `setup`: spawn a fixed `Camera3d` looking down the -Z axis at a play
      plane (z = 0), a `DirectionalLight`, and a status bar with a Score item
      plus FPS. Store shared mesh/material handles in a resource
      (`FruitAssets`) built from `TriangleMeshBuilder::new_octahedron` in a
      few colors.
- [x] Add a `Score` resource (usize) and a `SpawnTimer` resource; a spawn
      system that periodically launches a `Fruit` entity from below the view
      at the z=0 plane with an upward + sideways initial velocity
      (`FruitMotion { velocity }`) and a random color.
- [x] Add a `move_fruit` system applying gravity to `FruitMotion`, advancing
      translation, tumbling rotation, and despawning fruit that falls below a
      cutoff (missed).
- [x] Add slicing: `track_cursor` records the cursor world position on the
      z=0 plane via `Camera::viewport_to_world` + ray/plane intersection while
      LMB is held; `slice_fruit` tests the swipe segment (prev -> cur) against
      each fruit radius with `segment_hits_circle`, then removes `Fruit`,
      inserts `ExplodeMesh`, and increments `Score`.
- [x] Reuse the explode fragment pattern from `05_explode`: an
      `on_fragments_spawned` observer spawns each `ExplodeFragments` piece as a
      flying `FragmentMotion` entity with a `TempEntity` lifetime, then
      despawns the shell.
- [x] Register the example in AGENTS.md example list and confirm it builds:
      `cargo clippy --all-targets` (+ `--features debug`), `cargo fmt --check`,
      `./scripts/check-ascii.sh` -- all clean.
- [x] Pure hit-test `segment_hits_circle` has 6 `#[cfg(test)]` unit tests
      (run via `cargo test --example 06_fruitninja`). Real-GPU boot verified,
      plus a throwaway auto-slice boot that pushed sliced fragment meshes
      through the GPU upload with no panic.

## Notes

- Relevant files: `examples/05_explode.rs` (explode + fragment pattern),
  `examples/04_status_item.rs` (status bar wiring, FrameTimeDiagnosticsPlugin),
  `src/mesh/explode.rs` (`ExplodeMesh { fragment_count }`, `ExplodeFragments`
  with `fragment.mesh` + `fragment.direction: Dir3`), `src/helpers/temp.rs`
  (`TempEntity(seconds)`).
- `ExplodeMeshPlugin` slices with random planes through the mesh; the mesh must
  be centered so a plane hits it -- octahedron sphere works (as in 05).
- No avian physics needed: fruit + fragment motion is hand-rolled kinematics
  like `move_fragments` in 05_explode. Do NOT add `PhysicsPlugins` unless a
  plugin requires it (ExplodeMeshPlugin does not).
- Slicing detection: sample cursor world pos each frame; check distance from
  each fruit center to the cursor (and ideally the segment prev->cur cursor)
  against the fruit radius. Keep it simple: point-in-radius while LMB held is
  acceptable for a first cut.
- Status score item: use `status_bar_item` with a `value_fn` reading the
  `Score` resource from `&World` (closures get `&World`).
- Assumption: single numbered example `06_fruitninja.rs`; play field in a
  plane facing a static camera, mouse-drag to slice.

## Close-out

Shipped `examples/06_fruitninja.rs` (registered in AGENTS.md). Octahedron
fruit arc up from below under hand-rolled gravity; hold LMB and swipe to
slice them into exploding fragments via `ExplodeMeshPlugin`; fragments retire
via `TempEntity`; score shown in the status bar. No new dependencies, no
physics simulation (`PhysicsPlugins` is present only so the debug inspector
boots under `--features debug` -- the explode retro's near-miss lesson).

Review: 2 rounds. R1 raised one MAJOR -- the swipe segment was split across
two `Update` systems (`track_cursor` + `slice_fruit`) sharing `CursorTrail`,
so their unspecified order let the store race the read and collapse the
swept segment to a point. Fixed by fusing them into one read-test-store
system. Two NITs (fragment color inheritance, `Single` params) also taken.
R2 APPROVE.

Verification: 6 pure-geometry unit tests for `segment_hits_circle` (run via
`cargo test --example`), the library's MinimalPlugins explode test covers the
slice->fragments path, plus two real-GPU boots (plain + throwaway auto-slice)
with no panic. Note: plain `cargo test` (what CI runs) does not execute an
example's test module, so the 6 tests are compiled by CI but not run there.
