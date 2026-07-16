# Retro: Add fruit ninja style slicing example

- TASK: 20260703-114259
- BRANCH: feature/fruitninja-example (merged to master, deleted)
- REVIEW ROUNDS: 2 (R1 REQUEST_CHANGES, 1 MAJOR + 2 NITs; R2 APPROVE)

See TASK.md close-out for what shipped; this is process only.

## What went well

- Reading the two most relevant retros first (explode-example,
  slicer-hardening) paid off immediately and concretely: the explode retro's
  near-miss about `PhysicsPlugins` being needed for `--features debug` meant I
  added it from the start instead of discovering it via a broken debug build.
  I ran `cargo clippy --all-targets --features debug` as part of the suite for
  the same reason. The compounding worked exactly as intended.
- The graphical-example verification recipe from the explode retro
  transferred directly: a headless-provable core (6 pure `segment_hits_circle`
  unit tests), a real-GPU boot, and a throwaway auto-slice boot that pushed
  real sliced fragment meshes through the GPU upload. The auto-slice patch
  covered the one path (a live slice) that unit tests and a plain boot cannot.
- Reusing crate utilities as the example's whole point (`ExplodeMeshPlugin`,
  `TempEntity`, `StatusBarPlugin`, `TriangleMeshBuilder`) kept the new code to
  game glue, not engine plumbing - the dogfooding an example is supposed to
  demonstrate.

## What went wrong

- R1.1 (MAJOR): I split the swipe into `track_cursor` (writes
  `trail.previous`) and `slice_fruit` (reads it), both in one unordered
  `Update` tuple sharing `CursorTrail`. Because they conflict on that
  resource, Bevy runs them in some unspecified order; whenever the writer runs
  first, `previous == current` and the swept segment collapses to a point, so
  fast swipes between frames miss. Root cause: I reached for "one system per
  job" (a good default) without noticing that a cross-frame read-modify-write
  of shared state is a single ordered operation, not two. The split looked
  clean precisely because the ordering hazard is invisible at each call site -
  the same shape of blind spot the explode retro flagged for cross-plugin
  coupling, here for cross-system scheduling.

## What to improve next time

- When one frame's logic reads last frame's value of a resource and writes
  this frame's, keep the read, the test and the store in a single system (or
  `.chain()` explicitly). Splitting a read-modify-write of shared state across
  two unordered `Update` systems is an ordering bug even when each half looks
  correct. Litmus test: if two systems in the same schedule share a resource
  and the result depends on which runs first, they need explicit ordering or
  to be fused.

## Action items

- [x] Example shipped, reviewed to APPROVE, merged to master.
- [ ] No AGENTS.md change yet: this is the first system-ordering finding, so
  it stays a retro lesson. If a second cross-system-ordering bug shows up,
  promote "fuse or `.chain()` read-modify-write of shared state" into the
  Conventions section as a rule.
- [ ] Not filed as a task: `cargo test` (what CI runs) does not execute an
  example's `#[cfg(test)]` module, so the 6 `segment_hits_circle` tests are
  compiled but not run in CI. If example-local pure logic like this recurs and
  wants real CI coverage, either lift the helper into the library with tests
  next to it, or add a `tests/` integration target. Not worth doing for one
  helper today.
