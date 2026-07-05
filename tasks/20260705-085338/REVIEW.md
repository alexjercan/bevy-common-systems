# Review: Bastion juice - build particles, Core explosion, extra shake

- TASK: 20260705-085338
- BRANCH: feature/bastion-juice

## Round 1

- VERDICT: APPROVE

Reviewed `git diff master...HEAD` (~263 lines added to `examples/12_bastion.rs`).
Delivers all of the Goal (build/upgrade sparks, Core explode-on-death, build +
wave shake) and is well-tested.

Correctness (verified by reading + running):

- The Core explode preserves the persistent-Core revive path (the prior retro's
  top-flagged risk): `on_core_died` explodes a short-lived COPY of the Core mesh
  and only HIDES the real Core; `start_game` restores `Visibility::Visible` +
  refills health + clears the marker. The `get_mut` on the `With<Core>` query is
  the right identity gate (a stray `HealthZeroMarker` elsewhere is ignored), and
  the `dying`/`state` re-entry guards still run before any side effect, so the
  explosion cannot double-fire. The debris copy carries `DespawnOnExit(Playing)`
  and is despawned by the global `on_fragments_spawned` observer, so no leak.
- Spark particles are self-limiting: `TempEntity` life + `DespawnOnExit(Playing)`,
  and `move_sparks`' drag factor is clamped `.max(0.0)` so a large `dt` cannot
  flip the velocity sign. Emissive material is HDR `glowing_material` (not
  `unlit`), so it blooms under `camera/post` (the documented footgun avoided).
- Shake wiring is correct: `SHAKE_BUILD` on placement, `SHAKE_WAVE` on new waves
  (filling the packs-task deferral); the extra `&mut CameraShakeInput` /
  `With<MainCamera>` queries in one system access disjoint components, so no
  aliasing.

Tests are meaningful:

- `start_game_revives_hidden_core` deterministically drives `start_game` on a
  hidden/zero-health/marked Core and asserts it comes back Visible + full health +
  unmarked -- exactly the revive concern the last cycle flagged, now regression-
  guarded ("verify the effect, not a proxy").
- The build-bar integration test was correctly updated for the new
  `(&mut Tower, &Transform)` query + `GameAssets` dependency and still passes.
- Live: a temp log confirmed build fires 14-spark bursts at tower positions; the
  autopilot completed the full cycle with no panic.

Checks re-run: plain `cargo build --example` clean (no dead-code),
`clippy --all-targets` clean, `fmt --check` clean, `check-ascii` clean,
`cargo test --examples` green.

Findings:

- [x] R1.1 (NIT) examples/12_bastion.rs:1601 - the build burst allocates a fresh
  `StandardMaterial` via `materials.add(...)` on every placement; these are never
  freed, so a long game accumulates them. It matches the existing per-tower
  pattern (`spawn_tower` already leaks a body + turret material per placement), so
  it is consistent and negligible for an example, but if tidying: cache one
  tinted burst material per tower spec in `GameAssets` (mirroring
  `enemy_materials`) and reuse it. Optional.
  - Response: Done -- added `GameAssets::tower_spark_materials` (one glowing
    material per tower spec, built in `setup` like `enemy_materials`) and the
    build burst now reuses `tower_spark_materials[spec_idx]` instead of allocating
    a material per placement.
