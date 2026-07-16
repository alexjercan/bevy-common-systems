# scoring

Small, game-agnostic building blocks for scoring: a decaying hit-`Streak`
counter, and a generic `HighScore<T>` best-score resource with a "new best"
edge. There is deliberately no `Score` type -- a running score is a bare
`usize`/`f32` the game already owns. This module owns only the parts with real,
re-derived logic. There is no plugin: you tick and record from your own systems.

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
```

## Streak

`Streak` counts hits that land in quick succession and decays when the player
goes quiet. Construct it with `Streak::new(window)` (window length in seconds),
call `hit()` on each scoring event (it bumps the count and refreshes the window,
returning the new count), and `tick(dt)` each frame to advance the decay. `tick`
returns `Some(final_count)` on the frame the streak ends, so you can flash a
tally, and `None` otherwise.

It derives `Resource`, so `07_orbit` inserts it directly:

```rust
// setup
app.insert_resource(Streak::new(STREAK_WINDOW));

// on each hit: scale the hit's value by the streak length
fn on_hit(mut streak: ResMut<Streak>) {
    let count = streak.hit(); // 1, 2, 3, ...
    // ... award points based on count ...
}

// each frame: advance the decay
fn tick_streak(time: Res<Time>, mut streak: ResMut<Streak>) {
    streak.tick(time.delta_secs());
}
```

Other helpers: `extend_to(seconds)` lengthens the window without bumping the
count (a bonus that buys time), `reset()` clears it with no tally, and
`count()`/`is_active()`/`remaining()`/`remaining_frac()`/`window()` read the
state for a combo HUD. It owns only the count-and-decay bookkeeping; what a hit
is *worth* stays in the game.

## HighScore

`HighScore<T>` holds the best value seen so far plus whether the last `record`
beat it. It is generic over any `PartialOrd + Copy` score type (`usize`, `u32`,
`f64`, ...). Insert it as a resource, `record` a run's score on game over, and
read `best()`:

```rust
// 07_orbit / 13_glide both use this
app.init_resource::<HighScore<usize>>();

fn record_high_score(score: Res<Score>, mut high: ResMut<HighScore<usize>>) {
    high.record(score.value);
}

fn show(high: Res<HighScore<usize>>) {
    let _best = high.best(); // ... "Best: {best}" ...
}
```

Start from a non-zero floor with `HighScore::new(initial)`. It derives
`Serialize`/`Deserialize`, so it composes with
`PersistPlugin::<HighScore<u32>>` to survive a restart -- `13_glide` persists its
board's high score this way. See [persist](../persist/).

## The new-best edge

`record(score)` returns `true` when the score strictly beats the current best (a
tie is *not* a new best), and stashes the same answer in `is_new_best()` for
reading later on the game-over screen:

```rust
fn on_game_over(score: u32, mut high: ResMut<HighScore<u32>>) {
    if high.record(score) {
        // ... flash "New best!" ...
    }
}
```

The `new_best` flag is per-run state: it is not serialized (a loaded score never
reports a new best), and you can clear it with `clear_new_best()` when starting a
fresh run without touching the stored best.
