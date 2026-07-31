# NOTES: KISS pass over mesh/ + meth/ + camera/

Design/fix record for task 20260731-172224. Sibling of 20260731-172208
(which set the convention and shipped `scripts/check-comment-tags.sh`) and
20260731-172223 (integrity/ + physics/).

## Baseline

`./scripts/check-comment-tags.sh src/mesh src/meth src/camera` reported **57**
untagged non-doc comment BLOCKS (not lines - TASK.md's "33 comments" for
`shake.rs` counts raw comment lines, which is a different unit). Per file:
`builder.rs` 9, `explode.rs` 12, `lerp.rs` 6, `sphere.rs` 6, `shake.rs` 15,
`chase.rs` 5, `skybox.rs` 3, `wasd.rs` 1. Bare tatr HUIDs in scope: **0** (the grep
proof was already green at base; it is kept as a regression guard).

Code before `#[cfg(test)]`, measured at base:

| File | total | code | concerns | call |
| --- | --- | --- | --- | --- |
| `mesh/builder.rs` | 618 | 521 | builder API + plane-slice geometry kernel | **SPLIT** |
| `mesh/explode.rs` | 349 | 231 | one (observer + slicing loop) | keep |
| `mesh/mod.rs` | 16 | 16 | one | keep |
| `meth/lerp.rs` | 49 | 49 | one | keep |
| `meth/sphere.rs` | 104 | 57 | one | keep |
| `meth/mod.rs` | 81 | 81 | one (doc + recipe) | keep |
| `camera/shake.rs` | 584 | 313 | one (trauma shake) | keep |
| `camera/chase.rs` | 244 | 244 | one | keep |
| `camera/wasd.rs` | 219 | 219 | one | keep |
| `camera/skybox.rs` | 137 | 137 | one | keep |
| `camera/project.rs` | 117 | 117 | one | keep |
| `camera/post.rs` | 73 | 73 | one | keep |
| `camera/mod.rs` | 34 | 34 | one | keep |

## Split decision: `mesh/builder.rs` -> `+ mesh/slice.rs`

**Split, on evidence.** The file held two things with disjoint dependency
sets:

- `TriangleMeshBuilder` - a triangle soup with primitives, subdivision, noise
  displacement, boundary fill, normals/UVs and `Mesh` conversion. Needs
  `bevy::mesh`, `RenderAssetUsages`, `noise`, `crate::meth`.
- the plane-slice kernel - `edge_plane_intersection`, `TriangleSliceResult`,
  `triangle_slice`. Pure `Triangle3d`-vs-plane math. Touches no builder, no
  `Mesh`, no assets, and imports only `bevy::prelude`.

`slice()` is the only caller of the kernel, and the kernel is the only part of
the file that is total-by-construction geometry (every entry point returns a
finite result for degenerate/parallel input, because slicing runs on arbitrary
game meshes via `explode`). That contract now has a module doc stating it,
which it could not have while it was three loose private fns in the middle of
a builder.

`builder.rs` 618 -> 495 lines (521 -> 447 code); `slice.rs` is 138 lines
(87 code). Public API byte-identical: `slice` is a private MODULE (the
`TriangleMeshBuilder::slice` METHOD is untouched); of the moved items, the two
`builder.rs` still calls are `pub(super)` and `edge_plane_intersection` stays
plain private since its only caller moved with it. And
`bevy_common_systems::mesh::prelude::TriangleMeshBuilder` is untouched.

**Alternatives rejected.**

- Keep whole: it is the crate's largest code body and genuinely ran two
  concerns; the epic's own criterion (measured size *and* more than one
  concern) is met.
- Split further - primitives (`new_octahedron`, `new_cone`) vs mutation vs
  `Mesh` conversion: rejected as YAGNI. Those are all inherent methods on one
  type with one shared invariant (`self.triangles`); scattering them across
  files fragments the type's API for no dependency win.
- Make `slice` public: rejected, the task forbids public API changes.

**Counter-evidence weighed.** 20260731-172223's lesson is "look at the test
modules first - two that share no helper are a seam the author already found."
`builder.rs` had ONE test module, so that signal was absent here; the split
rests on the dependency-set argument alone, and the tests were partitioned to
follow the code (3 of 6 moved to `slice.rs`, sharing no helper with the rest).

`camera/shake.rs` (313 code) was the other size candidate and was **kept**:
one concern (trauma -> offset), and its bulk is a headless-app test rig that
belongs beside what it exercises.

## Comment calls (57 blocks)

Legend: **drop** = restates the code; **compact** = kept, rewritten as one
tagged `NOTE:` block; **doc** = kept as intent, promoted to a `///` doc
comment on the test fn it explains (rustdoc is exempt by design, and the
intent belongs to the test, not to a line inside it).

### `mesh/builder.rs` (9)

| Line | Comment | Call |
| --- | --- | --- |
| 241 | boundary vertices come in pairs / `chunks_exact` cannot panic | compact - guards a hazard |
| 297 | degenerate triangles would make `normalize` NaN | compact - guards `normalize_or_zero` |
| 331, 347 | "Recursively subdivide into four smaller triangles" x2 | drop - restates the four calls below |
| 558 | edge parallel to plane, denominator zero | doc (moved with the fn to `slice.rs`) |
| 575 | collapsed triangle has zero-length edges | doc |
| 589 | position-only mesh must decline, not panic | doc |
| 604, 614 | odd-length boundary must not index out of bounds / one pair forms a triangle | doc (merged into one) |

