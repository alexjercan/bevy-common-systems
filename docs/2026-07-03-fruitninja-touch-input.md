# Touchscreen input for 06_fruitninja (via bevy_enhanced_input)

Date: 2026-07-03
Scope: `examples/06_fruitninja.rs` (task 20260703-173128)

## What changed

`06_fruitninja` was mouse-only. It read input two ways, neither of which a
touchscreen drives:

- the press/hold from `Res<ButtonInput<MouseButton>>` (`MouseButton::Left`), and
- the on-screen position from `window.cursor_position()`.

On a touchscreen, touch lands in Bevy's separate `Touches` resource and does
*not* reliably synthesize mouse-button state or a cursor position (this varies
by platform and by winit's `mouse-as-touch`/`touch-as-mouse` behavior, and is
absent under wasm). So the game was unplayable by touch.

The game now reads all pointer input through a small unified `Pointer` layer:

```rust
struct Pointer {
    screen_pos: Option<Vec2>, // active touch, else mouse cursor (logical px)
    pressed: bool,            // finger down or left mouse button held
    just_pressed: bool,       // one-frame press edge (tap / click)
}
```

The four input-facing systems (`menu_click`, `gameover_click`, `slice_objects`,
`draw_cursor_indicator`) read `Pointer` instead of the raw mouse/cursor. On
desktop the behavior is byte-for-byte the same as before; on wasm/mobile the
same code path lights up for a finger.

## Why bevy_enhanced_input, and how touch feeds into it

The request was to route touch through `bevy_enhanced_input`, which the crate
already depends on (the WASD helper uses it). Two facts shaped the design:

1. **enhanced_input 0.26 has no native touch binding.** The `Binding` enum is
   keyboard / mouse button / mouse motion / mouse wheel / gamepad / `AnyKey` /
   `Custom` / `None`. Its documented extension point for "inputs that can't feed
   into Bevy input resources" is `Binding::Custom(CustomInput)`, fed via the
   `CustomInputs` resource -- their own docs feed trackpad pinch gestures this
   exact way. So the touch-pressed state is staged into a registered
   `CustomInput` every frame, and the press action binds to it.

2. **enhanced_input has no absolute-pointer-position binding at all** (only
   mouse *motion*), not even for the mouse. So the on-screen *position* is
   fundamentally read outside enhanced_input. It stays a direct read of
   `Touches` / `window.cursor_position()`.

The result: a single bool `PointerPress` action bound to BOTH
`MouseButton::Left` and `Binding::Custom(touch_id)`, so either device actuates
the same action. Its `Start`/`Complete` events drive `pressed`/`just_pressed`.
The position is resolved separately. This is the honest split -- enhanced_input
does the part it is good at (unifying a press across devices with one action),
and the position, which it has no binding for, is read directly.

### Wiring

- `stage_pointer_input` runs in `PreUpdate`, `.after(bevy::input::InputSystems)`
  and `.before(EnhancedInputSystems::Prepare)` (the `CustomInputs` docs use
  `.before(EnhancedInputSystems::Update)`; Prepare is earlier and safe). It
  writes `CustomInputs[touch_id] = Bool(any active touch)` and sets
  `pointer.screen_pos` (touch wins over cursor).
- `Start<PointerPress>` sets `pressed = true, just_pressed = true`;
  `Complete<PointerPress>` sets `pressed = false`; a `Last` system clears
  `just_pressed` so it is edge-triggered (true for exactly one frame). Start
  fires in PreUpdate, before the Update consumers read it, and Last clears it
  after -- clean one-frame semantics.
- The touch `CustomInput` is registered in `setup`, in the same system that
  spawns the action entity, so the id is in scope for both the `Binding::Custom`
  and the `TouchInputId` resource with no cross-system ordering to get wrong.

## Alternatives considered

- **Read `Touches` directly in each system, no enhanced_input.** Simpler, but
  the request was explicitly to use enhanced_input, and routing the press
  through one action is genuinely cleaner than duplicating a
  mouse-or-touch check in four systems.
- **Route the position through a `CustomInput` Axis2D too**, so the whole
  pointer goes through enhanced_input. Rejected: a continuous absolute position
  is a poor fit for enhanced_input's Fire/Complete value semantics, and it would
  wrap the mouse cursor in redundant machinery just to funnel it through the
  same pipe. Position as a plain read is clearer and is what enhanced_input's
  own design implies (it has no such binding).
- **A reusable `helpers/pointer` crate module** instead of in-example code.
  Deferred: a crate-level abstraction wants the crate's Config/Input/Output
  component split, a prelude, and a name that generalizes past this game -- a
  real design task on its own. This task was scoped to making the game playable
  by touch. If a second example needs the same unification, extract then.

## Verification

- Native: full check suite green -- `cargo build`, `cargo clippy --all-targets`
  (+`--features debug`), `cargo fmt --check`, `cargo test` (+`--features debug`),
  `./scripts/check-ascii.sh`. The example's own tests (`cargo test --example
  06_fruitninja`, 19 incl. 3 new `active_pointer_pos` tests) pass. Mouse play is
  unchanged.
- Web: built through the real showcase entry point,
  `web/scripts/build-games.sh` (trunk `--release --example 06_fruitninja`),
  confirming it compiles and packages for `wasm32-unknown-unknown`. On-device
  touch play is left for the user to confirm on real hardware.

## Gotchas / notes for next time

- **CI's `cargo test` does not run example tests.** The example's `#[cfg(test)]`
  block (now 19 tests) only runs under `cargo test --example 06_fruitninja` /
  `--examples`; plain `cargo test` builds the example but skips its tests. This
  predates this task (the geometry/combo tests were already unreachable from
  CI). Worth adding `--examples` to the CI test step in a follow-up so these
  don't silently rot.
- enhanced_input is 0.26.0 in Cargo.toml; AGENTS.md still says 0.25. Minor
  staleness, flagged for a docs pass.
