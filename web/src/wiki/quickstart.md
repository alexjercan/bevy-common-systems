# Quickstart

The shortest path from an empty `App` to a working feature. We will stand up the
[health](../health/) system, spawn something with a health pool, and damage it --
enough to see the shape every module shares.

First, [add the crate](../introduction/#add-it-to-your-project) and import the
prelude.

## A minimal app

Add the plugin you want and use its components. Most modules headline a single
`*Plugin`:

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HealthPlugin)
        .run();
}
```

`HealthPlugin` registers the systems that make `Health` live: it watches for
damage events and inserts a marker when a pool hits zero. Adding the plugin is
all the wiring a module needs.

## Add a system

Spawn an entity with a health pool from any system -- `Health` is just a
component:

```rust
fn spawn(mut commands: Commands) {
    commands.spawn(Health::new(100.0));
}
```

## Trigger an event

Modules that need to *do* something to an entity take an entity event, so any
system can drive them without holding a reference to the plugin. Damage is one:

```rust
fn hurt(mut commands: Commands, target: Entity, attacker: Entity) {
    commands.trigger(HealthApplyDamage {
        entity: target,
        source: Some(attacker),
        amount: 25.0,
    });
}
```

When the pool reaches zero, `HealthPlugin` inserts a `HealthZeroMarker` you can
react to -- to play a sound, spawn a wreck, or despawn the entity. See the
[health](../health/) page for the full flow.

## Next steps

- Read the [module conventions](../conventions/) to internalize the
  plugin / config / `Input` / `Output` shape.
- Browse the modules in the sidebar -- each has a worked example like this one.
- Play the [example games](../examples/) to see the modules combined into real,
  complete games.
