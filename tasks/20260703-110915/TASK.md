# Harden mesh slicer against crash-inducing edge cases

- STATUS: CLOSED
- PRIORITY: 100
- TAGS: bug

## Goal

The mesh slicer (`TriangleMeshBuilder::slice` / `explode_mesh`) can produce
NaN/inf geometry and can panic outright on some inputs. Feeding those meshes
to Bevy crashes the game (NaN AABB, wgpu validation) or panics directly.
Make slicing total: for any input mesh and any plane it must return a valid
mesh or cleanly decline (None), never NaN geometry and never a panic. Cover
each edge case with a pure unit test.

## Steps

- [x] `edge_plane_intersection` (src/mesh/builder.rs:418): guard the
      division. When `ab.dot(plane_normal)` is ~0 (edge parallel to the
      plane) the current `t` is inf/NaN and poisons the vertex. Return a
      finite fallback (e.g. clamp `t` to [0,1] / fall back to the edge
      midpoint) so the result is always finite. Unit test: an edge parallel
      to the plane does not yield NaN.
- [x] `uvs()` (src/mesh/builder.rs:299-300): `(b - a).normalize()` and the
      cross product produce NaN for degenerate (zero-length-edge or
      zero-area) triangles, which slicing through the origin readily
      creates. Use `normalize_or_zero` and skip/zero UVs for degenerate
      triangles so no NaN reaches the vertex buffer. Unit test: a degenerate
      triangle yields all-finite UVs.
- [x] `From<Mesh>` (src/mesh/builder.rs:389-415): it `.unwrap()`s the
      position attribute and indices and `panic!`s on non-Float32x3
      positions - any user mesh without indices, without positions, or with
      a different position format panics the game on explode. Add a
      fallible `TriangleMeshBuilder::try_from_mesh(&Mesh) -> Option<Self>`
      (or `TryFrom`) that returns None instead of panicking, and use it on
      the untrusted input path in explode.rs. Keep `From` only where the
      mesh is known-good (built by `build()`), or reimplement `From` on top
      of the fallible path with an explicit expect documented as
      internal-only. Unit test: a mesh with no indices returns None, not a
      panic.
- [x] `fill_boundary` (src/mesh/builder.rs:234-252): `reordered[i + 1]`
      panics on an odd-length boundary. It is even today (each split pushes
      2), but harden it - iterate with `chunks_exact(2)` or guard `i + 1 <
      len` - so a future change cannot turn it into an index panic. Unit
      test: an odd-length boundary does not panic.
- [x] explode.rs `Dir3::new_unchecked(normal.normalize())`
      (src/mesh/explode.rs:147): replace with a checked construction that
      falls back to a default axis if the normal is not finite/nonzero, so a
      bad normal cannot create an invalid `Dir3`.
- [x] Optional robustness in `explode_mesh` (src/mesh/explode.rs:162): drop
      fragments whose built mesh is empty before returning (the final
      `.filter(|(b, _)| !b.is_empty())`), so the caller never receives a
      zero-triangle fragment. Also carry unsliceable fragments forward
      instead of dropping them, so a missed plane never loses geometry.
- [x] Added an end-to-end finiteness test (not in the original plan):
      `test_explode_mesh_produces_finite_geometry` runs the real
      `explode_mesh` pipeline on an octahedron 50 times and asserts every
      position/normal/UV and every direction is finite. This is the actual
      "won't crash" guarantee and it runs headlessly, unlike the graphical
      example in the sibling task.
- [x] Run the full suite: `cargo fmt --check`, `cargo clippy --all-targets`,
      `cargo clippy --all-targets --features debug`, `cargo test`,
      `cargo test --features debug`, and `./scripts/check-ascii.sh`. All
      pass; unit tests 13 -> 19.

## Notes

- No example exercises this code today (grep confirms), so these paths are
  entirely untested end to end - the sibling task 20260703-110851 adds that
  example and depends on this one landing first.
- Design constraint from AGENTS.md: pure geometry gets `#[cfg(test)]` unit
  tests next to the code; put the edge-case tests in
  src/mesh/builder.rs's existing `mod test`.
- `normals()` already guards degeneracy via `t.normal().unwrap_or(Dir3::Y)`;
  mirror that style.
- Do not change the slicer's external API shape beyond adding the fallible
  conversion; `slice`/`build` signatures stay as they are.
- The root cause theme is "slicing produces degenerate/parallel geometry
  the math did not defend against". Prefer making each primitive total
  (finite output or None) over adding special-cases in the caller.

## Close-out

What changed and why:
- edge_plane_intersection: guard the zero denominator (edge parallel to
  plane) with a midpoint fallback, and clamp t to [0,1]. Was the primary
  NaN source - a parallel edge divided by zero and poisoned the vertex.
- uvs(): normalize_or_zero for both axes so degenerate (zero-area) slivers,
  which slicing through the origin readily makes, yield finite (0,0) UVs
  instead of NaN.
- Added TriangleMeshBuilder::try_from_mesh(&Mesh) -> Option<Self>;
  reimplemented From<Mesh> on top of it with a documented expect (internal,
  known-good meshes only). explode_mesh now converts the untrusted input
  once via try_from_mesh, so a user mesh with no indices / wrong position
  format returns None instead of panicking.
- fill_boundary: chunks_exact(2) instead of index+1, so an odd boundary
  cannot panic.
- explode.rs: Dir3::new(...).unwrap_or(Dir3::Y) instead of new_unchecked;
  refactored explode_mesh to keep builders in the queue (build() only at
  the end, no Mesh<->builder round-trips), carry unsliceable fragments
  forward, and filter empty fragments out of the result.

Alternatives considered:
- Making From<Mesh> itself fallible (return Result): rejected - From cannot
  fail by contract, and build() output is always well-formed, so the
  convenience impl is worth keeping. try_from_mesh covers untrusted input.
- Using the mesh centroid as the slice plane_point (instead of ZERO) so
  off-origin meshes always split: a real robustness/quality improvement,
  but it is beyond crash-prevention (a missed plane is a no-op, not a
  crash) and would change explosion behavior. Left for a possible
  follow-up; carrying unsliceable fragments already prevents geometry loss.

Difficulties:
- First cut of the finiteness test asserted `fragments.len() >= 8` for
  fragment_count 8. That is wrong: re-slicing a hemisphere with a random
  plane through the origin can leave one side empty (slice -> None), so the
  fragment is carried, not doubled. Growth is <= doubling and stochastic.
  Relaxed the assertion to `>= 2` (the first split of an origin-centered
  octahedron always succeeds); finiteness, not count, is the property under
  test. Caught by thinking through the geometry rather than by a flake,
  but only because I wrote the test and reasoned about its bound.

Self-reflection:
- Good: the headless end-to-end finiteness test is the honest proof of
  "won't crash" - it runs the real pipeline and would go red on any
  reintroduced NaN, and unlike the graphical example it runs in CI.
- Enumerating exit/degeneracy conditions up front (the plan listed all five
  sites) made this mechanical; the one surprise was the test's own bound,
  not the code.
