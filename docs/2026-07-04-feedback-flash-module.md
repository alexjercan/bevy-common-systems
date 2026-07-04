# feedback: material hit-flash module

- DATE: 2026-07-04
- TASK: tasks/20260704-134600
- SPIKE: docs/spikes/20260704-134035-game-juice-and-scaffolding-kit.md (Wave 1)

## What changed

Added a new top-level `feedback` concern (`src/feedback/`, starting with
`flash`): `FlashPlugin` gives an entity a short material "hit flash" -- override
a `StandardMaterial` channel (emissive by default, or base color) with a flash
color and ease it back over a duration. Demonstrated in `10_asteroids`: the
ship's hull flashes red when it takes a hull hit.

## Premise correction

The spike table credited 06/07/10 with hand-rolling a "material flash". Reading
the code, that is not what they do: 06/07/10 each hand-roll a full-screen red UI
*overlay* (`RedFlash` / `DamageFlash` + `BackgroundColor`), and 07/10 also blink
an entity's `Visibility` during i-frames. No game flashes a material at runtime.
So the task's "refactor one example, deleting its local copy" could not be done
literally -- there was no material-flash copy to delete.

Per a user decision, we still built the material `Flash` module the spike
explicitly specs (it is a genuinely useful, classic juice primitive and the
material-restore problem is the interesting design), and *demonstrated* it by
adding a hit-flash to 10 rather than refactoring an existing copy. The real
duplication -- the full-screen overlay -- is split into a follow-up
(tasks/20260704-155505) to be promoted as a separate `feedback/screen_flash`.

## Key decisions

### Per-entity material clone, not in-place mutation

The design crux (spike open question) is flashing without corrupting a *shared*
material. Entities usually share one material handle (10's rocks all share
`rock_material`), so mutating that asset in place would flash every sharer. On
`On<Add, Flash>` the plugin instead **clones** the entity's material into a
fresh per-entity asset, swaps the entity's `MeshMaterial3d` to the clone, and
records the original + clone handles in a private `FlashState`. The animation
mutates only the clone; the shared original is never touched. An ECS test proves
this: two entities share one material, one is flashed, and the bystander's
material is asserted unchanged.

### Leak-free via `On<Remove, FlashState>`

Cloning risks leaking the clone asset. A remove observer frees the clone
whenever `FlashState` leaves the entity, which covers both paths:

- normal completion -- `animate_flash` swaps the original handle back and
  removes `Flash`/`FlashState`, firing the observer;
- despawn mid-flash -- the despawn removes the components, firing the observer.

So a flashed entity that dies before the flash finishes (common: a hit enemy
that also explodes) does not leak. Both paths are covered by tests
(`flash_restores_original_and_frees_clone_when_done`, `despawn_mid_flash_frees_clone`).

### Demo target = the ship, not a rock

The obvious target -- flash a rock white when a bullet hits it -- does not work:
in 10 a hit rock despawns/splits the same frame, so the flash would never
render (and would only exercise the despawn-cleanup path). The ship *survives* a
hull hit (i-frames), so the demo flashes the `ShipModel` hull emissive red on the
hit, layered over the existing i-frame visibility blink. The shared-material
isolation that a rock would have shown is covered by the ECS test instead.

### Channel: emissive default

`Flash { color, duration, channel }` with `FlashChannel { Emissive (default),
BaseColor }`. Emissive is the default: it reads as a glow and blooms under
`camera/post` (10 runs bloom), and it is the classic hit-flash look. Base color
is there for unlit / 2D-ish materials.

## Testing

- Pure `flash_mix(original, flash, k)` unit tests (endpoints, midpoint, clamp,
  alpha preserved).
- Three ECS tests (minimal `App` + `AssetPlugin` + `Time` by hand): shared
  material stays untouched while a clone is flashed; original restored + clone
  freed on completion; clone freed on despawn mid-flash.
- `10_asteroids` boots to the render loop.

## Follow-ups

- tasks/20260704-155505: promote the full-screen damage overlay 06/07/10 truly
  duplicate into `feedback/screen_flash` (the real dedup).
- Back the ease-back with the `tween` module (task 20260704-134630) once it lands.
