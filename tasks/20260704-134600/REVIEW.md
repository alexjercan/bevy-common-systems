# Review: feedback material hit-flash module

- TASK: 20260704-134600
- BRANCH: feat/feedback-flash

## Round 1

- VERDICT: APPROVE (with MINOR findings addressed at implementer discretion)

Independent review traced the leak-free claim across all three paths (normal
completion, despawn mid-flash, re-flash) and confirmed no leak; verified the
animate borrows, the demo (ShipModel is a single lit-emissive entity, query
safe, no borrow conflict, respects the unlit gotcha), test quality, and
conventions. No BLOCKER/MAJOR.

- [x] R1.1 (MINOR) src/feedback/flash.rs - a re-flash on an already-flashing
  entity does not restart: `On<Add>` does not refire, so `elapsed`/clone are
  stale and the second hit produces a truncated flash. Untested. Observe
  `On<Insert, Flash>` and reset `elapsed` (re-pop) when `FlashState` exists; add
  a re-flash test.
  - Response: Switched the setup observer from `On<Add>` to `On<Insert>`; when
    `FlashState` already exists it now resets `elapsed = 0.0` (re-pops with the
    existing clone). Added `reflashing_restarts_the_animation`.
- [x] R1.2 (MINOR) src/feedback/flash.rs:114 - `FlashState` is not
  `register_type`'d, unlike `CameraShakeState`/`PopupState`. Add it.
  - Response: Added `.register_type::<FlashState>()`.
- [x] R1.3 (MINOR) src/feedback/flash.rs:162-171 - a `Flash` on an entity with no
  `MeshMaterial3d` (or an unloaded material) never gets a `FlashState`, so the
  `Flash` component lingers forever. Remove `Flash` on the early return.
  - Response: The setup observer now `remove::<Flash>()`s on both early returns
    (no material component / material asset missing) so a misapplied Flash does
    not linger.
- [x] R1.4 (MINOR) src/feedback/flash.rs - completion unconditionally restores
  the original handle, clobbering any material set by other code mid-flash;
  undocumented. Add a doc sentence.
  - Response: Documented in the module doc ("do not swap the entity's material
    while a flash is active").
- [ ] R1.5 (NIT) one-frame unflashed clone before the first animate. Cosmetically
  invisible; left as-is.
  - Response: Acknowledged, no change (invisible at frame rate).

## Round 2

- VERDICT: APPROVE

R1.1-R1.4 resolved and verified: setup observer is now `On<Insert>` and re-pops
an in-flight flash (new `reflashing_restarts_the_animation` test); `FlashState`
is registered; a `Flash` on a material-less entity is dropped (new
`flash_without_material_is_dropped` test); the mid-flash material-swap limitation
is documented. R1.5 (one-frame clone) left as NIT. 7 flash tests + full suite
green (50 lib tests, 18 doctests, clippy, ascii); 10_asteroids builds.
