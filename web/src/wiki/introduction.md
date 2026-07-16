# Introduction

`bevy_common_systems` is a collection of copy-pastable [Bevy](https://bevyengine.org)
utilities with one goal: build games faster. It bundles the gameplay
components, systems and plugins that almost every game ends up needing --
cameras, health, orbit motion, procedural meshes, a status-bar HUD, save/load,
juice effects, a modding event bus and more -- so a new project does not have to
rewrite them.

## Why it exists

Every Bevy game re-implements the same handful of things: a follow camera, a
health pool, a cooldown timer, some screen shake, a save file. None of it is
hard, but all of it is friction between an idea and a playable prototype.

This crate is that friction, already paid down. Each module is one small,
game-agnostic concern with an obvious API. Most add runtime behaviour through a
single `*Plugin`; pure-utility modules export plain types and functions. Modules
are self-contained enough to lift into a game on their own, and the crate as a
whole works as a normal dependency.

## Add it to your project

```toml
[dependencies]
bevy_common_systems = { git = "https://github.com/alexjercan/bevy-common-systems", tag = "v0.19.0" }
```

The crate's version tracks Bevy's minor: `0.19.x` targets Bevy `0.19.x`. Pin to
a release tag (`tag = "v0.19.0"`) for a stable build, or drop the `tag` to follow
the default branch. It also targets [avian3d](https://github.com/Jondolf/avian)
0.7.

## Features

The crate has no default features:

- `debug` -- compiles the [`debug`](../debug/) module (wireframe toggle, egui
  world inspector, avian gizmos, and the headless test harness). Pulls in
  `bevy-inspector-egui`.
- `dev` -- an alias that just enables `debug`.

```sh
# with the inspector and debug tools:
cargo run --example 01_sphere --features debug
```

## The prelude

Import the prelude, which aggregates every module's public API:

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
```

That single glob pulls in every module's `*Plugin`, component, and helper, plus
the `#[derive(EventKind)]` macro used by the [modding](../modding/) bus. You
almost never need to reach past it.

## How to read these docs

Start with the [Quickstart](../quickstart/) for the shortest path from an empty
`App` to a working feature, then read the [module conventions](../conventions/)
-- once you learn the shape one module follows, you know them all. After that,
the sidebar is a reference: one page per module, each with the plugin to add and
a worked example. The [example games](../examples/) tie it together: fourteen
small, complete games, each headlining one or more modules, all playable in the
browser.
