# Harden mesh slicer against crash-inducing edge cases

- STATUS: OPEN
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

- [ ] `edge_plane_intersection` (src/mesh/builder.rs:418): guard the
      division. When `ab.dot(plane_normal)` is ~0 (edge parallel to the
      plane) the current `t` is inf/NaN and poisons the vertex. Return a
      finite fallback (e.g. clamp `t` to [0,1] / fall back to the edge
      midpoint) so the result is always finite. Unit test: an edge parallel
      to the plane does not yield NaN.
- [ ] `uvs()` (src/mesh/builder.rs:299-300): `(b - a).normalize()` and the
      cross product produce NaN for degenerate (zero-length-edge or
      zero-area) triangles, which slicing through the origin readily
      creates. Use `normalize_or_zero` and skip/zero UVs for degenerate
      triangles so no NaN reaches the vertex buffer. Unit test: a degenerate
      triangle yields all-finite UVs.
- [ ] `From<Mesh>` (src/mesh/builder.rs:389-415): it `.unwrap()`s the
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
- [ ] `fill_boundary` (src/mesh/builder.rs:234-252): `reordered[i + 1]`
      panics on an odd-length boundary. It is even today (each split pushes
      2), but harden it - iterate with `chunks_exact(2)` or guard `i + 1 <
      len` - so a future change cannot turn it into an index panic. Unit
      test: an odd-length boundary does not panic.
- [ ] explode.rs `Dir3::new_unchecked(normal.normalize())`
      (src/mesh/explode.rs:147): replace with a checked construction that
      falls back to a default axis if the normal is not finite/nonzero, so a
      bad normal cannot create an invalid `Dir3`.
- [ ] Optional robustness in `explode_mesh` (src/mesh/explode.rs:162): drop
      fragments whose built mesh is empty/degenerate before returning, so
      the caller never receives a zero-triangle fragment. Only if it falls
      out naturally; do not over-engineer.
- [ ] Run the full suite: `cargo fmt --check`, `cargo clippy --all-targets`,
      `cargo clippy --all-targets --features debug`, `cargo test`,
      `cargo test --features debug`, and `./scripts/check-ascii.sh`.

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
