# Retro: 12_bastion on-screen build + upgrade buttons

- TASK: 20260705-085337
- BRANCH: feature/bastion-build-buttons (squash-merged to master)
- REVIEW ROUNDS: 1 (APPROVE with 1 NIT, fixed in-round)

Third and largest task of the 12_bastion polish flow: an always-visible bottom
build bar (tower + upgrade buttons with keybind labels) that works by tap/click
on mobile and desktop.

## What went well

- Fanned out an Explore agent FIRST to gather the exact reusable patterns
  (11_overload's vent-pad Node idiom, `ui/touchpad::button_grid_at`,
  `UnifiedPointer` semantics, the spark idiom). The first full compile had only
  trivial errors (a `WindowResolution::from` type and an over-deref), not a
  batch -- the standing "copy/verify the 0.19 visual layer, don't improvise it"
  lesson paying off yet again. Copying `spawn_vent_pad`'s exact
  `border_radius`-in-`Node` / `BorderColor::all` / `TextFont{FontSize::Px}` form
  meant zero UI-API churn.
- Chose region-owns-tap over Bevy `Interaction`, per the exploration's warning
  that `Interaction` + `UnifiedPointer` would double-count a press. The result is
  clean: `build_bar_input` and `place_or_select` both only READ `DragState` and
  partition the tap by a shared `BUILD_BAR_H_FRAC` zone, so no ordering between
  them is needed and a button tap never doubles as a world tap.
- Verified the ADVERTISED control's observable effect, not a proxy (the follow-up
  bastion retro's headline lesson, and exactly the class of bug that shipped last
  time -- "pressed D, never confirmed the view moved"). The autopilot is
  keyboard-only and cannot tap a button, so I wrote a real integration test that
  drives `build_bar_input` through a minimal App with a sized `Window` and
  asserts `build.spec` is armed and a selected tower actually levels up. Plus a
  phone-width screenshot to confirm the one-row layout renders (reactor mobile
  lesson).
- Honoured the packs-task lesson: ran a plain `cargo build --example` as a
  dead-code gate (not just `--all-targets` + tests), both before merge and after,
  so no test-only helper slipped through this time.

## What went wrong

- Two small first-compile errors that a moment's care would have avoided:
  `WindowResolution` has no `From<Vec2>`/`From<(f32,f32)>` (it is `From<(u32,u32)>`
  / `UVec2`), and I wrote `***credits` where `**credits` was right for a
  `&mut Credits` param. Both are the "improvised an API surface from memory"
  pattern the repo keeps flagging; here they were in test/helper code so they
  cost only a compile round, not a review finding.
- The integration test panicked first run because `SoundBank::load` calls
  `asset_server.load::<AudioSource>` and the minimal App had not registered the
  `AudioSource` asset type. Root cause: reached for a real `AssetServer` in a
  headless test without registering the asset it loads. Fix was one line
  (`app.init_asset::<AudioSource>()`), but it is a reusable gotcha: any headless
  test that builds a `SoundBank` (or otherwise `asset_server.load::<T>`s) must
  `init_asset::<T>()` first or it panics in `bevy_asset`.

## What to improve next time

- When a headless/minimal-App test needs a resource that internally loads an
  asset (e.g. `SoundBank`), register the asset type with `init_asset::<T>()`
  up front. Add to the mental checklist alongside the "doctests that build an App
  need StatesPlugin" gotcha.
- For any new Bevy value type used in a test (`WindowResolution`, ...), grep the
  crate source for its actual `From`/`new` impls before writing the literal,
  rather than assuming the ergonomic conversion exists.
