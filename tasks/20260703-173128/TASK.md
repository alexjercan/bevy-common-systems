# Fruit ninja: touchscreen support via bevy_enhanced_input

- STATUS: CLOSED
- PRIORITY: 95
- TAGS: feature,example,input,touch

## Goal

Make `examples/06_fruitninja.rs` fully playable on a touchscreen: tap the menu
to start, hold a finger down and swipe to slice fruit, and tap the game-over
screen to return to the menu. The touch press must be routed through
`bevy_enhanced_input` (per the request), and mouse play must keep working
unchanged on desktop. "Done" = the whole loop (menu -> slice -> game over ->
menu) is drivable with touch alone, and with the mouse alone, from the same
code path.

## Background / why this shape

The game today reads input in two raw ways, neither of which touch drives:

- `MouseButton::Left` via `Res<ButtonInput<MouseButton>>` (menu_click,
  gameover_click, slice_objects, draw_cursor_indicator).
- `window.cursor_position()` for the pointer's screen position
  (`cursor_on_play_plane`).

On a touchscreen neither updates: touch lands in Bevy's separate `Touches`
resource and does not synthesize mouse-button or cursor-position state (not
reliably across native/wasm). So both the press signal and the position must be
sourced from touch as well as mouse.

`bevy_enhanced_input` 0.26 has NO native touch binding (the `Binding` enum is
keyboard / mouse button / mouse motion / mouse wheel / gamepad / `AnyKey` /
`Custom` / `None`). Its sanctioned extension point for "inputs that can't feed
into Bevy input resources" is `Binding::Custom(CustomInput)` fed via the
`CustomInputs` resource (their own docs feed trackpad pinch gestures this way):
register an id, stage a value into it every frame in `PreUpdate` after
`bevy::input::InputSystems` and before `EnhancedInputSystems`, then bind an
action to `Binding::Custom(id)`. That is how touch will drive an enhanced_input
action here.

enhanced_input also has no absolute-pointer-position binding even for the mouse
(only mouse *motion*), so the on-screen position is read directly from
`Touches` / `window.cursor_position()` outside enhanced_input. That is expected
and idiomatic; only the press/hold goes through enhanced_input.

Design: a small unified pointer layer in the example.

- A `Pointer` resource: `{ screen_pos: Option<Vec2>, pressed: bool,
  just_pressed: bool }`.
- One enhanced_input context entity with a single bool `PointerPress` action
  bound to BOTH `MouseButton::Left` AND `Binding::Custom(touch_active)`, so
  either device presses the same action. `Start` -> pressed=true +
  just_pressed=true; `Complete` -> pressed=false.
- A `PreUpdate` staging system feeds `CustomInputs[touch_active] =
  Bool(any active touch)` and sets `Pointer::screen_pos` (an active touch's
  position takes priority, else the mouse cursor position).
- A `Last` system clears `just_pressed` so it is true for exactly one frame.
- The four consumer systems switch from `mouse`/`cursor_position()` to the
  `Pointer` resource.

Keep it in the example (not a new src/ module). A reusable crate-level pointer
abstraction (Config/Input/Output split, prelude, etc.) is a larger design
decision that deserves its own task; this task is scoped to making the game
playable by touch. Note that as a follow-up in the Outcome if it still seems
worth extracting.

## Steps

- [x] Add `bevy_enhanced_input` to the example's use imports and register the
      plugin + context in `main`: `app.add_plugins(EnhancedInputPlugin)` (guard
      with `is_plugin_added` in case a future dep adds it) and
      `app.add_input_context::<PointerInput>()`. Confirm `EnhancedInputPlugin`,
      `EnhancedInputSystems`, `CustomInputs`, `CustomInput`, `Binding`,
      `Action`, `Start`, `Complete`, `actions!`, `bindings!` all come from
      `bevy_enhanced_input::prelude`.
- [x] Define the `Pointer` resource `{ screen_pos: Option<Vec2>, pressed: bool,
      just_pressed: bool }` (Resource, Default) and `init_resource` it. Define
      a `PointerInput` context marker component and a `PointerPress` bool
      `InputAction` (`#[derive(InputAction)] #[action_output(bool)]`).
- [x] Register the touch custom input once and store its id in a resource
      (e.g. `TouchInputId(CustomInput)`): in a startup system (or in `setup`)
      call `custom_inputs.register_input()` and `insert_resource`. This must
      run before the staging system reads it.
