# Review: promote the full-screen damage overlay into feedback/screen_flash

- TASK: 20260704-155505
- BRANCH: feat/screen-flash

## Round 1

- VERDICT: APPROVE

Clean, real dedup: 110 deletions vs 58 insertions across the three examples plus
the new module. The one-primitive design (linear intensity decay, color in
`BackgroundColor`, `despawn_on_end` to split one-shot from persistent) genuinely
covers both the 06/10 spawn-and-fade and the 07 spike-and-decay shapes with no
behavior change (peak alphas 0.5/0.38 and the decay=1/lifetime mapping match the
originals exactly). Module matches sibling conventions checked against the
feedback-flash retro: `register_type` on both `ScreenFlash` and
`ScreenFlashState`, a `ScreenFlashSystems` set, prelude wiring, and an
`On<Insert>` observer for the re-spike (same as `Flash`). Four tests pin the two
usage shapes (insert-spike + RGB preserved, spawn-and-fade despawn, persistent
re-spike). Full check suite green (fmt, clippy --all-targets, 19 tests, examples
build, ascii) and all three examples boot to the render loop with no panic.

Only one MINOR, an editing oversight, not blocking:

- [ ] R1.1 (MINOR) examples/07_orbit.rs:1143 - `spawn_hud` now carries two
  stacked comment blocks: the original "Full-screen red damage overlay,
  transparent until a hit spikes it. Spawned first and pinned below the HUD..."
  two-liner was left in place above the new "Persistent full-screen overlay..."
  three-liner, so the same thing is explained twice. Delete the old two-line
  comment and keep only the new block.
  - Response: Fixed - deleted the stale two-line comment, kept the new block.
