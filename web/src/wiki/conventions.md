# Module conventions

Every module in the crate follows the same shape. Once you learn it, every other
module reads the same way -- which is the whole point of a "common systems"
crate. This page is that shape.

## The plugin

Most modules headline a single `*Plugin` that registers their systems and
observers. You add it once:

```rust
App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(HealthPlugin)
    .add_plugins(SfxPlugin)
    .run();
```

Pure-utility modules ([meth](../meth/), [tween](../tween/)) have no runtime
behaviour, so they have no plugin -- they export plain types and functions you
call directly.

## Config, Input, Output

Behavioural modules split an entity's state into up to three components, so the
data you set once, the data you write each frame, and the data you read each
frame never get tangled:

- **Config** -- a component named after the feature, holding the tunables. You
  spawn it once and rarely touch it again (a `Cooldown`'s duration, a chase
  camera's smoothing).
- **`*Input`** -- what your systems *write* each frame to drive the module (a
  target rotation, a throttle, a desired heading).
- **`*Output`** -- what the module *computes* for you to read and apply, or a
  direct `Transform` write when the module owns the entity's motion.

For example, the [transform](../transform/) orbit driver reads a `SphereOrbit`
config plus your per-frame input and writes a `SphereOrbitOutput` you copy onto
the entity's `Transform`. The [physics](../physics/) PD controller reads a target
rotation and applies torque to the avian rigid body directly. The split is the
same either way: set config once, write input each frame, read output each
frame.

This is a convention, not a hard rule -- a module uses only the pieces it needs.
[health](../health/) has no per-frame input at all; it is driven entirely by the
`HealthApplyDamage` event. When a module needs to *act on* an entity from
anywhere, it takes an entity event (which propagates up the hierarchy) rather
than a component, so any system can trigger it without a reference.

## The prelude

Every module re-exports its public surface under `module::prelude`, and the
crate prelude globs them all together:

```rust
use bevy_common_systems::prelude::*;
```

Reach for the crate prelude by default; drop to a module prelude
(`use bevy_common_systems::health::prelude::*;`) only if you want to pull in one
module in isolation.

## Feature flags

The crate has no default features. The [debug](../debug/) module is gated behind
the `debug` feature (aliased as `dev`) because it pulls in the egui inspector; a
release build leaves it out entirely. Everything else is always compiled. See
the [introduction](../introduction/#features) for the flag list.
