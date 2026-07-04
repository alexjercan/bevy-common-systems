# ui/popup: port 06 and 08 onto the module, delete local copies

- STATUS: OPEN
- PRIORITY: 34
- TAGS: feature,ui,cleanup


> Follow-up from tasks/20260704-134530 (ui/popup module). See
> docs/2026-07-04-ui-popup-module.md and
> docs/retros/20260704-134530-ui-popup-module.md.

## Goal

The `ui/popup` module (PopupPlugin + `popup()` builder) now owns the floating
"+N" text, with 07_orbit refactored onto it. Two more games still hand-roll the
same `FloatingText` component + spawn/animate systems and should be ported:

- `examples/06_fruitninja.rs` -- "+N" score popups and combo banners.
- `examples/08_dropzone.rs` -- "+FUEL" pickup popups (lifetime 0.9, rise 60,
  slightly different from the module default 0.8/70).

## Steps

- [ ] Port `06_fruitninja`: add `PopupPlugin`, route the "+N"/combo popups
      through `popup(...)` (or a hand-built `Popup` for any custom-layout
      banner), delete the local `FloatingText` + `spawn_floating_text` +
      `animate_floating_text` + `POPUP_*` consts. Keep the world_to_viewport
      projection in the example.
- [ ] Port `08_dropzone`: same, but its popup feel is lifetime 0.9 / rise 60 --
      set those on the `Popup` (override the module defaults) so the "+FUEL"
      feel is preserved, or decide the default is close enough and note it.
- [ ] Verify: full check suite + boot both examples to the render loop. Net line
      count should drop across the two files.

## Note

08's popup constants differ from the module default; decide at planning whether
to pass an explicit `Popup { lifetime: 0.9, rise_speed: 60.0, .. }` (preserves
feel exactly) or accept the default. Preserving feel is the safer default for a
pure refactor.