- [x] Spawn one persistent input entity (in `setup`) carrying `PointerInput`
      and `actions!(PointerInput[( Action::<PointerPress>::new(),
      bindings![MouseButton::Left, Binding::Custom(touch_id)] )])`. The touch
      id needs to be available at spawn; either register it before `setup`
      spawns (order the startup systems / registration) or spawn the action
      entity in the same system that registers the id. Pick the simplest
      ordering and document it in a comment.
- [x] Add a `PreUpdate` staging system `stage_pointer_input`, scheduled
      `.after(bevy::input::InputSystems)` and
      `.before(EnhancedInputSystems::Prepare)` (matching the crate's custom-
      input docs, which use `.before(EnhancedInputSystems::Update)`; Prepare is
      earlier and safe): read `Res<Touches>` and `Single<&Window>`, set
      `custom_inputs.insert(touch_id, ActionValue::Bool(has_active_touch))`,
      and set `pointer.screen_pos` = first active touch position if any, else
      `window.cursor_position()`.
- [x] Add `Start<PointerPress>` and `Complete<PointerPress>` observers that set
      `pointer.pressed`/`pointer.just_pressed`. Add a `Last` system
      `clear_pointer_just_pressed` that sets `just_pressed = false` each frame.
      Verify one-frame semantics: Start fires in PreUpdate (before Update
      consumers), Last clears after them.
- [x] Rewrite the consumers to use `Pointer`:
      - `menu_click`: `if pointer.just_pressed` (drop the `mouse` param).
      - `gameover_click`: `if pointer.just_pressed`.
      - `slice_objects`: replace `mouse.pressed(Left)` with `pointer.pressed`;
        replace `cursor_on_play_plane(&window, cam, cam_t)` with a
        position-taking projection of `pointer.screen_pos`.
      - `draw_cursor_indicator`: replace `mouse.pressed(Left)` with
        `pointer.pressed` and use `pointer.screen_pos`.
- [x] Refactor `cursor_on_play_plane(window, camera, camera_transform)` into
      `pointer_on_play_plane(screen_pos: Vec2, camera, camera_transform)` (take
      the already-resolved screen position instead of reading the window), so
      both mouse and touch share one projection path. Update both call sites.
- [x] Keep the `Escape` give-up (`giveup_on_escape`) as-is (keyboard, harmless
      on touch/desktop). Update the CLI `about` / module `//!` doc text to say
      "tap or click and swipe" instead of mouse-only, and mention touch works.
- [x] Update the menu hint text if needed ("swipe to slice ...") -- it already
      says "swipe", which reads fine for touch; adjust only "Click to play" /
      "Click to return to menu" copy to "Tap or click ...".
- [x] Add/keep pure-logic unit tests where it helps (e.g. a helper that picks
      the active pointer position given touch-vs-cursor). Do not try to unit
      test the ECS wiring; the example itself is the integration test.
- [x] Verify native: `cargo run --example 06_fruitninja` (in `nix develop`)
      still plays with the mouse exactly as before (menu click, hold-swipe
      slice, game over click). Redirect build output to a file and check `$?`
      -- do NOT judge a build by a piped `| tail` (known gotcha).
- [x] Verify the wasm/web path, since touch mainly matters there: build the
      game through the real entry point used by the showcase
      (`web/scripts/build-games.sh` via `npm run build`, or at minimum
      `trunk build` FROM THE REPO ROOT with the wasm target), not a hand-rolled
      trunk call from a subdir. Confirm it compiles for
      `wasm32-unknown-unknown`. Actual on-device touch testing is the user's
      call; note what was and wasn't verified.
- [x] Keep CI green: `cargo build`, `cargo clippy --all-targets`,
      `cargo clippy --all-targets --features debug`, `cargo fmt --check`,
      `cargo test`, `cargo test --features debug`, `./scripts/check-ascii.sh`.
- [x] Document the decision in `docs/` (why CustomInputs, why position stays
      out of enhanced_input, why in-example not a src/ module), per AGENTS.md.

## Notes

- enhanced_input version is 0.26.0 (Cargo.toml), not 0.25 as AGENTS.md says;
  the module map is slightly stale. API confirmed against the vendored source:
  events `Start`/`Fire`/`Complete`/`Cancel` in
  `bevy_enhanced_input::action::events`; `CustomInputs`/`CustomInput` in the
  prelude; `Binding::Custom(CustomInput)`; `EnhancedInputSystems` variants
  `Prepare`/`Update`/`Apply`.
