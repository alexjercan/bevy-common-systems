# Review: Fruit ninja floating +N score popup

- TASK: 20260703-132210
- BRANCH: feature/fruitninja-popup

## Round 1

- VERDICT: APPROVE

Clean, reusable addition. `spawn_floating_text` spawns a `Text` UI node at a
viewport position with a `FloatingText` component; `animate_floating_text`
rises the node, fades `TextColor` alpha over an 0.8s lifetime, and despawns it;
popups are also `DespawnOnExit(Playing)`. Storing the base `color` and
recomputing alpha each frame avoids compounding-fade bugs. The viewport space
is consistent: `world_to_viewport`'s output matches the same space as the
existing cursor code (`cursor_position` -> `viewport_to_world`), which is UI
logical pixels, so placement is scale-correct. Verified on real GPU: slicing
in-view fruit produced on-screen popups (e.g. (231, 869) in a 640x1057 window),
5 spawned, live-count peaked at 1 then despawned, no panic. Checks clean
(`fmt`, `clippy --all-targets` both configs, `check-ascii`).

- [ ] R1.1 (NIT) examples/06_fruitninja.rs:spawn_floating_text - the node is
  anchored at its top-left on the projected point, so "+1" sits down-right of
  the fruit center rather than centered on it. Cosmetic; if a centered popup is
  wanted, subtract roughly half the text width from `left` and a bit from
  `top`. Fine to leave for a small "+1".
  - Response: Left as-is. Centering needs text-width measurement (approximate
    fixed offsets would look wrong once the combo task adds wider "COMBO xN"
    text); acceptable for a short "+N".
