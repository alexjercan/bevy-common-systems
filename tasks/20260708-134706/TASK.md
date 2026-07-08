# Harden the mesh slicer against degenerate meshes

- STATUS: OPEN
- PRIORITY: 68
- TAGS: bug,mesh

The mesh slicer (`ExplodeMeshPlugin` / `ExplodeMesh` in `mesh/`) can panic on bad input.
Guard its internal edge cases so it returns gracefully instead of crashing:

- empty or degenerate meshes (no vertices / no triangles);
- slice planes that miss the mesh entirely;
- zero-area triangles;
- non-triangle-list topologies (anything other than `TriangleList`).

The slicing algorithm should degrade gracefully (produce no fragments, or the intact mesh)
rather than panic on any of the above.

## Context

Moved here from the nova-protocol repo (nova task 20260706-160503), where it was blocked:
the slicer lives in this crate, so the hardening has to happen here, not in the game repo.

On the nova side the consumer already defends against the two edge cases it can see from
outside: it only triggers the slicer on entities that have a `Mesh3d`, and it falls back to
`Collider::sphere` when `convex_hull_from_mesh` fails. What is left is the slicing algorithm
itself (`TriangleMeshBuilder` slice / `ExplodeMesh` fragment generation) not panicking on the
degenerate inputs listed above. `examples/05_explode` exercises the happy path; add coverage
for the degenerate cases (unit tests on the pure slicing/geometry functions per the crate's
testing convention).
