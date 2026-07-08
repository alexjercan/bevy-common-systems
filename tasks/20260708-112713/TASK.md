# Promote nova integrity + destructible systems into bevy_common_systems

- STATUS: CLOSED
- PRIORITY: 70
- TAGS: crates, feature, integrity

Promote the Tier A + Tier B promotion candidates from the nova-protocol spike
(`nova-protocol/docs/spikes/20260708-110317-promotion-eligible-systems.md`) into this
crate. These are the game-agnostic destruction/health building blocks that any game with
destructible physics bodies can reuse.

## Steps

- [ ] New `integrity` module (Tier B core): `components` (IntegrityRoot, ConnectedTo,
      leaf/disabled/destroy markers), `blast` (radial blast sensor + falloff), `plugin`
      (IntegrityPlugin: impact + blast collision damage, health-depletion -> disabled ->
      destroy, leaf derivation, chain reaction). `IntegrityDestroyMarker` is the generic
      "destroyed" seam games observe. Nova's section `glue` and `explode` stay in nova.
- [ ] Physics helpers (Tier A): `rigid_body_point_velocity` (pure formula) and
      `destructible_body` bundle in `physics/rigid_body`.
- [ ] UI (Tier A): `ui/health_display` (text % HUD over Health) and `ui/objectives`
      (generic id+message list).
- [ ] Port the full existing test suites (avian-free core unit tests + real-avian
      physics tests) and keep them green.
- [ ] Integration example `examples/NN_integrity.rs` that wires the whole thing:
      damage/blast -> destruction -> chain reaction, with the destroy seam hooked to the
      existing `ExplodeMeshPlugin`, plus health display + objectives.
- [ ] Verify: cargo fmt, clippy (both configs), test (both configs), test --examples,
      check-ascii. Do not merge - leave for review.

## Notes

Spike: nova-protocol/docs/spikes/20260708-110317-promotion-eligible-systems.md
Downstream cross-repo move + local deletion in nova is tracked by nova task 20260706-151804.
