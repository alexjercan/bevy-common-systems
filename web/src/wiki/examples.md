# Example games

The numbered examples under `examples/NN_name.rs` are small, complete games that
each headline one or more modules. They are the crate's integration tests *and*
its quickstart documentation: if a module works, one of these games proves it in
motion. Every game past the foundations is [playable in your browser](../../play/).

## Foundations

The first five examples are focused demos rather than full games -- the shortest
possible thing that exercises one module:

- **`01_sphere`** -- an octahedron sphere from [`TriangleMeshBuilder`](../mesh/)
  with a [WASD camera](../camera/).
- **`02_planet`** -- the same mesh displaced with Fbm/Perlin noise: a planet.
- **`03_modding`** -- the [modding](../modding/) event bus end to end, including
  `#[derive(EventKind)]` and JSON-authored handlers.
- **`04_status_item`** -- the [status-bar HUD](../ui/) with FPS and custom items.
- **`05_explode`** -- the [mesh slicer](../mesh/): press the left mouse button to
  blow a mesh into fragments.

## The games

Examples `06` through `14` are the complete, playable games:

- **`06_fruitninja`** -- swipe to slice arcing fruit into exploding fragments,
  dodge bombs.
- **`07_orbit`** -- "Orbit Runner": a surface-dodge game steering a marker around
  a planet, exercising the whole [transform](../transform/) orbit family.
- **`08_dropzone`** -- a lunar-lander game and the headline demo of the
  [PD controller](../physics/): fly a ship down onto a noise planet with radial
  gravity.
- **`09_reactor`** -- "Reactor": a rules-as-machine incremental where the whole
  simulation runs on the [modding](../modding/) event bus and you build the
  machine at runtime.
- **`10_asteroids`** -- a top-down shooter where shot rocks slice into real avian
  [physics](../physics/) bodies that keep drifting as new hazards.
- **`11_overload`** -- "Overload": a dashboard-survival game rendered entirely on
  the [status bar](../ui/); juggle four coupled gauges before the reactor melts
  down.
- **`12_bastion`** -- "Bastion": a defend-the-core tower defense demoing
  [camera/project](../camera/) and the aim/track halves of
  [transform](../transform/).
- **`13_glide`** -- "Glide": a slide-merge (2048-style) puzzle rendered entirely
  in Bevy UI, demoing [tween](../tween/) and [persist](../persist/) + high
  scores.
- **`14_breach`** -- "Breach": a grounded, Doom-like first-person arena shooter
  with a hitscan gun (avian `SpatialQuery` raycast, see [physics](../physics/)).

## Which modules each one demos

| Example | Headlines |
| --- | --- |
| `01_sphere` | [mesh](../mesh/), [camera](../camera/) |
| `02_planet` | [mesh](../mesh/), [meth](../meth/) |
| `03_modding` | [modding](../modding/) |
| `04_status_item` | [ui](../ui/) |
| `05_explode` | [mesh](../mesh/) |
| `06_fruitninja` | [mesh](../mesh/), [feedback](../feedback/), [input](../input/) |
| `07_orbit` | [transform](../transform/), [scoring](../scoring/) |
| `08_dropzone` | [physics](../physics/), [mesh](../mesh/), [camera](../camera/) |
| `09_reactor` | [modding](../modding/), [ui](../ui/) |
| `10_asteroids` | [physics](../physics/), [health](../health/), [mesh](../mesh/) |
| `11_overload` | [ui](../ui/), [time](../time/) |
| `12_bastion` | [camera](../camera/), [transform](../transform/), [health](../health/) |
| `13_glide` | [tween](../tween/), [persist](../persist/), [scoring](../scoring/), [ui](../ui/) |
| `14_breach` | [physics](../physics/), [health](../health/), [camera](../camera/) |

## Running them

Run any example from the repo root:

```sh
cargo run --example 01_sphere
# add the inspector and debug tools:
cargo run --example 01_sphere --features debug
```

To build them as the WebAssembly pages the [showcase](../../play/) serves, see
[Web builds](../web-builds/).
