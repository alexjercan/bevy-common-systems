# Promoting nova's integrity + destructible systems

Date: 2026-07-08

## What and why

Nova Protocol ran a spike
(`nova-protocol/docs/spikes/20260708-110317-promotion-eligible-systems.md`) to find the
game-agnostic destruction/health building blocks worth lifting into this crate. This change
promotes the Tier A + Tier B candidates from that spike:

- `integrity/` (new module, Tier B): the destruction pipeline over a graph of connected,
  health-bearing nodes.
  - `components` - `IntegrityRoot`, `ConnectedTo` (node-local neighbour list),
    `IntegrityLeafMarker`, `IntegrityDisabledMarker`, `IntegrityDestroyMarker`.
  - `blast` - `blast_damage()` radial sensor bundle + `BlastDamageConfig` (linear falloff).
  - `plugin` - `IntegrityPlugin`: impact damage (impulse/energy from relative velocity and
    mass), blast falloff damage, health-depletion -> disabled -> destroy, leaf derivation,
    and the prune-and-cascade chain reaction.
- `physics/rigid_body` (Tier A): `rigid_body_point_velocity` (the
  `v = v_lin + omega x (p - com)` muzzle-velocity formula) and `destructible_body(health,
  density)` (the Health + density + Visibility bundle).
- `ui/health_display` (Tier A): a "Health: N%" text readout over a target's `Health`.
- `ui/objectives` (Tier A): a generic id+message objectives list driven by the
  `GameObjectives` resource.

## The seam (Tier B)

The integrity core is game-agnostic because two responsibilities stay with the game:

1. Building the graph. The game decides how nodes connect by inserting `ConnectedTo` and
   marking the owning body `IntegrityRoot`. Nova builds this from its ship section grid; the
   `15_integrity` example builds it from a rectangular grid. The core never assumes a layout.
2. Reacting to destruction. The pipeline's only output is the `IntegrityDestroyMarker`
   component. A game observes `On<Add, IntegrityDestroyMarker>` to explode, spawn debris,
   fire an event, or despawn. The core inserts the marker and prunes the graph; it never
   decides what "destroyed" looks like.

So no new "destroyed" event type was needed - the marker *is* the seam, and it already was
in the nova design. That is why nova's `integrity/glue.rs` (section-grid adjacency, section
disable, ship-health rollup) and `integrity/explode.rs` (mesh-slice + debris +
`OnDestroyedEvent`) stay in nova: they are the game's two ends of the seam, not part of the
core. The mesh-slice half is instead demonstrated here by hooking the marker to the existing
`ExplodeMeshPlugin` in the example.

`destructible_body` was promoted without nova's `ExplodableEntity` marker (which belongs to
nova's explode integration): here it is just the physics/health half of a destructible body,
and the destruction behaviour is wired separately via the integrity seam.

## Tests and example

The full nova test suites came across intact: the avian-free core unit tests (leaf
derivation, disable/destroy transitions, the full damage -> destruction sequence, the
chain reaction) and the real-avian `physics_tests` (impact damage from computed mass, the
velocity gate, blast falloff, the blast-owns-its-events regression, single-hit dedup,
out-of-range). `examples/15_integrity` is the interactive demo and integration check: a grid
wall you blast a hole in and watch collapse, with the destroy seam feeding the mesh slicer,
plus the health display (aggregate health summed onto a core entity) and an objectives panel.

## Follow-up

The cross-repo move on the nova side - depend on these promoted symbols and delete nova's
local `hud/health`, `hud/objectives`, `integrity/blast`, `integrity/components`, and the
`game_object` helpers - is tracked by nova task `20260706-151804`. Nova keeps `glue`,
`explode`, and the section/HUD orchestration.