- Model the context/action wiring on `src/helpers/wasd.rs`
  (`add_input_context`, `actions!`, `Action::<T>::new()`, `bindings![...]`,
  `On<Fire<...>>` / `On<Complete<...>>` observers). This example does not use
  the WASD camera, so enhanced_input is NOT yet in the app -- it must be added.
- Relevant code (line numbers approximate, examples/06_fruitninja.rs):
  `main` plugin/registration (~L131-240), `setup` (~L513), `menu_click`
  (~L849), `gameover_click` (~L1002), `slice_objects` (~L1110),
  `draw_cursor_indicator` (~L1276), `cursor_on_play_plane` (~L1403).
- `Touch::position()` and `Window::cursor_position()` are both logical window
  coordinates, so `camera.viewport_to_world` works identically for both.
- On desktop `Touches` is always empty, so the touch custom input stays
  `false` and behavior is identical to today (mouse-left through the action +
  cursor position). On wasm/mobile the same code path lights up for touch.
- Web relevance: the showcase embeds this game in a portrait, mobile-ish frame
  (see the wasm32 window config at ~L139), so touch is the primary reason to
  build the web version; verifying the wasm build is part of "done".

## Outcome

Added a unified mouse + touch `Pointer` layer to `examples/06_fruitninja.rs`.
The press/hold is one `bevy_enhanced_input` action (`PointerPress`) bound to
BOTH `MouseButton::Left` and `Binding::Custom(touch_id)`; `stage_pointer_input`
(PreUpdate, after `InputSystems`, before `EnhancedInputSystems::Prepare`) feeds
the touch value from Bevy's `Touches` and resolves `Pointer::screen_pos` (active
touch wins over the mouse cursor). `Start`/`Complete<PointerPress>` observers
drive `pressed`/`just_pressed`, and a `Last` system clears `just_pressed` so it
is edge-triggered for one frame. The four input systems (`menu_click`,
`gameover_click`, `slice_objects`, `draw_cursor_indicator`) now read `Pointer`;
`cursor_on_play_plane` became `pointer_on_play_plane(screen_pos, ...)` shared by
both devices. Menu/game-over copy is "Tap or click ..."; module and CLI docs
mention touch.

Design decisions (full write-up in
`tasks/20260703-173128/NOTES.md`):

- enhanced_input 0.26 has NO native touch binding, so touch is fed through its
  documented `Binding::Custom` + `CustomInputs` extension point (same mechanism
  its docs use for trackpad pinch gestures).
- enhanced_input has no absolute-pointer-position binding even for the mouse, so
  the position stays a direct `Touches`/`cursor_position()` read; only the
  press goes through enhanced_input. This is the honest split.
- The touch `CustomInput` is registered in `setup`, in the same system that
  spawns the action entity, so its id is in scope for both the binding and the
  `TouchInputId` resource with no cross-system ordering.
- Kept in the example, not a new `src/` module: a reusable crate-level pointer
  abstraction is a larger design task (Config/Input/Output split, prelude,
  general name). Extract if a second example needs it.

Desktop behavior is unchanged: on native `Touches` is always empty, so the
custom input stays `false` and the position falls back to the cursor -- the same
code path the mouse always drove.

Verification: `cargo build`, `cargo clippy --all-targets`
(+`--features debug`), `cargo fmt --check`, `cargo test` (+`--features debug`),
`./scripts/check-ascii.sh` all exit 0. `cargo test --example 06_fruitninja` runs
19 tests (3 new `active_pointer_pos` cases) green. The wasm/web path was built
through the real showcase entry point `web/scripts/build-games.sh`
(trunk `--release --example 06_fruitninja`): "success", a 49MB bundle under
`web/build/games/06_fruitninja/`, confirming the touch code compiles and
packages for `wasm32-unknown-unknown`. On-device touch play is left for the user
to confirm on real hardware.

Follow-ups noted: (1) CI's `cargo test` does not run example tests -- the
example's `#[cfg(test)]` block (19 tests) only runs under `--example` /
`--examples`; worth adding `--examples` to the CI test step so these do not
silently rot. (2) AGENTS.md says enhanced_input 0.25 but it is 0.26.0; minor
staleness for a docs pass.
