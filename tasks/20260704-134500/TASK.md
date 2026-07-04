# camera/shake: CameraShake trauma module (Wave 1)

- STATUS: CLOSED
- PRIORITY: 40
- TAGS: spike,feature,camera

> Spike: docs/spikes/20260704-134035-game-juice-and-scaffolding-kit.md (read
> first). Wave 1 -- promote recurring example juice into the library.

## Goal

Add a `camera/shake` module beside `camera/chase` and `camera/post` that owns
the camera-shake code four example games (06, 07, 08, 10) currently hand-roll,
including the accumulate-vs-absolute drift bug the asteroids retro recorded
(`docs/retros/20260703-170744-asteroids-example.md`).

Follow the crate convention: a `CameraShake` config component (trauma decay
rate, max offset/kick), a way to add trauma each frame (an `*Input` component
or a small `Commands` extension / event), and an `*Output` that writes the
offset as `BASE + trauma^2 * random` -- never an accumulating `+=`. Must
compose with `camera/chase`: shake is applied *after* the chase camera writes
the transform, so give it an ordering `*Systems` set. Prove it by refactoring
at least one example (06_fruitninja) onto the module and deleting its local
copy. Decide during planning whether v1 shakes rotation (kick) or translation
only (spike open question).

## Design decisions (planning)

- **Rotation kick: support both, default to translation-only.** `CameraShake`
  carries both `max_offset: Vec3` and `max_kick: Vec3`; `max_kick` defaults to
  `Vec3::ZERO`, so out of the box v1 shakes translation only (matches all four
  current example copies), but a game can opt into a rotational kick. Resolves
  the spike open question without closing the door on kick.
- **Restore/Apply two-phase, driver-agnostic.** The drift bug is caused by
  `+=` on a camera whose base is *not* refreshed each frame. To compose with
  *any* base driver (chase, a fit system, or nothing) without drift, the module
  runs two systems: `Restore` un-applies the previous frame's offset/kick
  (before drivers run), and `Apply` decays trauma and re-applies a fresh
  offset/kick (after drivers run). This yields `driver_base + offset` every
  frame -- never an accumulator -- for both the static (06) and the chase
  (07/08) case. Exposed as a `CameraShakeSystems { Restore, Apply }` set;
  `Apply` is ordered `.after(ChaseCameraSystems::Sync)` and `Restore`
  `.before` it, so chase composition is automatic and safe even when the chase
  plugin is absent (ordering against an empty set is a no-op).
- **Component split.** Config `CameraShake { decay, max_offset, max_kick,
  exponent }`; `*Input` `CameraShakeInput { add_trauma, reset }` the game
  writes each frame (drained to zero after consumption); `*Output`
  `CameraShakeOutput { offset, kick }` the game can read; private
  `CameraShakeState { trauma, last_offset, last_kick }`. Companions attached
  via `#[require(...)]`. A `Commands`-style trigger is out of scope for v1; the
  Input component matches the `transform/*` driver convention.

## Steps

- [x] Add `src/camera/shake.rs`: module doc with usage snippet, the four
      components, `CameraShakeSystems`, `CameraShakePlugin`, and the
      Restore/Apply systems. Factor the pure math (`decay_trauma`,
      trauma->amount via `exponent`, offset/kick from a sampled unit vector)
      into small functions.
- [x] `#[cfg(test)]` unit tests for the pure math: decay clamps at 0, trauma
      add clamps at 1, amount = trauma^exponent, zero trauma -> zero offset,
      offset scales with `max_offset`.
- [x] Wire preludes: `pub mod shake;` + `shake::prelude::*` in
      `src/camera/mod.rs`; confirm `crate::prelude` re-exports it.
- [x] Refactor `examples/06_fruitninja.rs` onto the module: delete the local
      `CameraShake` resource, `apply_camera_shake`, and the `SHAKE_*`/base
      consts that move into the component; add `CameraShakePlugin`, put
      `CameraShake` on the camera, and route slice/bomb trauma through
      `CameraShakeInput` (reset on new game).
- [x] Verify: `cargo fmt`, `cargo clippy --all-targets`, `cargo test`,
      `cargo test --examples`, `scripts/check-ascii.sh`, and boot
      `06_fruitninja` under `timeout` to confirm it reaches the render loop.
