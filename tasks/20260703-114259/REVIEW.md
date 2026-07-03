# Review: Add fruit ninja style slicing example

- TASK: 20260703-114259
- BRANCH: feature/fruitninja-example

## Round 1

- VERDICT: REQUEST_CHANGES

Diff reviewed against master. The example is well-structured, reuses the
crate utilities as intended (`ExplodeMeshPlugin`, `TempEntityPlugin`,
`StatusBarPlugin`, `TriangleMeshBuilder`), adds no dependencies, keeps
`--features debug` booting via `PhysicsPlugins`, and the pure hit-test is
unit-tested. One real correctness flaw undermines the core swipe mechanic.

- [ ] R1.1 (MAJOR) examples/06_fruitninja.rs:270-330 - the swipe segment is
  broken by system ordering. `track_cursor` writes `trail.previous = current
  cursor` and `slice_fruit` reads `trail.previous`; both live in the same
  unordered `Update` tuple and conflict on `CursorTrail`, so Bevy orders them
  arbitrarily (deterministically-per-build but unspecified, not registration
  order). Whenever `track_cursor` runs first, `trail.previous` already equals
  this frame's cursor, so `previous == current` and `segment_hits_circle`
  degenerates to a point-in-circle test every frame - the cross-frame swept
  segment (the whole reason `previous` is stored) never forms, and a fast
  swipe that passes over a fruit between frames is missed. The behavior is
  also nondeterministic across builds. Fix: fold the trail bookkeeping into
  `slice_fruit` and delete `track_cursor` - read `current`, compute
  `previous = trail.previous.unwrap_or(current)`, run the hit test, then set
  `trail.previous = Some(current)` (and clear it to `None` on the frames LMB
  is not held). That makes the read-test-store order explicit and removes the
  duplicate `cursor_on_play_plane` call. If you prefer to keep two systems,
  `.chain()` them (slice before track) - but merging is simpler and cheaper.
  - Response:

- [ ] R1.2 (NIT) examples/06_fruitninja.rs:349 - fragments always use
  `assets.materials[0]`, so a green or purple fruit always bursts into
  fragments of the first palette color. Cheap polish: store the chosen
  `Handle<StandardMaterial>` on the `Fruit` component (or a small
  `FruitColor` component) and reuse it when spawning fragments so the burst
  matches the sliced fruit.
  - Response:

- [ ] R1.3 (NIT) examples/06_fruitninja.rs:375-377,393 - `windows.iter()
  .next()` / `cameras.iter().next()` can be the more idiomatic
  `Single<&Window>` / `Single<(&Camera, &GlobalTransform)>` system params,
  which also documents the single-window/single-camera assumption. Optional.
  - Response:
