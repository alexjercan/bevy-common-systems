# 07_orbit: hazard-hit impact feedback (camera shake + damage flash)

- STATUS: OPEN
- PRIORITY: 80
- TAGS: feature,example

Second polish pass on `examples/07_orbit.rs`: make taking a hit *feel* like a
hit. Right now a hazard touch plays `hurt.wav` and blinks the marker, but the
world stays perfectly steady. Add the same impact feedback `06_fruitninja` has
(task 20260703-140237, screen shake + impact) plus a brief damage screen flash,
so hits read viscerally. No new crates, no new assets.

## Behavior

- On a hazard hit, inject "trauma" into a `CameraShake`-style resource that
  decays over time; while it is non-zero the camera position is perturbed by a
  small, decaying random offset. Port fruitninja's `CameraShake` /
  `apply_camera_shake` (trauma add + decay).
- The catch specific to 07: the chase camera *owns* the camera transform (it
  writes it in `ChaseCameraSystems::Sync`). Apply the shake as an additive
  offset in a system ordered `.after(ChaseCameraSystems::Sync)` so it layers on
  top of the chase result instead of being overwritten. Confirm the set name
  from `src/camera/chase.rs`.
- A brief red damage vignette/flash: a full-screen UI node whose alpha spikes on
  hit and fades out, so the hit is unmissable even off-camera.

## Steps

- [ ] Read fruitninja's `CameraShake`, `apply_camera_shake`, and the shake
      constants (`SHAKE_DECAY`, trauma-to-offset curve); reuse the same shapes.
- [ ] Read `src/camera/chase.rs` to confirm the public `ChaseCameraSystems`
      set name and that the shake system can run after it in `PostUpdate`.
- [ ] Add a `CameraShake` resource + a pure trauma/decay helper with
      `#[cfg(test)]` tests (decays to zero, clamps at max), per the testing
      convention.
- [ ] Add trauma on hazard hit in `resolve_collisions`; add an
      `apply_camera_shake` system after the chase-camera sync that offsets
      `MainCamera`'s transform by the decaying shake, and settles it back
      exactly (no drift) when trauma is zero.
- [ ] Add a damage-flash: a `DespawnOnExit(Playing)` full-screen node + a
      component the hit spikes, faded out each frame; keep it out of the way of
      the HUD (behind it, non-interactive).
- [ ] Update the module `//!` doc to mention the impact feedback.
- [ ] Verify: `cargo fmt --check`, `cargo clippy --all-targets`,
      `cargo test --example 07_orbit`, `./scripts/check-ascii.sh`, and a manual
      `cargo run --example 07_orbit`.

## Notes

- Keep the shake subtle -- this is a smooth chase-camera game, not fruitninja's
  fixed camera; an over-large shake fights the `LerpSnap` glide. Tune small.
- Depends on nothing in 20260703-214926 functionally, but both edit the orb/
  hazard collision path and the `//!` doc; sequence after it to avoid a
  needless conflict (this flow runs them one at a time on the same branch).
