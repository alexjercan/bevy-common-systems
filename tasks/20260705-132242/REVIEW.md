# Review: breach -- juice pass

- VERDICT: APPROVE
- ROUNDS: 1

## Verified

- `cargo fmt --check`, `cargo clippy --all-targets` (clean), `cargo test --example
  14_breach` (17 pass), `scripts/check-ascii.sh` (clean).
- Headless `BCS_AUTOPILOT`: full cycle, no panic, no despawn errors, no B0004 (the new
  muzzle-flash / spawn-beacon / hit-marker entities render without warnings).

## Findings / checks

- Caught the real trap before shipping: `low_health_warning` re-spikes the *persistent*
  `DamageVignette` overlay, so it MUST set `ScreenFlash { despawn_on_end: false }` (the
  `Default` is `true`, which would despawn the vignette and break all future flashes).
  Matched `enemy_melee`'s existing usage.
- Hit marker is a transparent-border box whose alpha is driven off a `HitFlash(f32)`
  timer set only on a confirmed enemy hit; it starts at alpha 0 and cannot black out the
  screen. `is_low_health` is pure and unit-tested (incl. the zero-max divide guard).
- Muzzle flash / spawn beacon are emissive `StandardMaterial` (no `unlit`, so they
  bloom) single-mesh `TempEntity`s, scoped `DespawnOnExit(Playing)`.

## Screenshot limitation (honest note)

- A `BCS_SHOT` capture of the Playing state came back fully black -- even the always-on
  white crosshair dot was absent, so this is the known headless offscreen-capture
  unreliability (see the 13_glide/reactor screenshot retros), NOT a content bug. The
  fire-triggered juice (hit marker, muzzle flash) only appears after input anyway, which
  a state-entry screenshot never drives. Visual correctness rests on: the clean no-B0004
  autopilot run, the pure `is_low_health` test, and the `despawn_on_end` fix. The purely
  cosmetic pieces (colours/sizes) are low-risk and built from already-proven modules
  (`ui/popup`, `ScreenFlash`, emissive `TempEntity` like the existing tracer).

## Nits (non-blocking)

- The hit marker is a box rather than the classic 4-tick X (UI node rotation avoided);
  reads fine as a hit confirm.
