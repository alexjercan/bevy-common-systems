# Review: route ui/popup + feedback onto Tween

- TASK: 20260704-201801
- BRANCH: feat/tween-adoption

## Round 1

- VERDICT: APPROVE

Realizes the "foundation, not a leaf" claim cleanly: all three modules the spike
named (`ui/popup`, `feedback/flash`, `feedback/screen_flash`) now consume
`Tween<f32>`, and the config components + builders are unchanged, so the games
did not need edits.

The adoptions are each behaviour-preserving and use the right `Tween` shape:
- **popup**: `On<Insert, Popup>` attaches `Tween(base_alpha -> 0, lifetime,
  Despawn)`; animate reads its value for the fade, rise stays velocity. Ageing
  and despawn move entirely to the tween. Using `On<Insert>` (not `Add`) is
  correct -- it lets a game re-inserting `Popup` to override the feel (08's
  0.9s) rebuild the tween.
- **screen_flash**: `Tween(peak -> 0, 1/decay, Despawn|Keep)`; `On<Insert>`
  rebuilds it for the spike-and-decay re-trigger, and a zero decay maps to an
  infinite duration that holds at the peak -- a neat match for the old
  `decay == 0` hold, and it clears a stale `TweenFinished` on re-spike.
- **flash**: `Tween(1 -> 0, duration, Keep)` drives the mix fraction, and an
  `On<Add, TweenFinished>` observer restores the original material and frees the
  clone. The observer is correctly filtered `Query<&FlashState, With<Flash>>`,
  so a game's *other* finishing tweens do not cross-fire it (the one real risk
  with the per-entity `TweenFinished` marker).

Correctness checks that held up:
- No `TweenPlugin` double-add: each plugin adds it guarded by `is_plugin_added`.
  06 adds `TweenPlugin` explicitly (for its slice pop) *before* `PopupPlugin` /
  `ScreenFlashPlugin`, so the guards skip; 07/08/10 have no explicit add so the
  first fade plugin adds it. All four games boot with no duplicate-plugin panic.
- Tests are meaningful and preserved: every ECS test (rise/fade/despawn, spike,
  decay-and-despawn, persistent re-spike, clone-not-shared, restore-and-free,
  reflash-reuses-clone) still passes with its exact-value assertions. The only
  removed tests were for the deleted pure functions `popup_alpha` / `flash_alpha`,
  whose linear-ramp logic is now the tween's `Linear` ease (tested in `tween`) --
  relocated coverage, not weakened.

Verified in the worktree: `cargo fmt --check`, `cargo clippy --all-targets`
(default and `--features debug`), `cargo test` (81 unit + 43 doctests),
`cargo test --examples` and `scripts/check-ascii.sh` all pass; 06/07/08/10 boot
to the render loop with no panic.

- [ ] R1.1 (NIT) src/ui/popup.rs:98 (and the two feedback plugins) - the
  `is_plugin_added::<TweenPlugin>()` guard makes each fade plugin tolerant of
  `TweenPlugin` already being present, but a game that adds `TweenPlugin`
  *explicitly after* a fade plugin would still hit Bevy's duplicate-plugin
  panic (the explicit add is not guarded). Safe today (only 06 adds it
  explicitly, and before the fade plugins), and it matches how the crate's other
  dep-adding plugins behave, so this is informational -- worth a one-line note in
  a plugin doc that `TweenPlugin`, if added by the game, should come first.
  - Response: Accepted as-is (informational). The crate's other guarded dep-adds
    (06 guard-adds `EnhancedInputPlugin` and `FrameTimeDiagnosticsPlugin` the same
    way) carry no such note, so adding one only here would be inconsistent; the
    `is_plugin_added` guard is the established pattern and it is safe as used.
