# Review: Harden mesh slicer against crash-inducing edge cases

- TASK: 20260703-110915
- BRANCH: fix/slicer-hardening

## Round 1

- VERDICT: APPROVE

Verified independently:

- All five identified crash sites are fixed and the reasoning holds up:
  - `edge_plane_intersection` guards the zero denominator (midpoint
    fallback) and clamps `t` - the primary NaN source is gone. The existing
    `test_edge_plane_intersection` still passes, so the happy path is
    unchanged.
  - `uvs()` uses `normalize_or_zero`; degenerate triangles now give finite
    (0,0) UVs (test_uvs_degenerate_triangle_are_finite).
  - `try_from_mesh` returns None instead of `.unwrap()`/`panic!`; explode
    converts the untrusted input through it. I audited every `From<Mesh>` /
    `::new` / `.into()` caller: the only untrusted path (explode) now uses
    `try_from_mesh`, and `From`'s remaining `expect` is reached only by the
    documented known-good convenience API (`::new` is unused in-tree).
  - `fill_boundary` uses `chunks_exact(2)` (test_fill_boundary_odd_length).
  - `Dir3::new(...).unwrap_or(Dir3::Y)` replaces `new_unchecked`.
- The headless end-to-end test (`test_explode_mesh_produces_finite_geometry`,
  50 runs) is the right proof of the goal and would catch any reintroduced
  NaN in CI. I ran a harsher throwaway stress myself - a cone (with its
  coplanar base cap and centroid off the origin, exercising the
  carry-forward path) at fragment_count 64, 30 runs - and saw no NaN and no
  panic. Removed it after; it was verification, not a committed change.
- Suite clean on the committed state: fmt, clippy (both feature configs),
  `cargo test` 19 (was 13) and `--features debug`, and the ascii guard all
  pass.
- Behavior changes are sound: carrying unsliceable fragments (instead of
  dropping + logging a misleading `error!`) avoids losing geometry; the
  final empty-fragment filter means callers never get a zero-triangle mesh;
  keeping builders in the queue removes a Mesh<->builder round-trip.

Two non-blocking observations, no change required:

- (NIT) The `edge_plane_intersection` midpoint fallback is a heuristic: for
  a genuinely-crossing but numerically ill-conditioned edge it can place the
  boundary vertex slightly off the true crossing. It is always finite and
  visually fine for an explosion effect, which is the only caller. Fine as
  is.
- (NIT) The finiteness test is stochastic (random planes, 50 runs). It is
  backed by the deterministic targeted unit tests for each guard, so
  coverage of the specific fixes does not depend on the RNG. Good balance.

No BLOCKER/MAJOR/MINOR findings. APPROVE.
