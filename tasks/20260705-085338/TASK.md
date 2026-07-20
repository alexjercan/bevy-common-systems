# Bastion: juice - build particles, explode-on-destroy, extra shake

- STATUS: CLOSED
- PRIORITY: 75
- TAGS: feature,example,bastion,historical

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

- [x] Add a `spawn_spark_burst(commands, assets, origin, count, color/speed)`
      helper that emits `count` glowing `TempEntity` particles flying outward
      from `origin`. Give particles a `Spark { velocity }` component + a
      `move_sparks` system (advance by velocity*dt, optional gravity/drag),
      running in Playing. Keep the pure direction math trivial or reuse rand.
- [x] Call the burst on tower placement (in `place_or_select` / `spawn_tower`)
      and on upgrade (`upgrade_selected` / the shared upgrade helper), tinted by
      the tower colour. Confirms the "particles on build" ask.
- [x] Make the Core explode on destruction in `on_core_died`
      (`12_bastion.rs:1701`): trigger a fragment explosion effect at the Core's
      position WITHOUT breaking the revive path. Preferred: spawn a transient
      copy of the Core mesh with `ExplodeMesh` (so `on_fragments_spawned` shards
      it and despawns the copy), and hide the real Core mesh (Visibility::Hidden)
      for the death beat, restoring it in `start_game`. Verify a second run
      still shows a full Core (the retro flagged Core-revive as the top risk).
- [x] Add camera-shake kicks (`CameraShakeInput::add_trauma`) on tower build (a
      small `SHAKE_BUILD`) and on each new wave (`SHAKE_WAVE`), with new named
      constants alongside the existing `SHAKE_*`. Coordinate with the packs task
      (whichever lands second owns the wave-start shake; avoid double-adding).
- [x] Optional extra polish if cheap: a brief flash (`feedback::flash`) on the
      built tower, and slightly scale the death-beat shake. Keep scope tight.
- [x] Update the module `//!` "juice kit" paragraph to mention build particles
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

## Close-out

Done. Added juice to 12_bastion, reusing the existing kit (no new deps):

- Spark bursts: a `Spark { velocity }` component + `spawn_spark_burst` helper
  (glowing `TempEntity` particles fountaining up+out, random dir/speed/scale) and
  a `move_sparks` system that flies them under `SPARK_GRAVITY` + `SPARK_DRAG`.
  Fired on tower placement (tower-tinted HDR-emissive burst, `BUILD_SPARKS`) and
  on upgrade (gold, `UPGRADE_SPARKS`, via the shared `try_upgrade_selected`).
- Core explode-on-destroy: `on_core_died` now spawns a short-lived COPY of the
  Core mesh at its pose with `ExplodeMesh { CORE_EXPLODE_FRAGMENTS }` (the global
  `on_fragments_spawned` observer shards it into debris and despawns the copy) and
  hides the real, persistent Core; `start_game` restores `Visibility::Visible` so
  the next run shows a full Core. This preserves the revive path (the retro's
  top-flagged risk) rather than despawning the Core.
- Extra screen shake: `SHAKE_BUILD` on tower placement (in `place_or_select`) and
  `SHAKE_WAVE` on each new wave (in `advance_waves`, replacing the packs-task
  NOTE placeholder). Kill/core-hit/death shakes unchanged.

Verification:
- New deterministic integration test `start_game_revives_hidden_core` asserts the
  highest-risk path: a hidden, zero-health, marked Core is revived by `start_game`
  to Visible + full health + no marker (the follow-up retro's "verify the effect,
  not a proxy" lesson applied to the exact revive concern the prior cycle flagged).
- Existing `build_bar_tap_arms_tower_and_upgrades` updated for the new
  `(&mut Tower, &Transform)` query + GameAssets dependency and still passes.
- Observable-effect live check: a temporary log confirmed build fires spark
  bursts (14 sparks per placement at the tower position); temp log removed. The
  autopilot run completed the full Menu->Playing->GameOver cycle with "no panic",
  exercising sparks + move_sparks + wave/build shake.
- Emissive particles use `glowing_material` (HDR emissive), not `unlit: true`, so
  they bloom under `camera/post` (the documented no-bloom footgun).
- Checks: plain `cargo build --example` clean (no dead-code), `clippy
  --all-targets` clean, `fmt --check` clean, `check-ascii` clean,
  `cargo test --examples` all green.

Deliberately skipped the optional per-tower `feedback::flash` (the spark burst +
shake already read as punchy; kept scope tight).
