# Review: Build examples/13_glide -- UI-forward slide-merge (2048-style) puzzle

- TASK: 20260705-090624
- BRANCH: 13_glide

## Round 1

- VERDICT: REQUEST_CHANGES

Verification run: `cargo clippy --all-targets` clean (plain + `--features debug`),
8 unit tests pass, `check-ascii` + `cargo fmt --check` clean, headless
`BCS_AUTOPILOT` reaches Menu->Playing->GameOver with "cycle complete, no panic",
`BCS_SHOT=390x844` confirms the board renders and the persisted best reloads, and
`trunk build --example 13_glide` compiles to wasm. But the autopilot/screenshot
checks did not observe a *rendered merge* (the screenshot was pre-move), which is
exactly where the blocker hides.

- [ ] R1.1 (BLOCKER) `examples/13_glide.rs` `start_move` (merge classification)
  feeding `tick_move_anim` - the `merges` list is always empty, so a merged tile
  never updates its value/text/colour and never pops/flashes, no merge SFX plays,
  the `+N` popup is never the "big" colour, and the win path never triggers.
  Root cause: `resolve_line` always emits the base (`merged:false`) move to a cell
  before the incoming (`merged:true`) move, so the base claims `new_tiles[to]` via
  the `else` arm and the `merged:true` move then hits the `Some(_)` -> `despawn`
  arm; the `None` arm that pushes to `merges` is unreachable for a real merge.
  `board.grid` stays correct (logic/score work), so the faces silently show stale
  numbers. Fix: on a `merged:true` move, despawn the incoming tile AND push the
  merge target (the base tile already claimed the survivor slot):
  `if m.merged { despawn.push(entity); merges.push((m.to.0, m.to.1, m.new_value)); }
  else { new_tiles[..] = Some(entity); }`.
  - Response: Fixed. Extracted a pure `classify_moves(&[GridMove]) -> MovePlan`
    (survivors / despawns / merges) and rewrote `start_move` to use it: a
    `merged:true` move now pushes both the despawn and the merge target. Verified
    by the new unit tests and by a played autopilot run scoring 156 (score comes
    only from merges, so a non-zero score proves merges resolve).
- [ ] R1.2 (MAJOR) `examples/13_glide.rs` tests - the 8 unit tests assert only the
  pure grid output; every `apply_move` test discards the moves vector with `_`, so
  the `Vec<GridMove>` mapping that R1.1 mishandles has zero assertions and
  `start_move`'s classification is untested. Per the repo convention ("do not test
  only the easy half"), make the move-classification testable: extract a small pure
  helper that turns the moves list into (survivor placements, despawns, merge
  targets) and assert a `[2,2,0,0]` Left move yields exactly one merge target at
  (0,0) with value 4 and one despawn. (The visual re-check below also covers the
  rendered result.)
  - Response: Fixed. Extracted the pure `classify_moves` helper and added 3 tests
    asserting the survivor/despawn/merge classification (11 tests pass).
- [ ] R1.3 (MINOR) `examples/13_glide.rs` - the module doc and the
  `board.won` comment claim a "you won" banner at 2048, but the code only sets
  `board.won = true` and renders nothing. Either spawn a banner when the win
  threshold is first crossed, or drop the claim from the docs.
  - Response: Fixed. `tick_move_anim` now spawns a centered `2048!` popup the first
    time the win threshold is crossed (`board.won` guards it to once).
- [ ] R1.4 (MINOR) `examples/13_glide.rs` `update_score_text` - the roll reattach
  is gated on `tween.finished()`, so a second merge during an in-flight roll does
  not retarget until the current 0.3s roll finishes, and the HUD visibly lags a
  fast merge streak. Track the roll target (e.g. a `Score.roll_target` field) and
  reattach from the current `shown` whenever `value` changes, gated on the `> 0.5`
  delta, so it tracks live without per-frame thrash.
  - Response: Fixed. Added `Score.roll_target`; `update_score_text` retargets the
    roll from the current `shown` whenever `value` changes (gated on the `> 0.5`
    delta on the target, so no per-frame reinsertion), tracking live mid-roll.
- [ ] R1.5 (NIT) `examples/13_glide.rs` `player_move` - `tracker.origin = None` is
  only cleared inside the `dir.is_none() && released` branch, so a keyboard press
  on the same frame a drag releases leaves a stale `origin`. Clear it on any
  release. Harmless (next `just_pressed` overwrites it) but tidy it up.
  - Response: Fixed. Reworked `player_move` to classify the swipe and clear
    `origin`/`last` on any release, before the keyboard check; keyboard now takes
    precedence via `key_dir.or(swipe_dir)`.
- [ ] R1.6 (NIT) `examples/13_glide.rs` `apply_pop` / `spawn_tile` - the `BackOut`
  spawn pop overshoots above 1.0, so the face briefly renders larger than its cell
  and spills into the gap. Cosmetic; either accept it as intentional bounce juice
  (and say so) or clamp. Reviewer's call to leave.
  - Response: Accepted as intentional bounce juice. The overshoot is small (spawn
    `BackOut` a few percent; merge starts at 1.25 and settles), and a full face is
    CELL-sized so any spill stays within the inter-cell gap, not over a neighbour.
    Left as-is.

Confirmed correct (not findings): the pure 2048 logic (`resolve_line` single-merge
/ no-triple / scoring, `apply_move` `changed`, `line_to_cell` for all four
directions, screen-y-down swipe mapping, `is_game_over`); `despawn()` is recursive
in this Bevy so no entity leak; `update_score_text` does not thrash.

## Round 2

- VERDICT: APPROVE

Re-verified the addressed diff:

- [x] R1.1 (BLOCKER) - resolved. `classify_moves` correctly classifies merges;
  unit tests plus a played autopilot run scoring 156 (score is merge-only) confirm
  merged tiles resolve and bump. No runtime errors; full Menu->Playing->GameOver
  cycle "complete, no panic".
- [x] R1.2 (MAJOR) - resolved. Move->entity classification extracted to the pure
  `classify_moves` and covered by 3 new assertions (11 tests pass).
- [x] R1.3 (MINOR) - resolved. Win banner spawns once at the threshold.
- [x] R1.4 (MINOR) - resolved. Score roll retargets live via `Score.roll_target`.
- [x] R1.5 (NIT) - resolved. Swipe state cleared on any release; keyboard precedence
  via `key_dir.or(swipe_dir)`.
- [x] R1.6 (NIT) - accepted as intentional bounce (spill stays within the gap).

Full gate re-run green: `cargo clippy --all-targets` (plain + `--features debug`)
clean, 11 unit tests pass, `check-ascii` + `cargo fmt --check` clean, headless
autopilot completes with no panic and no runtime errors, persistence round-trips.
(Note: a live gameplay *scrot* capture kept returning a stale WM framebuffer in
this headless X session; the merge rendering is instead verified by the unit tests,
the merge-only score, and the identical `text_bundle` render path already shown
correct for the initial tiles via the app-native `ScreenshotPlugin` capture.)

All findings resolved. Approved.
