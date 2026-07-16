# audio

The `audio` module is fire-and-forget one-shot sound effects. Games trigger a
`PlaySfx` (or call `commands.play_sfx(handle)`) and `SfxPlugin` spawns a
self-despawning `AudioPlayer` for it, so game code never repeats the
`AudioPlayer` / `PlaybackSettings::DESPAWN` boilerplate. A `SoundBank` registry
keeps the loaded handles keyed by a game-defined enum. This is transient SFX
only -- for looping music, spawn an `AudioPlayer` with `PlaybackSettings::LOOP`
directly.

All snippets assume:

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
```

## SfxPlugin

Add `SfxPlugin` to enable the `PlaySfx` observer and the global
`SfxMasterVolume` resource. Add it once at startup:

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SfxPlugin)
        .run();
}
```

The `SoundBank<K>` registry pairs well with it: declare a small `Copy` key enum
and load handles under the `assets/sounds/<name>.wav` convention, then read them
back with `get`. `SoundBank::all_loaded` and the `sounds_loaded::<K>`
run-condition give an opt-in "wait for assets" gate.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Sfx {
    Click,
    GameOver,
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    // Loads assets/sounds/click.wav and assets/sounds/game_over.wav.
    commands.insert_resource(SoundBank::load(
        &assets,
        [(Sfx::Click, "click"), (Sfx::GameOver, "game_over")],
    ));
}
```

## Playing a sound

The `SfxCommandsExt` trait adds `play_sfx(handle)` and
`play_sfx_volume(handle, volume)` to `Commands`. For per-shot volume and pitch,
trigger a `PlaySfx` directly with its `with_volume` / `with_speed` builders
(speed also shifts pitch, handy for a rising combo).

```rust
fn on_slice(mut commands: Commands, sfx: Res<SoundBank<Sfx>>) {
    // Simplest: play once at master volume.
    commands.play_sfx(sfx.get(Sfx::Click));

    // A quieter shot.
    commands.play_sfx_volume(sfx.get(Sfx::GameOver), 0.9);

    // Full control for per-shot pitch variation.
    commands.trigger(PlaySfx::new(sfx.get(Sfx::Click)).with_volume(0.8).with_speed(1.2));
}
```

## Master volume

`SfxMasterVolume(f32)` (default `1.0`) scales every sound before playback,
multiplied by each shot's own volume. Set it from a volume slider, or to `0.0`
to mute everything at once:

```rust
fn set_volume(mut master: ResMut<SfxMasterVolume>, slider: f32) {
    master.0 = slider; // 0.0 mutes, 1.0 is unchanged
}
```

Pair sound with [feedback](../feedback/) effects for a full hit reaction.
