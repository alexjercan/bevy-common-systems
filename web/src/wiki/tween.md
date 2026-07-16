# tween

The `tween` module is a narrow, duration-based value tween over a Bevy
`EaseFunction`. Where [meth](../meth/)'s `LerpSnap` does open-ended smoothing
toward a moving target, a `Tween` animates a value from a fixed `start` to a fixed
`end` over a fixed `duration`, shaped by an easing curve. It is the shared
bookkeeping behind "ease something from A to B over N seconds" -- a slice pop, a
menu pulse, a popup fade -- not a keyframe-timeline system.

## Tween

`Tween<T>` is a component that interpolates a `TweenValue` (implemented for `f32`,
`Vec2`, `Vec3`, `Vec4`) from `start` to `end` over `duration` seconds, shaped by
an `ease`. Construct it with `Tween::new(start, end, duration, ease)`. Add
`TweenPlugin` and it advances every built-in tween each frame in
`TweenSystems::Advance`.

Following the crate's output pattern, a tween is an output: the plugin advances
it and you read `Tween::value()` to apply the current value wherever you want (a
`Transform` field, a color, a `Node` position), so one component drives any
target with no per-field adapters.

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

fn setup(app: &mut App) {
    app.add_plugins(TweenPlugin);
}

// Pop a sprite from 1x to 1.5x scale over 0.15s, then remove the tween.
fn spawn_pop(mut commands: Commands) {
    commands.spawn((
        Transform::default(),
        Tween::new(Vec3::ONE, Vec3::splat(1.5), 0.15, EaseFunction::QuadraticOut),
    ));
}
```

On completion the plugin applies the `TweenOnComplete` policy and inserts a
`TweenFinished` marker (so an `On<Add, TweenFinished>` observer can run a side
effect). The policy defaults to `TweenOnComplete::Remove`; set it builder-style:

- `TweenOnComplete::Keep` -- leave the finished tween in place, holding at `end`
  (useful when another system keeps reading the end value).
- `TweenOnComplete::Remove` -- drop the `Tween<T>` component, keep the entity.
- `TweenOnComplete::Despawn` -- despawn the whole entity (a fire-and-forget one-shot).

```rust
// A pop that stays at its end value so the reader can keep applying it.
let pop = Tween::new(1.25, 1.0, 0.15, EaseFunction::QuadraticOut)
    .with_on_complete(TweenOnComplete::Keep);
```

## Ease functions

The curve is a Bevy `EaseFunction`, sampled internally with `sample_clamped`, so
any of Bevy's easing variants apply directly -- `EaseFunction::Linear`,
`QuadraticIn` / `QuadraticOut`, `BackOut`, and so on. The example game
`examples/13_glide` uses `EaseFunction::BackOut` for a tile pop overshoot and
`EaseFunction::QuadraticOut` for slides and a rolling score readout.

`Tween` exposes the raw and eased progress if you need them: `fraction()` is the
un-eased `0..=1` time, `eased_fraction()` passes it through the `EaseFunction`,
and `finished()` reports whether the duration is reached.

```rust
// Slide a UI tile with an overshoot curve.
let slide = Tween::new(start, end, 0.2, EaseFunction::BackOut);
```

## Driving a value

The plugin only advances the tween; you apply `value()` yourself in a system
ordered `.after(TweenSystems::Advance)`. This is how `13_glide` drives its UI:
each tile carries a `Tween` whose current value is copied into a plain field, and
the score number rolls to its new total on a `Tween<f32>` read back each frame.

```rust
// Apply the current scale each frame, after the tween has advanced.
fn apply_scale(mut q: Query<(&mut Transform, &Tween<Vec3>)>) {
    for (mut transform, tween) in &mut q {
        transform.scale = tween.value();
    }
}

// A score readout that reads its Tween<f32> back each frame (a Keep tween).
fn roll_score(mut q: Query<(&mut Text, &Tween<f32>)>) {
    for (mut text, tween) in &mut q {
        text.0 = format!("{}", tween.value().round() as i64);
    }
}
```

Order this after `TweenSystems::Advance` so you read the freshly advanced value.
A `Vec4` tween is the natural fit for a color (its linear-RGBA channels), which is
how `13_glide` flashes a tile's `BackgroundColor` on a merge.
