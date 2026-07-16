# feedback

The `feedback` module collects short-lived "juice" effects that give a hit or
pickup a visible kick: a per-entity material `flash` and a full-screen
`screen_flash` overlay. Both ride the crate's [tween](../tween/) module for their
timing, so a hit reads as a crisp pop that eases back on its own.

All snippets assume:

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
```

## flash

`FlashPlugin` briefly overrides an entity's `StandardMaterial` channel with a
flash color, then eases it back over a duration -- the white / red pop an enemy or
the player shows the instant it is hit. Insert a `Flash` on the entity that owns
the `MeshMaterial3d<StandardMaterial>`. The interesting part is that it never
mutates a shared material: on insert it clones the material into a per-entity
asset, animates the clone, and restores the original handle (freeing the clone)
when done -- so bystanders sharing the handle stay untouched.

`Flash` has `color`, `duration`, and `channel` (a `FlashChannel`, either
`Emissive` -- the default, which blooms under post -- or `BaseColor`).

```rust
fn on_hit(mut commands: Commands, entity: Entity) {
    commands.entity(entity).insert(Flash {
        color: Color::srgb(1.0, 0.2, 0.2),
        duration: 0.25,
        ..default()
    });
}
```

Re-inserting `Flash` on an already-flashing entity restarts it from full.

## screen_flash

`ScreenFlashPlugin` drives a full-screen UI overlay whose alpha spikes to a peak
and decays to transparent -- the red flash a game slams across the screen on a hit
or death. The overlay is a full-screen `Node` with a `BackgroundColor`; the plugin
only touches the alpha, so the caller picks the tint. Two shapes fall out of the
one `ScreenFlash` component (`peak_alpha`, `decay`, `despawn_on_end`):

- Spawn-and-fade (one-shot on death): `screen_flash(color, peak_alpha, decay)`
  spawns an overlay that fades over `1 / decay` seconds and despawns itself.
- Spike-and-decay (reusable hit overlay): spawn a persistent overlay once with
  `despawn_on_end: false`, then re-insert `ScreenFlash` on each hit to re-spike it.

```rust
fn on_death(mut commands: Commands) {
    // Red flash that fills the screen, fades over ~0.35s, then despawns.
    commands.spawn(screen_flash(Color::srgb(0.9, 0.1, 0.1), 0.5, 1.0 / 0.35));
}
```

`screen_flash_node()` returns just the bare full-screen node if you want to build
a persistent overlay with your own `BackgroundColor` / marker / `GlobalZIndex`.

## Wiring it to damage

The 06_fruitninja example uses both together: a persistent-or-one-shot
`screen_flash` on the death beat, plus a per-target `Flash` on the sliced object.
The pattern is to react to your own damage event and insert the effect:

```rust
#[derive(Event)]
struct Damaged {
    entity: Entity,
}

fn on_damaged(mut ev: MessageReader<Damaged>, mut commands: Commands) {
    for Damaged { entity } in ev.read() {
        // Local pop on the thing that got hit...
        commands.entity(*entity).insert(Flash {
            color: Color::WHITE,
            duration: 0.15,
            ..default()
        });
        // ...and a full-screen sting for the player.
        commands.spawn(screen_flash(Color::srgb(0.9, 0.1, 0.1), 0.5, 3.0));
    }
}
```

Pair it with [audio](../audio/) for the sound and [health](../health/) for the
damage source.