### `mesh/explode.rs` (12)

| Line | Comment | Call |
| --- | --- | --- |
| 64 | "Observe when an ExplodeMesh component is added" | drop - restates `add_observer` |
| 95, 118, 156, 216 | "Collect all mesh entities" / "Generate fragments" / "Attach the fragments" / "Build only the surviving" | drop - restate the code below |
| 144 | carried fragment has zero direction, bad normal non-finite | compact - guards the `Dir3::Y` fallback |
| 188 | queue entry is builder + last cut direction | compact - the `Vec3` in the tuple type is otherwise unexplained |
| 206 | plane missed the fragment, keep it intact | compact - explains a non-obvious branch |
| 259 | random normals, run many times | doc |
| 280 | index-less mesh declines gracefully | doc |
| 314 | same component shape as the example | drop - the spawn below shows it |
| 339 | every fragment needs a real handle | drop - restates the two asserts |

Also fixed: the module header used `///` on the `use` statement, so it
documented the import rather than the module and `mesh::explode` rendered with
no description at all. Now `//!`.

### `meth/lerp.rs` (6)

All six ("Adjust smoothing factor" / "Interpolate using Bevy's built-in lerp"
/ "Snap to target if very close", x2 impls) **drop** - pure restatement. One
**new** `NOTE:` earns its place instead: `powi(7)` is an unexplained magic
exponent, and the note records that it maps the `0..1` dial to a per-second
retention factor which `powf(dt)` then makes frame-rate independent.

### `meth/sphere.rs` (6)

`// -Z`, `// +X`, `// +Y` in two tests: **drop**. Each labels an assertion
whose expected value (`Vec3::new(0.0, 0.0, -1.0)`) already states the axis.

### `camera/shake.rs` (15)

| Line | Comment | Call |
| --- | --- | --- |
| 190 | Restore-before-Apply must be pinned; an empty `ChaseCameraSystems::Sync` drops the edges | compact - the module's core hazard, and an instance of the AGENTS.md empty-set ordering rule |
| 274 | reset is a floor, not a veto | compact - non-obvious input semantics |
| 320, 335 | overshoot clamps to zero / zero trauma yields zero | drop - restate the `assert_eq!` on the next line |
| 349, 353 | offset scales with amount / zeroed axis stays zero | doc (merged) |
| 399 | peak per axis 0.6, so bounded by that diagonal | doc |
| 416 | the accumulating-shake bug this module prevents | doc - the regression's whole point |
| 427 | advance past full decay (1.0/1.8 ~= 0.56 s) | compact - guards the magic `60` |
| 448, 458, 487, 510 | moving-base driver narration x4 | doc (first) + drop (rest) |
| 522 | guards the `last_kick.inverse()` restore order | doc |
| 569 | reset snaps back next frame | drop - the test name says it |

### `camera/chase.rs` (5)

| Line | Comment | Call |
| --- | --- | --- |
| 146 | PostUpdate avoids a one-frame lag | compact - explains a non-obvious schedule choice |
| 189 | `try_remove` in case the entity is despawned | compact - guards the fallible call |
| 208, 214, 236 | "Compute offset in the target's rotation frame" / "Smooth interpolation" / "Compute the look-at point" | drop - restate the expressions below |

### `camera/skybox.rs` (3)

| Line | Comment | Call |
| --- | --- | --- |
| 119 | only reinterpret if not already an array | compact - guards idempotency when several cameras share one handle |
| 121 | "Convert stacked image into a 6 layer array" | drop - restates `reinterpret_stacked_2d_as_array` |
| 125 | mark the view as a cubemap | compact - non-obvious setting; the reinterpret alone does not make a skybox |

### `camera/wasd.rs` (1)

| Line | Comment | Call |
| --- | --- | --- |
| 123 | PostUpdate ensures input has run | compact |

## Rustdoc audit

Two defects fixed, both factual:

1. `mesh/explode.rs` module header was `///` on a `use` item (above), so the
   module had no doc.
2. `meth/mod.rs` promised "the crate's *future* `tween` easing will wrap the
   same call". `tween` has shipped; the sentence now links `crate::tween`.

Everything else was left alone, including prose the pass would have written
differently - the epic forbids style rewrites.

## Convention decision: test intent as `///`, not `//`

Eleven kept comments in this cluster were promoted to `///` doc comments on
the `#[test]` fn they explain rather than tagged `NOTE:` blocks inside the
body. That needs stating, because `check-comment-tags.sh` exempts rustdoc and
rustdoc never renders items inside a `#[cfg(test)] mod`, so the route is
invisible to both tools and would otherwise read as a loophole.

It is deliberate and it has precedent: 13 `///`-on-`#[test]` comments already
exist in landed code, including `mesh/explode.rs` itself and both files of the
preceding epic task (`physics/pd_controller.rs`, `debug/inspector.rs`). The
line the pass drew:

- text that says what the TEST proves, or why it exists -> `///` on the fn.
  It describes the whole test, it survives the body being rewritten, and
  `cargo test -- --list` / an IDE shows it.
- text that guards a VALUE or a line inside the body (the 60-frame decay loop
  in `shake.rs`, the `chunks_exact(2)` boundary in `builder.rs`) -> tagged
  `NOTE:` block, because it is attached to that line and dies with it.

Folded into AGENTS.md's Conventions bullet in this task so the two remaining
epic clusters do not each re-decide it.
