# DefaultPlugins doctests panic in a headless (CI) run

## Symptom

`cargo test` reported six failing doctests:

- `src/helpers/pointer.rs - helpers::pointer (line 23)`
- `src/input/pointer.rs - input::pointer (line 22)`
- `src/persist/mod.rs - persist (line 20)`
- `src/scoring/high_score.rs - scoring::high_score (line 14)`
- `src/tween/mod.rs - tween (line 23)`
- `src/ui/touchpad.rs - ui::touchpad (line 24)`

They pass in the nix devshell (which has `DISPLAY` set) but fail in CI /
any headless run.

## Root cause

`cargo test --doc` *runs* doctests, it does not only compile them (already
noted in AGENTS.md's gotchas about `StatesPlugin`). All six snippets execute

```rust
App::new()
    .add_plugins(DefaultPlugins)
    ...;
```

at the top level of the snippet, which rustdoc wraps in `fn main()` and runs.
`DefaultPlugins` pulls in `WinitPlugin`, whose `build` does
`EventLoop::new().expect("Failed to build event loop")`. With no display the
event loop cannot be created and the doctest panics:

```
thread 'main' panicked at bevy_winit-0.19.0/src/lib.rs:128:
Failed to build event loop: Os(... "neither WAYLAND_DISPLAY nor WAYLAND_SOCKET
nor DISPLAY is set.")
```

CI (`.github/workflows/ci.yml`) installs only Bevy's *build* deps (alsa, udev,
wayland/xkb headers), no runtime graphics stack and no virtual display, so the
`App::new().add_plugins(DefaultPlugins)` runs headless and panics. The nix
devshell masks it because a real `DISPLAY` is present.

Reproduce locally: `env -u DISPLAY -u WAYLAND_DISPLAY cargo test --doc`.

## Why some `DefaultPlugins` doctests were fine

`src/camera/post.rs` uses `DefaultPlugins` too but did not fail: its App-build
lives inside a hidden `# fn demo(...) { ... # }`. rustdoc compiles that function
but never calls it (the wrapping `main()` only *defines* it), so `WinitPlugin`
never runs. The six failing modules put the App-build at the top level, so it
executed.

## Fix

Match the existing `camera/post.rs` convention: wrap the App-build in a hidden
`# fn wire_up() { ... # }` in each of the six modules. The snippet still
compiles (so the wiring is type-checked and stays a real doc example) but is
never executed, so no window / GPU / event loop is required.

Verified with `env -u DISPLAY -u WAYLAND_DISPLAY cargo test --doc`: 54 passed,
0 failed.

## Alternatives considered

- `no_run` on the fence (as `debug/harness` uses): also works, but the
  hidden-fn wrapper matches the sibling `camera/post.rs` doctest exactly and
  keeps the whole snippet compile-checked, so it is the more consistent choice.
- Swapping `DefaultPlugins` for `MinimalPlugins`: would change what the example
  demonstrates and risk fresh runtime panics from plugins that expect render /
  asset infrastructure. Rejected.
