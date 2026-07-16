# time

Timing helpers for games. The core type is `Cooldown`, a countdown for the
"you can do this again in N seconds" pattern -- weapon-fire gates and post-hit
invulnerability windows. There is no plugin: you `tick` it from whatever system
already runs, since a game usually carries several cooldowns on one entity.

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
```

For a plain "spawn something every N seconds" cadence you do not need this
module -- Bevy's `Timer` in `Repeating` mode already is the primitive (see the
recipe in the module docs and `06_fruitninja`).

## Cooldown

`Cooldown` gets the semantics a cooldown actually wants, which a raw `Timer` gets
backwards: a fresh `Cooldown::new(duration)` is **ready** (remaining zero), so
the first action fires immediately, whereas a fresh `Timer` in `Once` mode is not
finished. The core loop is `trigger()` to start the wait, `tick(dt)` to advance
it (clamped at zero, a no-op once ready), and `ready()` to gate the action:

```rust
weapon.fire.tick(time.delta_secs());
if weapon.fire.ready() {
    weapon.fire.trigger(); // ... spawn a bullet ...
}
```

It derives `Component` (so a lone cooldown can attach directly) and is a plain
`Copy` value you can embed in your own component. Read `remaining()`,
`duration()`, and `fraction()` (remaining as `0.0..=1.0`) to drive a cooldown
gauge or blink.

## Fire gates

The `10_asteroids` ship carries its cooldowns on a component and gates firing on
`ready()`. A fresh `Cooldown::new` starts ready, so the very first shot fires at
once:

```rust
#[derive(Component)]
struct Ship {
    fire: Cooldown,
    invuln: Cooldown,
}

// spawn: fire cooldown starts ready
commands.spawn(Ship {
    fire: Cooldown::new(FIRE_COOLDOWN),
    invuln: Cooldown::started(INVULN_TIME),
});

fn fire(wants_fire: bool, mut ship: Single<&mut Ship>) {
    if !wants_fire || !ship.fire.ready() {
        return;
    }
    ship.fire.trigger(); // ... spawn a bullet ...
}
```

## Invulnerability frames

The same type models a post-hit i-frame window: `ready()` means vulnerable again.
Use `Cooldown::started(duration)` when the entity should begin *on* cooldown
(e.g. a ship that spawns already invulnerable), and `trigger_for(seconds)` to
start a variable-length window independent of the construction `duration` (i-frames
scaled by the hit). `10_asteroids` triggers i-frames only when currently
vulnerable, and drives a blink off `fraction()`:

```rust
ship.invuln.tick(time.delta_secs());

// on a hit, only if not already invulnerable
if ship_hit && ship.invuln.ready() {
    ship.invuln.trigger();
    // ... apply the hit ...
}

// blink while the i-frames run so they read clearly
let visible = ship.invuln.ready() || (time.elapsed_secs() * 18.0).sin() > 0.0;
```

Pairs well with [health](../health/) for the hit itself and [feedback](../feedback/)
for the blink.
