# health

A minimal health system for game entities: a `Health` component, a damage event
that propagates up the entity hierarchy, and a marker inserted when an entity
hits zero health. It owns only the bookkeeping -- what "death" means is the
game's job, wired through an observer.

Add `HealthPlugin` once and you get the damage observer:

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

## Health

`Health` tracks `current` and `max` (both `f32`). `Health::new(max)` starts
`current` equal to `max`. Health is clamped to zero and never exceeds `max`.

```rust
commands.spawn((
    Ship,
    Health::new(100.0), // current == max == 100.0
));
```

## HealthApplyDamage

`HealthApplyDamage` is the way to deal damage -- trigger it, do not mutate
`Health` by hand. It carries the target `entity`, an optional `source` entity,
and the `amount`:

```rust
commands.trigger(HealthApplyDamage {
    entity: target,
    source: Some(attacker),
    amount: 25.0,
});
```

It is an `EntityEvent` with `propagate` + `auto_propagate`, so a hit on a child
bubbles up the hierarchy to ancestors that also carry `Health` (e.g. an aggregate
hull that sums its sections). The observer bubbles up only the amount that
actually landed on each node: `amount` is clamped to the node's remaining health
before propagation continues, so overkill on a 100 hp child costs a parent 100,
not 1000. A node already at zero (or carrying `HealthZeroMarker`) absorbs
nothing.

## HealthZeroMarker

When a node's `current` reaches zero, the `on_damage` observer sets `current` to
`0.0` and inserts `HealthZeroMarker`. It is a plain marker component you read to
run destruction logic -- despawn, spawn loot, play an effect. The
[integrity](../integrity/) pipeline builds directly on it: a zeroed node becomes
disabled and, if it is a leaf, destroyed.

## Reacting to death

React to death by observing `On<Add, HealthZeroMarker>` and filtering to the
entities you care about. This is exactly what `10_asteroids` does for the player
ship:

```rust
fn on_player_died(
    add: On<Add, HealthZeroMarker>,
    q_ship: Query<(), With<Ship>>,
    mut commands: Commands,
) {
    // The marker can land on anything with Health; keep only the ship.
    if !q_ship.contains(add.entity) {
        return;
    }
    // ... kick a camera shake, flash red, head to the game-over screen ...
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, HealthPlugin))
        .add_observer(on_player_died)
        .run();
}
```

Because damage propagates, healing or reviving means removing the marker
yourself: `12_bastion` calls `commands.entity(core).remove::<HealthZeroMarker>()`
when its core is repaired.

See also [feedback](../feedback/) for the flash effect and [camera](../camera/)
for the shake.
