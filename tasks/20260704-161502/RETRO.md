# Retro: camera/project screen<->world projection helpers

- TASK: 20260704-161502
- BRANCH: feat/camera-project (squash-merged to master as 0ca8d13)
- REVIEW ROUNDS: 1 (APPROVE, one NIT addressed)

Another clean harvest cycle in the mold of the `ui/popup` and `camera/shake`
port cycles -- short retro by design. This was Wave A item 1 of the
input-and-projection harvest spike
(`tasks/20260704-161210/SPIKE.md`).

## What went well

- Read the spike doc and both `pointer_on_play_plane` copies before writing a
  line, so the helper signature was right the first time. The one non-obvious
  call -- `InfinitePlane3d` carries only a normal, so a single `plane` arg cannot
  reproduce both games' plane offsets -- was caught up front and split into
  `plane_origin: Vec3` + `plane: InfinitePlane3d`, preserving 06's `PLAY_Z`
  offset and 10's `Vec3::ZERO` exactly. No rework, no revert commits.
- Reproduced the duplicated bodies byte-for-byte inside the helpers, so the
  refactor was provably behavior-preserving; the four-example refactor is the
  integration test (the crate convention for camera-coupled code that cannot be
  pure-unit-tested without a live render target).
- Applied the crate's "leave the game-specific config in the game" line
  deliberately: 06 kept a thin `pointer_on_play_plane` wrapper (2 call sites,
  pins the play plane) while 10's single-call copy was inlined and deleted. The
  reviewer accepted the asymmetry as justified rather than flagging it.
- The one NIT was the spike's own stated payoff -- repointing `ui/popup`'s doc
  at the new blessed `world_to_screen` -- so addressing it (a one-line intra-doc
  link) realized the "unblocks ui/popup world-entity tracking" goal instead of
  leaving it as a loose end.

## What went wrong

- Nothing of substance. Review found no BLOCKER/MAJOR/MINOR, only the one NIT,
  which was a doc pointer rather than a defect.

## What to improve next time

- Keep doing this for byte-for-byte harvests: read every copy first, reproduce
  the body verbatim in the helper, refactor the call sites as the test, and
  sweep the docs that referenced the old hand-rolled call (grep the raw API name
  -- here `world_to_viewport` -- across `src/` too, not just the examples; that
  is what surfaced the `ui/popup` doc NIT). The recipe is now proven across
  three harvest cycles.

## Action items

- [x] `camera/project` shipped; `ui/popup` doc now points at `world_to_screen`,
  closing the spike's "popup world-entity tracking" open question.
- [ ] Wave A continues: `input/pointer` unified pointer resource
  (tatr 20260704-161508) and `ui/touchpad` reveal-on-first-touch + hit-test
  primitives (tatr 20260704-161513). `input/pointer` is the natural next pick
  -- three independent copies, and its `screen_pos` feeds straight into this
  task's `pointer_on_plane`.
