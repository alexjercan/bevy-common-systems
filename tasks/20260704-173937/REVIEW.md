# Review: helpers enhanced-input bridge to UnifiedPointer

- TASK: 20260704-173937
- BRANCH: feat/pointer-enhanced-bridge

## Round 1

- VERDICT: APPROVE

Delivers the deferred follow-up cleanly and answers its planning question well.
The design decision -- the bridge owns the whole `UnifiedPointer` (press +
position), used *instead of* the raw `UnifiedPointerPlugin`, never alongside --
is the right one: both plugins write the resource every frame, so mutual
exclusivity is the only coherent contract, and the module doc states it plainly.
It mirrors `helpers/wasd` (the enhanced-input binding layer for `camera/wasd`),
so it fits the established shape.

Correctness / equivalence:
- The bridge's schedule is byte-identical to 06's old machinery: `setup_pointer_action`
  in `Startup`, `stage_pointer_input` in `PreUpdate` `.after(InputSystems)
  .before(EnhancedInputSystems::Prepare)`, `clear_pointer_just_pressed` in
  `Last`, and the two `Start`/`Complete` observers. So it is a faithful lift, not
  a re-derivation.
- Position resolve uses `Query<&Window, With<PrimaryWindow>>().iter().next()`
  rather than the old `Single<&Window>` -- more robust (no panic without a
  window), equivalent for a single-window game, and consistent with the raw
  `UnifiedPointerPlugin`.
- Registers `UnifiedPointer`'s reflect type (parity with the raw plugin), guards
  the `EnhancedInputPlugin` add, and keeps the enhanced-input dependency in
  `helpers/` so the core `input/pointer` stays dependency-free -- exactly the
  task's constraint.

06 refactor is complete: the local `struct Pointer`, the input context, action,
custom-input id, staging system, both press observers, the frame-end clear, the
`main()` wiring and the `setup` registration are all gone, the
`bevy_enhanced_input` import is dropped, and all `Res<Pointer>` became
`Res<UnifiedPointer>`. 138 lines removed from the example, 159 added as the
reusable module. Remaining `Pointer`/enhanced-input mentions in 06 are prose
only. The last duplicated `Pointer` copy the harvest tasks tracked is retired.

Verification is exemplary: rather than ship an input refactor on "it compiles",
the implementer drove it end-to-end with the crate's own `AutopilotPlugin`
(`BCS_AUTOPILOT`, `--features debug`) -- 35 probe frames showing
`pressed=true`, a correct one-frame `just_pressed` edge, a resolved `screen_pos`,
and a clean `Menu -> Playing -> GameOver, no panic`. That the enhanced-input
path can't be stood up in a minimal unit test (it needs a full app) is why the
crate exercises ECS via examples; 06 + autopilot is that exercise, matching how
`helpers/wasd` is covered.

Verified in the worktree: `cargo fmt --check`, `cargo clippy --all-targets`
(default and `--features debug`), `cargo test` (67 unit + 37 doctests, the new
bridge doctest included) and `scripts/check-ascii.sh` all pass.

- [ ] R1.1 (NIT) src/helpers/pointer.rs:70 - adding both `EnhancedInputPointerPlugin`
  and `UnifiedPointerPlugin` silently fights over the resource; the doc warns
  against it but nothing enforces it. Consider a debug-only guard (e.g. warn or
  `debug_assert!(!app.is_plugin_added::<UnifiedPointerPlugin>())` in `build`).
  Optional -- the crate does not generally guard plugin misuse, so a doc note may
  be enough. Not blocking.
  - Response: Left as a doc warning, no runtime guard. The crate does not guard
    plugin misuse anywhere else (e.g. adding two conflicting controllers), so a
    guard here would be an inconsistent one-off; the module doc's "use it instead
    of, not alongside" is the crate's established level of protection.
