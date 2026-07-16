# mesh

The `mesh` module gives you procedural triangle-mesh generation and a plugin that
shatters an existing mesh into fragments. It solves two recurring needs -- building
custom geometry (spheres, planets, cones) at runtime, and destroying meshes into
pieces for hit effects -- without hand-writing vertex buffers or slice math.

## TriangleMeshBuilder

`TriangleMeshBuilder` stores a `Vec<Triangle3d>` and knows how to turn it into a
Bevy `Mesh`. You start from a primitive (`new_octahedron`, `new_cone`) or an
existing mesh (`new`, `From<Mesh>`, or the fallible `try_from_mesh`), transform the
triangles, then call `build` to produce the `Mesh`.

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

fn setup(mut meshes: ResMut<Assets<Mesh>>) {
    // A subdivided octahedron approximates a unit sphere.
    let mesh = TriangleMeshBuilder::new_octahedron(3).build();
    let _handle = meshes.add(mesh);
}
```

Useful methods: `add_triangle`, `subdivide(depth)`, `with_scale(Vec3)`,
`apply_noise(&noise_fn)`, `slice(plane_normal, plane_point)`, and the inspectors
`vertices_and_indices`, `normals`, `uvs`, `is_empty`. `build` comes from the
`MeshBuilder` trait and fills in positions, per-face normals and planar UVs.

## Spheres and planets

`new_octahedron(resolution)` recursively subdivides an 8-face octahedron using
spherical interpolation, so every vertex lands on the unit sphere. Feed the result
through `apply_noise` -- which displaces each vertex along its normalized direction
by a 3D noise sample -- to get a lumpy planet. This is exactly what
`examples/02_planet.rs` does.

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

fn spawn_planet(mut meshes: ResMut<Assets<Mesh>>) {
    let perlin = noise::Fbm::<noise::Perlin>::new(0);
    let mesh = TriangleMeshBuilder::new_octahedron(3)
        .apply_noise(&perlin)
        .build();
    let _handle = meshes.add(mesh);
}
```

`new_cone(radial_subdivisions, height_subdivisions)` builds a capped cone (tip at
`+Y`, base radius `1.0`) the same way.

## Slicing a plane

`slice(plane_normal, plane_point)` cuts the mesh along an infinite plane and
returns `Some((positive_side, negative_side))` -- two fresh builders, each with the
cut face filled in -- or `None` when one side comes out empty. This is the
primitive the explode plugin drives.

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

fn cut(builder: &TriangleMeshBuilder) -> Option<(TriangleMeshBuilder, TriangleMeshBuilder)> {
    // Slice through the origin along the X axis.
    builder.slice(Vec3::X, Vec3::ZERO)
}
```

## ExplodeMeshPlugin

`ExplodeMeshPlugin` watches for the `ExplodeMesh` component. When you insert it on
an entity, an observer collects that entity's mesh (and its children's meshes),
recursively slices each one along random planes into roughly `fragment_count`
pieces, and writes the results into an `ExplodeFragments` component. Each
`ExplodeFragment` carries the `origin` entity, a `mesh: Handle<Mesh>`, and a
normalized `direction: Dir3` you can use to fling the piece.

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

fn build(app: &mut App) {
    app.add_plugins(ExplodeMeshPlugin);
}

// Trigger an explosion (see examples/05_explode.rs):
fn explode(mut commands: Commands, target: Entity) {
    commands.entity(target).insert(ExplodeMesh { fragment_count: 8 });
}

// React to the fragments once the plugin produces them:
fn on_fragments(insert: On<Insert, ExplodeFragments>, q: Query<&ExplodeFragments>) {
    if let Ok(fragments) = q.get(insert.entity) {
        for fragment in fragments.iter() {
            let _ = (fragment.origin, &fragment.mesh, fragment.direction);
        }
    }
}
```

The mesh entities must have `Mesh3d` and `MeshMaterial3d<StandardMaterial>`. For the
glowing look many effects want, build fragment materials with
[material](../material/); to shake the camera on impact, see [camera](../camera/).
