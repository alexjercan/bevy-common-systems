# Fruit ninja: screen shake and bomb impact feedback

- STATUS: OPEN
- PRIORITY: 100
- TAGS: feature,example

## Goal

Add impact feedback to `examples/06_fruitninja.rs`: a small camera shake on
every slice, and a bigger shake plus a red screen flash and a short beat before
the game-over screen when a bomb is sliced. Cuts feel punchy and losing lands
instead of blinking away.

## Steps

- [ ] Add a `MainCamera` marker to the camera spawn (:302) and a
      `CameraShake { trauma: f32 }` resource (`init_resource`); keep the base
      translation as a const (camera is at (0, 0, 22)).
- [ ] Add `apply_camera_shake` (Update, `Playing`): decay `trauma` toward 0,
      offset the camera translation from base by a small random amount scaled by
      `trauma * trauma`, and snap back to base when trauma ~0.
- [ ] In `slice_objects`, bump `trauma` a little on a fruit slice and more on a
      bomb slice.
- [ ] Bomb beat: rework `on_player_died` (:356) to not transition immediately;
      spawn a full-screen red flash `Node` (fading alpha) and start a
      `DyingTimer` resource; a system fires `GameState::GameOver` when it
      elapses (~0.3s). Decide and note whether the `Escape` give-up also uses
      the beat or stays instant.
- [ ] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, and a real boot (auto-slice a bomb to
      confirm the flash/beat and no panic).

## Notes

- Camera spawn: `examples/06_fruitninja.rs:302`. `on_player_died` at :356.
- `rand::rng()` is fine in the example binary for the shake offset (unlike
  workflow scripts). Vary the offset each frame.
- Red flash node: `DespawnOnExit(Playing)` plus a self-despawn on full fade so
  it does not linger.
- No new dependencies.
