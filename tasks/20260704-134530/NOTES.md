# ui/popup: floating "+N" text module

- DATE: 2026-07-04
- TASK: tasks/20260704-134530
- SPIKE: tasks/20260704-134035/SPIKE.md (Wave 1)

## What changed

Added `src/ui/popup.rs` (`PopupPlugin`), promoting the floating "+N" score /
"+FUEL" pickup text that 06/07/08 hand-roll into the library, and refactored
`examples/07_orbit.rs` onto it (deleting its local `FloatingText` component,
`spawn_floating_text` / `animate_floating_text`, and the `POPUP_*` consts).

API: a public `Popup { lifetime, rise_speed, base_color }` component put on any
UI `Text` node, a private `PopupState { age }`, and a `popup(position, text,
font_size, color) -> impl Bundle` convenience builder for the common "+N at a
viewport point" case (mirrors `ui/status::status_bar_item()`).

## Key decisions

### Screen-space UI node, not worldspace 3D text

The spike left open whether the popup should be worldspace billboarded 3D text
or a screen-space UI node tracked to a projected world point. Resolved by
following the evidence: all three examples already use a screen-space `Text` UI
node that rises in px/s and fades. Matching that keeps the refactor a true
promotion (delete-and-replace, no behavior change) and avoids a 3D-text
dependency.

### The projection stays with the caller

A screen-space popup for a *world* event needs a world-to-screen projection.
Rather than take a camera handle in the module, the caller projects
(`camera.world_to_viewport(...)`) and passes the resulting viewport point to
`popup(...)`. This is deliberate: the projection varies per game (07 projects an
orb position; a static banner uses a fixed screen point; a future game might
anchor to a HUD slot), and threading a camera through the module would couple it
to a single camera setup. The module owns the animation and self-despawn; the
caller owns placement.

### Component + bundle-builder, and a custom-layout escape hatch

The reusable core is the `Popup` component (the rise/fade/despawn behavior),
which can sit on any `Text`/`Node`. The `popup()` builder covers the common
anchored "+N". A non-standard layout -- 07's centered, full-width "STREAK xN"
banner with a slower rise -- just spawns its own `Text`/`Node`/`TextLayout` with
a hand-built `Popup { rise_speed, base_color, ..default() }`. So the module
handles both the 90% case and the outliers without a rigid one-size builder.

### Self-despawn, not helpers/temp

The task suggested building on `helpers/temp`. `TempEntity` would own the
despawn, but the fade needs the age *fraction*, which `TempEntity`'s timer keeps
private -- so the popup would need its own age counter anyway. Tracking `age` in
`PopupState` and despawning at `age >= lifetime` in the one animate system is
simpler than running two timers. `Node`/`TextColor` are queried `Option`-ally so
an expired popup still despawns even if it somehow lacks them.

### tween: deferred

The rise/fade is a linear lerp. When the `tween` module (task 20260704-134630)
lands it should back the rise and fade so the easing is shared, per the spike;
noted in the module doc.

## Testing

- Pure `popup_alpha(age, lifetime, base_alpha)` unit tests: full at birth, half
  at half-life, zero (clamped) at/after lifetime, respects a translucent base
  alpha, handles zero lifetime.
- ECS test (`popup_rises_fades_and_despawns`): a minimal `App` with `PopupPlugin`
  and `Time` driven by hand -- one step confirms the node rose and the color
  faded; stepping past the lifetime confirms it despawned.
- `examples/07_orbit` boots to the render loop; its own unit tests are untouched.

## Follow-ups

06 and 08 still hand-roll the popup; porting them (and 08's "+FUEL" feel) is left
for a follow-up so this task stays scoped to the module plus one proof-of-use
refactor.
