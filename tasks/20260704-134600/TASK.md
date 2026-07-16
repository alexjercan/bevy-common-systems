# feedback: hit-flash material flash module (Wave 1)

- STATUS: CLOSED
- PRIORITY: 36
- TAGS: spike,feature,feedback

> Spike: tasks/20260704-134035/SPIKE.md (read
> first). Wave 1 -- promote recurring example juice into the library.

## Goal

Add a `feedback` module (new top-level concern) for the hit/damage material
flash that three games (06, 07, 10) hand-roll: a `Flash { color, duration }`
component that briefly overrides an entity's material emissive / base color and
eases it back to the original.

The design problem to solve cleanly (spike open question) is restoring the
material without leaking handles when the base material is shared -- e.g. snap
the original values on `On<Add, Flash>` into a private `*State` and restore on
completion, or spawn a per-entity material clone. Follow the Config / State
convention and observers for setup. Prove it by refactoring one example
(10_asteroids) onto it. Pairs with the `tween` module for the decay curve.

## Premise correction (planning)

The spike table counts 06/07/10 as hand-rolling a "material flash", but reading
the code they do NOT: 06/07/10 each hand-roll a full-screen red UI *overlay*
(`RedFlash`/`DamageFlash` + `BackgroundColor`), and 07/10 additionally blink an
entity's `Visibility` during i-frames. No game flashes a material's emissive /
base color at runtime. So there is no local material-flash copy to delete.

Per a user decision (2026-07-04), we still build the material `Flash` module the
spike explicitly specs (it is the interesting design -- restoring a shared
material without leaking handles), and DEMONSTRATE it by *adding* a hit-flash to
10_asteroids rather than refactoring an existing copy. The real duplication (the
full-screen overlay) is split out to a separate follow-up task.

## Design decisions

- **Per-entity material clone, not in-place mutation.** On `On<Add, Flash>`,
  clone the entity's `StandardMaterial` into a fresh asset, swap the entity's
  `MeshMaterial3d` to the clone, and store the original handle + the clone
  handle in a private `FlashState`. This is the only correct answer for a
  *shared* material: mutating the shared asset in place would flash every entity
  that shares it. The flash animates the clone; the shared original is untouched.
- **Leak-free via `On<Remove, FlashState>`.** A remove observer frees the clone
  asset whenever `FlashState` is removed -- both on normal completion (the
  animate system swaps the original handle back and removes `Flash`/`FlashState`)
  AND on entity despawn (despawn removes the components -> observer fires). So a
  flashed entity that dies mid-flash does not leak its clone.
- **Channel: emissive or base color.** `Flash { color, duration, channel }` with
  `FlashChannel { Emissive (default), BaseColor }`. Emissive is the default (the
  juicy, bloom-friendly hit look; 10 runs `camera/post` bloom).
- **Demo target = the ship, not a rock.** A rock despawns/splits the instant a
  bullet hits it (no persistence), so flashing it would never render. The ship
  survives a hull hit (i-frames), so flash its `ShipModel` hull emissive red on
  the hit. The shared-material isolation is proven by an ECS test instead (two
  entities sharing one material; flash one; assert the other is untouched).
- **tween**: the ease-back is a linear lerp for now; back it with the `tween`
  module (task 20260704-134630) once it lands.

## Steps

- [ ] Add `src/feedback/` (`mod.rs` + `flash.rs`): module docs, `Flash` config,
      `FlashChannel`, private `FlashState`, `FlashSystems`, `FlashPlugin`, the
      `On<Add,Flash>` clone observer, the `On<Remove,FlashState>` free observer,
      and the `animate_flash` system. Factor the color ease into a pure
      `flash_mix(original, flash, k)` fn.
- [ ] Tests: pure `flash_mix` test + an ECS test that shares one material across
      two entities, flashes one, and asserts (a) the flashed entity gets a
      distinct clone whose emissive moved toward the flash color, (b) the other
      entity's material is untouched, (c) after the duration the original handle
      is restored and the clone asset is freed (no leak).
- [ ] Wire preludes: `pub mod feedback;` + `feedback::prelude::*` in `src/lib.rs`.
- [ ] Demo in `10_asteroids`: on a ship hit, insert `Flash` on the `ShipModel`
      so the hull flashes red and eases back. Keep the existing blink/overlay.
- [ ] Add a follow-up task: promote the full-screen damage OVERLAY that 06/07/10
      actually duplicate into `feedback` (screen flash), deleting the 3 copies.
- [ ] Verify: fmt, clippy (both configs), test, test --examples, check-ascii,
      boot 10_asteroids.
