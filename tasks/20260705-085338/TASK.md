# Bastion: juice - build particles, explode-on-destroy, extra shake

- STATUS: OPEN
- PRIORITY: 75
- TAGS: feature,example,bastion

> Part of the 12_bastion polish goal. The user asked for "juice": particles when
> a tower is built, an explosion when things are destroyed, and (more) screen
> shake. The example already explodes killed enemies into fragments and shakes on
> kill/core-hit/death; this task adds the build burst, makes the Core explode on
> destruction, and adds shake to the build/wave beats so the game feels punchier.

## Goal

Add "juice" to `examples/12_bastion.rs`: a spark/particle burst when a tower is
placed (and on upgrade), the Core physically exploding into fragments when it
dies, and camera-shake kicks on building and on each new wave. Reuse the crate's
existing kit (`TempEntity`, `glowing_material`, `ExplodeMesh`/`ExplodeFragments`,
`CameraShakeInput`) - do not add new dependencies.

## Design notes (from exploration)

- Spark burst idiom: `TempEntity(ttl)` + `Mesh3d(spark_mesh)` +
  `MeshMaterial3d(glowing_material(...))`, spawned N times in a `rand` loop each
  pushed outward along a random unit direction (bastion already caches
  `spark_mesh`/`spark_material` in `GameAssets` and pulls `rand::Rng`). Add a
  small per-particle velocity + a system to move+fade them, OR reuse the
  fruitninja `FragmentMotion` gravity idiom (`06_fruitninja.rs:1226-1238`). Keep
  it cheap (a handful of particles).
- Core explode: the Core is a mesh entity; on death insert
  `ExplodeMesh { fragment_count }` and let the existing
  `on_fragments_spawned` observer (`12_bastion.rs:1570`) turn shards into debris
  - but the Core is persistent across runs (revived in `start_game` by resetting
  Health + removing `HealthZeroMarker`). Do NOT despawn the Core; explode a
  short-lived COPY/effect at the Core's position instead, or hide the Core mesh
  during the death beat and restore it on `start_game`. Check `on_fragments_spawned`
  despawns its target - the Core must survive to be revived. Reconcile carefully.

## Steps

- [ ] Add a `spawn_spark_burst(commands, assets, origin, count, color/speed)`
      helper that emits `count` glowing `TempEntity` particles flying outward
      from `origin`. Give particles a `Spark { velocity }` component + a
      `move_sparks` system (advance by velocity*dt, optional gravity/drag),
      running in Playing. Keep the pure direction math trivial or reuse rand.
- [ ] Call the burst on tower placement (in `place_or_select` / `spawn_tower`)
      and on upgrade (`upgrade_selected` / the shared upgrade helper), tinted by
      the tower colour. Confirms the "particles on build" ask.
- [ ] Make the Core explode on destruction in `on_core_died`
      (`12_bastion.rs:1701`): trigger a fragment explosion effect at the Core's
      position WITHOUT breaking the revive path. Preferred: spawn a transient
      copy of the Core mesh with `ExplodeMesh` (so `on_fragments_spawned` shards
      it and despawns the copy), and hide the real Core mesh (Visibility::Hidden)
      for the death beat, restoring it in `start_game`. Verify a second run
      still shows a full Core (the retro flagged Core-revive as the top risk).
- [ ] Add camera-shake kicks (`CameraShakeInput::add_trauma`) on tower build (a
      small `SHAKE_BUILD`) and on each new wave (`SHAKE_WAVE`), with new named
      constants alongside the existing `SHAKE_*`. Coordinate with the packs task
      (whichever lands second owns the wave-start shake; avoid double-adding).
- [ ] Optional extra polish if cheap: a brief flash (`feedback::flash`) on the
      built tower, and slightly scale the death-beat shake. Keep scope tight.
- [ ] Update the module `//!` "juice kit" paragraph to mention build particles
      and the Core explosion.

## Verification

- `cargo clippy --all-targets` clean; `cargo fmt --check`;
  `./scripts/check-ascii.sh`; `cargo test --examples` still green.
- Headless autopilot run reaches Playing and completes with no panic; if
  `$DISPLAY` is set, boot under `timeout` and (per the visual-layer rule) grab a
  `scrot`/`ScreenshotPlugin` frame showing particles on a fresh build and confirm
  the Core is intact on a second run (autopilot cycles Menu->Playing again).
- Emissive particles must bloom: use `glowing_material` (HDR emissive), NOT
  `unlit: true` (the documented no-bloom footgun).
