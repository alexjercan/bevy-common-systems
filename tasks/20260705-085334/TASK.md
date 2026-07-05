# Bastion: yaw-only orbit camera (A/D + drag, fixed pitch)

- STATUS: CLOSED
- PRIORITY: 90
- TAGS: feature,example,bastion

> Part of the 12_bastion polish goal. Right now the orbit camera in
> `examples/12_bastion.rs` lets A/D yaw and pointer drag change BOTH yaw and
> pitch (arrow up/down also pitch), with an in-game pitch clamp. The user wants
> A/D and drag to ONLY orbit around yaw; pitch must stay fixed at its pleasant
> starting angle. This simplifies the control and removes the pitch-clamp
> machinery entirely.

## Goal

In `examples/12_bastion.rs`, make the orbit camera a pure yaw orbit: A/D (and
left/right arrows) and pointer drag rotate the view around the vertical axis
only. The camera pitch never changes from input; the starting angled top-down
framing is preserved. Drag still distinguishes tap vs orbit for placement.

## Steps

- [x] In `orbit_camera` (around `12_bastion.rs:765`), stop feeding any pitch
      delta: remove the ArrowUp/ArrowDown pitch keys and the
      `pitch -= delta.y * ORBIT_DRAG_RATE` term from the drag branch. Only yaw
      is accumulated from A/D, Left/Right arrows, and drag `delta.x`.
- [x] Remove the now-dead pitch-clamp block (the `forward_y` computation and the
      `PITCH_FORWARD_Y_MIN`/`MAX` gate) and the two constants
      `PITCH_FORWARD_Y_MIN` / `PITCH_FORWARD_Y_MAX`. Set `input.0 =
      Vec2::new(yaw, 0.0)` so `PointRotation`'s pitch axis is always zero.
- [x] Keep `orbit_camera` running in every state and keep it copying
      `out.0` -> `transform.rotation` (the retro's two known bugs: the camera
      never orbited without the Transform copy, and a gated driver spins forever
      on its stale last delta). Do NOT gate it by state.
- [x] Keep the DragState tap/drag bookkeeping intact (start/last/moved/
      released_tap/tap_pos) so `place_or_select` still works.
- [x] Update the module `//!` doc and the menu/HUD hint text that mention pitch
      or "orbit the battlefield" so they say A/D + drag orbit (yaw) only; drop
      the "pitch is clamped in-game" sentence. Grep for `pitch` and
      `PITCH_FORWARD` in the file to catch every mention.
- [x] Adjust the autopilot input closure if needed (it presses `D`) - it stays
      valid; confirm it still exercises orbit.

## Verification

- `cargo clippy --all-targets` clean (examples are the compile gate, not bare
  `cargo build`).
- `cargo fmt --check`, `./scripts/check-ascii.sh`.
- Run headless with the harness and confirm orbit still works as an OBSERVABLE
  effect, not a proxy (per the bastion follow-up retro): temporarily log the rig
  yaw / `transform.rotation` and confirm it changes frame-to-frame while `D` is
  held, and that pitch (forward.y) stays constant. Remove the temp log before
  finishing.
- If `$DISPLAY` is set, boot `cargo run --example 12_bastion --features debug`
  under `timeout` and confirm it reaches the render loop.

## Close-out

Done. `orbit_camera` now feeds `PointRotationInput = (yaw, 0.0)`: A/D and
left/right arrows and horizontal pointer drag accumulate yaw only; the
ArrowUp/ArrowDown pitch keys, the `delta.y` drag pitch term, the `forward_y`
pitch-clamp block, and the `PITCH_FORWARD_Y_MIN/MAX` constants are all gone. The
system still runs in every state and still copies `out.0 -> transform.rotation`,
and the DragState tap/drag bookkeeping is untouched, so placement still works.
Module `//!` docs and the controls line updated; no "pitch" mentions remain
except the code comments explaining pitch is held at zero.

Verified observable-effect (per the follow-up retro, not a proxy): a temporary
`info!` logging the pivot forward showed `forward_y` pinned at exactly 0.0000 for
the whole autopilot run while `yaw_of_forward` swept a full 360 range as `D` was
held (3.14 -> 1.99 -> 0.58 -> -0.91 -> ... wrapping). Temp log removed.

Checks: `cargo clippy --all-targets` clean, `cargo fmt --check` clean,
`./scripts/check-ascii.sh` clean, headless `BCS_AUTOPILOT=1 --features debug`
run completed the full Menu->Playing->GameOver cycle with "no panic".
