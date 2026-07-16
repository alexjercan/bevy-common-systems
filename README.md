# Bevy Common Systems

`bevy_common_systems` is a collection of copy-pastable [Bevy](https://bevyengine.org)
utilities with one goal: build games faster. It bundles the gameplay
components, systems and plugins that almost every game ends up needing --
cameras, health, orbit motion, procedural meshes, a status-bar HUD, save/load,
juice effects, a modding event bus and more -- so a new project does not have
to rewrite them.

Each module is one small, game-agnostic concern with an obvious API. Most add
runtime behaviour through a single `*Plugin`; pure-utility modules export plain
types and functions. Modules are self-contained enough to lift into a game on
their own, and the crate as a whole works as a normal dependency.

This repo also ships fourteen runnable example games (`examples/NN_name.rs`)
that double as the integration tests and the quickstart documentation, plus a
[web showcase](web/) that serves them as WebAssembly builds.

## Add it

```toml
[dependencies]
bevy_common_systems = { git = "https://github.com/alexjercan/bevy-common-systems", tag = "v0.19.0" }
```

The crate's version tracks Bevy's minor: `0.19.x` targets Bevy `0.19.x`. Pin to a
release tag (`tag = "v0.19.0"`) for a stable build, or drop the `tag` to follow
the default branch. It also targets avian3d 0.7. It has no default features:

- `debug` - compiles the `debug` module (wireframe toggle, egui world
  inspector, avian gizmos, and the headless test harness). Pulls in
  `bevy-inspector-egui`.
- `dev` - an alias that just enables `debug`.

Then import the prelude, which aggregates every module's public API:

```rust
use bevy_common_systems::prelude::*;
```

## Quickstart

Add the plugins you want and use their components. For example, the health
system:

```rust
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HealthPlugin)
        .run();
}

fn spawn_and_damage(mut commands: Commands, entity: Entity, attacker: Entity) {
    // An entity with a health pool.
    commands.spawn(Health::new(100.0));

    // Apply damage from any system; a HealthZeroMarker is inserted at zero.
    commands.trigger(HealthApplyDamage {
        entity,
        source: Some(attacker),
        amount: 25.0,
    });
}
```

Every module follows the same shape: a `*Plugin` you add, a config component
named after the feature, a `*Input` component your systems write each frame,
and a `*Output` component (or direct `Transform` writes) your systems read.

## What's inside

- `audio` - `SfxPlugin`: fire-and-forget one-shot sound effects. Trigger
  `PlaySfx` (or `commands.play_sfx(handle)`); a global `SfxMasterVolume`
  scales everything.
- `camera` - `chase` (third-person follow with smoothing and look-ahead),
  `wasd` (first-person free-fly camera math), `post` (tonemapping + bloom),
  `skybox` (a stacked 6-face image into a cubemap), `shake` (trauma-driven
  screen shake), `project` (screen <-> world projection helpers).
- `debug` (feature `debug`) - `wireframe` and `inspector` toggles plus
  `harness`: env-gated headless verification (`AutopilotPlugin` drives a
  game's state machine, `ScreenshotPlugin` captures a frame).
- `feedback` - short-lived "juice": `flash` (material hit-flash) and
  `screen_flash` (full-screen damage vignette).
- `health` - `HealthPlugin`: a `Health` component, the `HealthApplyDamage`
  entity event (propagates up the hierarchy), and a `HealthZeroMarker`.
- `helpers` - `DespawnEntity` (despawn now), `TempEntity` (auto-despawn after
  N seconds), and the WASD input controller binding.
- `input` - a unified mouse + touch + cursor pointer resource, cursor-grab
  helpers, and small input-state utilities.
- `material` - `glowing_material`: the emissive `StandardMaterial` that
  actually blooms.
- `mesh` - `TriangleMeshBuilder` (procedural triangle meshes: octahedron
  spheres, subdivision, noise displacement, plane slicing) and
  `ExplodeMeshPlugin` (slice a mesh into flying fragments).
- `meth` - vector math (the name is an intentional pun): the `LerpSnap`
  smoothing trait plus spherical coordinates and `slerp`.
- `modding` - a generic, serde-friendly event bus for modding and scripting:
  `EventWorld`, `EventKind`, `EventHandler` entities, and a JSON-authored
  `EventHandlerRegistry`. Event payloads travel as `serde_json::Value`.
- `persist` - cross-platform save/load of a Bevy `Resource` (native data-dir
  file on desktop, `localStorage` on wasm).
- `physics` - avian3d helpers: `pd_controller` (PD attitude torque toward a
  target rotation) and `doom_controller` (a Doom-style first-person character
  controller).
- `scoring` - a `Streak` counter that grows on each hit and decays when the
  player goes quiet, and a generic `HighScore<T>` best-score resource with a
  "new best" edge.
- `time` - `cooldown`: a countdown for fire gates and i-frames.
- `transform` - motion-driver components, each computing an `Output` you
  apply: sphere orbits (explicit, directional, random), point rotation, and
  smooth look rotation.
- `tween` - a narrow, duration-based value tween over a Bevy `EaseFunction`.
- `ui` - `status` (a screen-corner metrics HUD), `animate` (copy a `Tween`
  into a UI `Node` field), `menu` (screen/button builders), `popup` (floating
  "+N" text), and `touchpad` (reveal-on-first-touch controls).

The `#[derive(EventKind)]` macro used by the modding bus lives in the
re-exported `bevy_common_systems_macros` subcrate.

## Examples

The numbered examples are small, complete games that each headline one or more
modules. Run one with:

```sh
cargo run --example 01_sphere
# add the inspector and debug tools:
cargo run --example 01_sphere --features debug
```

- `01_sphere` - an octahedron sphere from `TriangleMeshBuilder` with a WASD
  camera.
- `02_planet` - the same mesh displaced with Fbm/Perlin noise: a planet.
- `03_modding` - the modding event bus end to end, including
  `#[derive(EventKind)]` and JSON-authored handlers.
- `04_status_item` - the status-bar HUD with FPS and custom items.
- `05_explode` - the mesh slicer: press Left Mouse Button to blow a mesh into
  fragments.
- `06_fruitninja` - a fruit-ninja slicer built from procedural shapes: swipe
  to slice arcing fruit into exploding fragments, dodge bombs.
- `07_orbit` - "Orbit Runner": a surface-dodge game steering a marker around
  a planet, exercising the whole `transform` orbit family.
- `08_dropzone` - a lunar-lander game and the headline demo of the PD
  controller: fly a ship down onto a noise planet with radial gravity.
- `09_reactor` - "Reactor": a rules-as-machine incremental where the whole
  simulation runs on the modding event bus and you build the machine at
  runtime.
- `10_asteroids` - a top-down shooter where shot rocks slice into real avian
  physics bodies that keep drifting as new hazards.
- `11_overload` - "Overload": a dashboard-survival game rendered entirely on
  the status bar; juggle four coupled gauges before the reactor melts down.
- `12_bastion` - "Bastion": a defend-the-core tower defense demoing
  `camera/project` and the aim/track halves of `transform`.
- `13_glide` - "Glide": a slide-merge (2048-style) puzzle rendered entirely in
  Bevy UI, demoing `tween` and `persist` + high scores.
- `14_breach` - "Breach": a grounded, Doom-like first-person arena shooter
  with a hitscan gun (avian `SpatialQuery` raycast).

## More

- [`AGENTS.md`](AGENTS.md) - the deep orientation doc: full module map,
  conventions, the build/verify/run command suite, and hard-won gotchas. Read
  this before contributing.
- [`web/`](web/) - the TypeScript + webpack showcase site that serves the
  examples as WebAssembly builds; see `web/README.md` and
  `docs/wasm-web-builds.md`.
- [`docs/`](docs/) - reference docs, the `docs/LESSONS.md` ledger, and
  `docs/plans/`. Per-task records (spikes, reviews, retros, design notes)
  live next to their task under `tasks/<id>/`.

## License

MIT. See [`LICENSE`](LICENSE).
