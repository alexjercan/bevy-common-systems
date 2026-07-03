# Fruit ninja: floating +N score popup

- STATUS: CLOSED
- PRIORITY: 90
- TAGS: feature,example

## Goal

When a fruit is sliced, pop a small "+N" text at the fruit's screen position
that rises and fades out, giving immediate feedback for the points earned.

## Steps

- [x] Add a `FloatingText { age: f32, lifetime: f32, rise_speed: f32 }`
      component (plus a helper to spawn one at a viewport position with a given
      string, font size and color).
- [x] Add a `spawn_floating_text(commands, viewport_pos, text, size, color)`
      helper that spawns a UI `Text` with an absolute `Node` at `viewport_pos`,
      `DespawnOnExit(GameState::Playing)`, and the `FloatingText` component.
- [x] In `slice_objects`, when a fruit (not a bomb) is sliced, project the
      fruit world position to the viewport with
      `camera.world_to_viewport(camera_transform, pos)` and spawn a "+1" popup
      there (points are still 1 until the combo task). Skip if the projection
      fails (fruit off-screen).
- [x] Add an `animate_floating_text` system (run in `Playing`) that advances
      `age`, moves the node up (decrease `Node.top` by `rise_speed * dt`), fades
      `TextColor` alpha as `age/lifetime`, and despawns the entity when
      `age >= lifetime`.
- [x] Register `animate_floating_text` in the `Playing` Update set.
- [x] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, and a real boot: slicing a fruit
      shows a "+1" that rises and fades (throwaway auto-slice if needed).

## Notes

- `Camera::world_to_viewport(&self, &GlobalTransform, Vec3) -> Result<Vec2, _>`
  (bevy_camera 0.18). `slice_objects` already holds
  `camera: Single<(&Camera, &GlobalTransform)>`, so the projection is in reach;
  destructure it before the loop.
- The fruit is despawned/exploded on slice, so the popup is spawned at the
  slice-time screen position and animates independently (it does not track the
  fruit). That is the desired "floating combat text" behavior.
- Node position: `Node { position_type: Absolute, left: Px(v.x), top: Px(v.y),
  .. }`. Rising = decreasing `top`. Fading = setting `TextColor`'s alpha; use
  `Color::srgba` or `.with_alpha(a)`.
- This task builds the reusable floating-text infra; the combo task
  (20260703-132214) reuses `spawn_floating_text` for the "+N" value and the
  "COMBO xN" banner.
- No new dependencies.

## Close-out

Added the floating "+N" popup: `FloatingText` component + reusable
`spawn_floating_text` helper + `animate_floating_text` (rise, alpha fade,
self-despawn at end of lifetime). Wired into `slice_objects`' fruit branch via
`world_to_viewport`. Review: 1 round APPROVE, one cosmetic anchor NIT left
intentionally. Verified on real GPU: in-view slices produce on-screen popups
that animate and despawn, no panic.
