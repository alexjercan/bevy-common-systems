# Review: Bastion yaw-only orbit camera

- TASK: 20260705-085334
- BRANCH: feature/bastion-yaw-orbit

## Round 1

- VERDICT: APPROVE

Reviewed `git diff master...HEAD` (a single ~40-line change to
`examples/12_bastion.rs`). The diff delivers the Goal cleanly:

- `orbit_camera` now feeds `PointRotationInput = Vec2::new(yaw, 0.0)`. The
  ArrowUp/ArrowDown pitch keys, the `delta.y` drag-pitch term, the `forward_y`
  pitch-clamp block, and the `PITCH_FORWARD_Y_MIN/MAX` constants are all removed.
  Grep confirms no `ArrowUp|ArrowDown|PITCH_FORWARD|pitch =` stragglers remain
  (only the explanatory comments about pitch-held-at-zero).
- The system still runs in every state and still copies
  `out.0 -> transform.rotation`, so both retro-documented bugs (missing Transform
  copy; state-gated driver spinning on a stale delta) stay avoided. The
  `PointRotationOutput` binding `out` is still used, so no dead binding.
- DragState tap/drag bookkeeping is untouched, so `place_or_select` still works.
- Module `//!` doc, the orbit-control constant doc, the `orbit_camera` doc, and
  the controls line are all updated consistently; no misleading "pitch is
  clamped" text remains.

Checks run in the worktree: `cargo clippy --all-targets` clean (only the known
transitive proc-macro-error2 future-incompat note), `cargo fmt --check` clean,
`./scripts/check-ascii.sh` clean, `cargo test --examples` all green (14 bastion
unit tests pass).

Observable-effect verification (the follow-up retro's key lesson - verify the
control's effect, not a proxy) was performed and documented in the close-out: a
temporary log showed the pivot `forward_y` pinned at exactly 0.0000 across the
whole run while yaw swept a full 360 range under a held `D`. No new pure logic
was added, so no new unit test is warranted (ECS behavior is verified by run,
per the repo convention). Nothing to change.
