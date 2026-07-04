# Review: port 06/08 floating text onto ui/popup

- TASK: 20260704-153237
- BRANCH: feat/ui-popup-ports

## Round 1

- VERDICT: APPROVE

Independent review verified both files against master with no findings:

- 06: all 4 `spawn_floating_text` call sites (COMBO tally, golden "+N", plain
  "+N", in-flow "COMBO xN") ported to `popup(...)` with identical
  position/text/font-size/color and `DespawnOnExit(Playing)` scoping; none
  dropped. The COMBO banner was a normal viewport popup in master (no custom
  layout), so `popup()` preserves its look. Old consts equalled the module
  defaults, so feel is unchanged.
- 08: the single "+FUEL" site spawns the builder then overrides with
  `Popup { lifetime: 0.9, rise_speed: 60.0, .. }`, preserving 08's distinct feel
  exactly (not the 0.8/70 default); same color/size/scoping.
- Both delete `FloatingText`, `spawn_floating_text`, `animate_floating_text` and
  the `POPUP_*` consts; remove `animate_floating_text` from add_systems; add
  `PopupPlugin`. No orphaned refs, no surviving `With<FloatingText>` queries,
  clippy --all-targets clean, both examples build.

Only a NIT: `popup()` names the entity "Popup" vs the old "Floating Text"; not
queried anywhere, harmless.
