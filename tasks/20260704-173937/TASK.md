# helpers: bevy_enhanced_input bridge to UnifiedPointer (retire 06 local Pointer)

- STATUS: CLOSED
- PRIORITY: 25
- TAGS: feature,input,helpers

> Follow-up from tatr 20260704-161508 (input/pointer), review NIT R1.2.
> Spike: tasks/20260704-161210/SPIKE.md.

## Goal

`input/pointer` (tatr 20260704-161508) added `UnifiedPointer` +
`UnifiedPointerPlugin` reading raw `Touches`, and left the `bevy_enhanced_input`
path to `helpers/` on purpose. As a result `examples/06_fruitninja.rs:283` still
carries a local `struct Pointer` (identical in shape to `UnifiedPointer`) whose
`pressed` / `just_pressed` are driven by enhanced-input `Start`/`Complete`
observers on a `PointerPress` action bound to LMB + a touch `Binding::Custom`.

Add a `helpers/` bridge (mirroring how `helpers/wasd` binds enhanced-input for
`camera/wasd`) that writes the crate `UnifiedPointer` from a `bevy_enhanced_input`
press action, so a game wanting the enhanced-input press semantics can still
consume the one shared `UnifiedPointer` resource. Then refactor 06 onto it,
deleting the last local `struct Pointer` copy -- the same proof-by-refactor the
harvest tasks use.

Decide at planning: does the bridge own the full `UnifiedPointer` (position +
press) or only feed the press edge while `UnifiedPointerPlugin` still resolves
position? Keep the core `input/pointer` module free of the enhanced-input
dependency either way.
