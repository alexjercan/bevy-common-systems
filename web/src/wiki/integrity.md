# integrity

A destruction pipeline for structures built from connected, health-bearing
nodes. Model a destructible object as a graph: each node is a health-bearing
collider carrying `ConnectedTo` (its structural neighbours), all under one body
marked `IntegrityRoot`. `IntegrityPlugin` turns collisions and blast volumes into
damage, disables nodes at zero health, destroys disabled leaves, and cascades the
destruction as nodes are pruned from the graph.

The game owns the two ends: it builds the graph, and it decides what a destroyed
node does. Everything in between is this module. It builds on
[health](../health/) and requires avian's `PhysicsPlugins`.

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HealthPlugin)
        .add_plugins(IntegrityPlugin)
        .run();
}
```

## Components

Building the graph is the game's job, using these node-local components:

- `IntegrityRoot` -- marks the body that owns the structure (the `RigidBody`
  whose colliders are the nodes, or a lone boulder). Its disabling destroys the
  whole body.
- `ConnectedTo(pub Vec<Entity>)` -- a node's structural neighbours. A node with
  one or zero neighbours is a leaf.
- `IntegrityLeafMarker` -- derived automatically from `ConnectedTo` by the plugin.
- `IntegrityDisabledMarker` -- inserted automatically when a node gains
  `HealthZeroMarker`.
- `IntegrityDestroyMarker` -- inserted the frame a node is destroyed; the public
  seam you observe.

```rust
// A node: a collider under the root, with health and its neighbours.
commands.spawn((
    ChildOf(root),          // root carries IntegrityRoot
    Collider::sphere(0.5),
    Health::new(50.0),
    ConnectedTo(vec![left, right]),
));
```

## Applying damage

You rarely trigger damage by hand: `IntegrityPlugin` installs collision
observers that funnel into `HealthApplyDamage`. A fast impact deals
impulse/energy damage scaled by relative velocity and mass (a near-stationary
graze is gated out), and a blast sensor deals radial falloff damage. Any node
that carries both `ColliderOf` and `Health` has collision events enabled for it
automatically. You can of course still trigger `HealthApplyDamage` yourself for
scripted damage.

## Blast damage

`blast_damage(BlastDamageConfig { radius, max_damage })` returns a bundle for a
static sensor sphere that deals `max_damage` at its centre, falling off linearly
to zero at `radius`. It carries `BlastDamageMarker` so the observers tell the
blast side of an overlap from the target side, and it owns its collision events
so it reaches every overlapped body regardless of the target's own config. Pair
it with a short `TempEntity` so the volume cleans itself up:

```rust
fn detonate(mut commands: Commands, at: Vec3) {
    commands.spawn((
        blast_damage(BlastDamageConfig { radius: 6.0, max_damage: 80.0 }),
        Transform::from_translation(at),
        TempEntity(0.1),
    ));
}
```

Each overlap deals damage exactly once (the swapped event ordering is ignored),
so a target never double-dips.

## Destruction

The lifecycle, all driven by observers: health hits zero ->
`IntegrityDisabledMarker`; a disabled *leaf* (or a disabled `IntegrityRoot`) ->
`IntegrityDestroyMarker`; destroying a node prunes it from its neighbours'
`ConnectedTo` lists, which can turn them into leaves and cascade the destruction
through the structure. A disabled *interior* node is merely deactivated, not
destroyed.

The plugin only inserts `IntegrityDestroyMarker` and prunes the graph -- it never
decides what "destroyed" looks like. React to it as the public seam:

```rust
fn on_destroyed(add: On<Add, IntegrityDestroyMarker>, mut commands: Commands) {
    // ... slice the mesh, spawn debris, or despawn the entity ...
    commands.entity(add.entity).despawn();
}
```

`examples/15_integrity.rs` builds a grid, damages it with a blast, and hooks the
destroy marker to the [mesh](../mesh/) slicer. See also [physics](../physics/).
