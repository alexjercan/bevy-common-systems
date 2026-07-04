# Review: ui/touchpad reveal-on-touch + hit-test primitives

- TASK: 20260704-161513
- BRANCH: feat/ui-touchpad

## Round 1

- VERDICT: APPROVE

Clean Wave A harvest, scoped exactly to the user's "primitives only" decision
(no button-row builder). New `ui/touchpad` wired into the ui prelude ships:
`TouchpadPlugin` + `TouchSeen` + `RevealOnTouch`/`HideOnTouch` markers, and the
pure `button_grid_at` / `stick_deflection` hit tests. Module/item docs, the
`debug!`/`trace!` logging, the `*Systems` set and `Reflect`/`register_type`
registration all follow the crate conventions. Names checked against
`bevy::prelude` (the gotcha from the last cycle) -- no collisions, confirmed by
a clean `clippy --all-targets` with both preludes glob-imported.

Behaviour-equivalence verified against the deleted code:
- Reveal semantics are identical. Old 11 `update_touch_pad`: pad `seen ?
  Visible : Hidden`, legend `seen ? Hidden : Inherited`. New markers:
  `RevealOnTouch` = same as pad, `HideOnTouch` = same as legend. Old 08
  `update_touch_hud` root reveal = `RevealOnTouch`. The steer ring/knob
  positioning stayed in 08's game system.
- `stick_deflection` reproduces `touch_lean`'s dead-zone + unit-disc clamp; 08
  keeps the per-axis sign and `MAX_LEAN` scale in its wrapper. It also adds a
  `radius <= dead` guard that fixes a latent div-by-zero (the old `touch_lean`
  would NaN if `radius == dead`); unreachable with 08's constants, strictly
  more robust.
- `button_grid_at` reproduces `vent_button_at` (11's local test still passes
  through the delegating wrapper) and adds a correct upper-y bound that the old
  code lacked (unreachable in practice, since touches sit inside the window).
- Frame-derivation lesson respected: the primitive never latches a touch id;
  `mark_touch_seen` reads the live `any_just_pressed()` each frame and the hit
  tests are pure.

Verified independently in the worktree: `cargo fmt --check`,
`cargo clippy --all-targets` (default and `--features debug`), `cargo test`
(60 unit + 29 doctests), `cargo test --examples` (08 and 11 wrapper tests still
green) and `scripts/check-ascii.sh` all pass; 08 and 11 both boot to the render
loop with no panic.

- [ ] R1.1 (NIT) src/ui/touchpad.rs:186 - `mark_touch_seen` runs in `PreUpdate`
  in every state, whereas both games previously flipped `TouchSeen` only from
  Playing-gated systems (`update_touch_pad` / `update_touch_control`, both
  `run_if(in_state(Playing))`). Net effect: on a touch device, tapping the menu
  now marks `TouchSeen`, so the pad is already revealed when Playing begins,
  instead of after the first in-game tap. This is a benign (arguably better)
  behaviour change and desktop is unaffected (no touch -> never marked), but it
  is a change worth being a conscious one. No action needed unless the deferred
  reveal was intended; if so, gate the plugin or expose a set the game can
  order. Informational.
  - Response: Accepted as intended -- revealing the pad the instant Playing
    starts for a user who already tapped the menu is the better behaviour, and
    desktop is unaffected. Left the plugin running in all states (it is also
    what lets menu/meltdown screens share one `TouchSeen`). No code change.
