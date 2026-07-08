# Promoting the direction shader materials

Date: 2026-07-08

## What and why

The last Tier-C candidate from the nova-protocol promotion spike
(`nova-protocol/docs/spikes/20260708-110317-promotion-eligible-systems.md`, nova task
`20260706-151804`): the two `ExtendedMaterial<StandardMaterial, _>` shader materials behind
nova's velocity HUD are game-agnostic "visualize a direction+magnitude vector" building
blocks, so they move here.

Added as `material/direction`:

- `DirectionMagnitudeMaterial` - displaces a mesh's vertices along local +Y by a driven
  `magnitude_input`, peaking at the local origin and falling off past `radius`, clamped to
  `max_height`. A cone becomes a needle whose length tracks a magnitude.
- `DirectionSphereMaterial` - brightens the fragments whose normal faces the mesh's local -Z,
  raised to `sharpness`. A sphere becomes a soft highlight pointing along the object's forward
  axis.
- `DirectionMaterialsPlugin` - registers both `MaterialPlugin`s and embeds the shaders.

`material.rs` became `material/` (mod + `direction`), keeping `glowing_material`.

## Embedding the shaders (the open question from the spike)

Nova loaded these shaders from `assets/shaders/directional_*.wgsl` by path. A library can't
assume the consumer ships those files, so the wgsl now lives in `src/material/shaders/` and is
compiled into the binary with `bevy::asset::embedded_asset!`. The `MaterialExtension` shader
refs point at the embedded path
(`embedded://bevy_common_systems/material/shaders/directional_magnitude.wgsl`). This is the
crate's first use of `embedded_asset!`; the pattern is: call it in the plugin's `build` (once
per shader), and reference `embedded://<crate>/<dir-under-src>/<path>`. Consumers add
`DirectionMaterialsPlugin` and ship no wgsl.

The WebGL2 16-byte-alignment padding fields (`#[cfg(target_arch = "wasm32")]` + the shader's
`#ifdef SIXTEEN_BYTE_ALIGNMENT`) carried over unchanged, so the materials keep working on the
wasm/web builds.

## What stayed in nova

Only the materials + shaders promoted. Nova keeps the velocity HUD orchestration
(`hud/velocity`): reading the target's avian `LinearVelocity`, driving the
`DirectionalSphereOrbit` placement, feeding `magnitude_input`, and the spaceship-HUD lifecycle.
That is game-specific glue over these generic materials (and the already-promoted
`transform/directional_sphere_orbit`).

## Example

`examples/16_direction` is the demo and compile check: a spinning direction vector with a
pulsing magnitude, shown by a highlight sphere (`DirectionSphereMaterial`) and a magnitude
needle (`DirectionMagnitudeMaterial`).

## Follow-up

The nova side (depend on these materials, delete the local `DirectionMagnitudeMaterial` /
`DirectionSphereMaterial` and `assets/shaders/directional_*.wgsl`, keep `hud/velocity`
orchestration) is the remaining half of nova task `20260706-151804`, done once this is merged.
