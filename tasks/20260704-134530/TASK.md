# ui/popup: floating +N text module (Wave 1)

- STATUS: CLOSED
- PRIORITY: 38
- TAGS: spike,feature,ui

> Spike: docs/spikes/20260704-134035-game-juice-and-scaffolding-kit.md (read
> first). Wave 1 -- promote recurring example juice into the library.

## Goal

Add a `ui/popup` module for the floating "+N" score/damage text that three
games (06, 07, 08) hand-roll: a helper (e.g. `spawn_popup(text, pos, color)`
or a `Commands` extension) that spawns a label which rises and fades over a
lifetime, then self-despawns (build on `helpers/temp`).

Decide at planning time (spike open question) between worldspace billboarded
3D text and a screen-space UI node tracked to a projected world point; if the
latter, the popup helper needs a camera handle for world-to-screen. Prove it
by refactoring one example (07_orbit) onto the module. Once the `tween` module
(task 20260704-134630) exists, back the rise/fade with it rather than a bespoke
lerp.

## Design decisions (planning)

- **Screen-space UI node, not worldspace 3D text.** Resolves the spike open
  question by following the evidence: all three examples (06/07/08) already use
  a screen-space `Text` UI node (`FloatingText { age, lifetime, rise_speed,
  base_color }`) positioned at an absolute px point and rising in px/s. Match
  that. The world-to-screen projection stays with the *caller* (it already
  varies per game -- some project an orb/can/fruit world position, and a static
  banner uses a fixed screen point), so the module needs no camera handle. The
  module owns the animation + despawn, not the projection.
- **Component + bundle-builder API.** Public `Popup { lifetime, rise_speed,
  base_color }` config component (the reusable behavior: put it on any UI text
  node), a private `PopupState { age }`, and a `PopupPlugin` running the animate
  system. Plus a `popup(position, text, font_size, color) -> impl Bundle`
  convenience builder for the common "+N at a viewport point" case (mirrors
  `ui/status::status_bar_item()`); callers add scoping (`DespawnOnExit`, `Name`)
  by chaining. A custom layout (centered banner, different rise) just spawns its
  own `Text`/`Node` with a hand-built `Popup` component.
- **Self-despawn in the animate system**, not via `helpers/temp`. The fade needs
  the age fraction, which `TempEntity`'s timer does not expose (private state),
  so tracking `age` in `PopupState` and despawning at `age >= lifetime` in the
  same system is simpler than running two timers. Node/TextColor are queried
  `Option`-ally so despawn still fires even if a popup lacks them.
- **tween**: deferred. The rise/fade stays a bespoke lerp for now; back it with
  the `tween` module once task 20260704-134630 lands (noted in the module doc).

## Steps

- [x] Add `src/ui/popup.rs`: module doc + usage snippet, `Popup` config,
      private `PopupState`, `PopupSystems`, `PopupPlugin`, the animate system
      (rise + fade + despawn), and the `popup(...)` bundle builder. Factor the
      fade ramp into a pure `popup_alpha(age, lifetime, base_alpha)` fn.
- [x] Tests: `#[cfg(test)]` pure test for `popup_alpha` (full at 0, ~0 at
      lifetime, respects base alpha) + an ECS test (spawn a popup, step, assert
      it rises and despawns at end of life).
- [x] Wire preludes: `pub mod popup;` + `popup::prelude::*` in `src/ui/mod.rs`;
      confirm `crate::prelude` re-exports it.
- [x] Refactor `examples/07_orbit.rs`: delete the local `FloatingText`,
      `animate_floating_text`, and `POPUP_*` consts; route the "+N" popups
      through `popup(...)` and the STREAK banner through a hand-built `Popup`;
      keep the `world_to_viewport` projection in the example.
- [x] Verify: fmt, clippy (both configs), test, test --examples, check-ascii,
      and boot 07_orbit to the render loop.
