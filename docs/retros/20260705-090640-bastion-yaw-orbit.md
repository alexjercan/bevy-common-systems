# Retro: 12_bastion yaw-only orbit camera

- TASK: 20260705-085334
- BRANCH: feature/bastion-yaw-orbit (squash-merged to master)
- REVIEW ROUNDS: 1 (APPROVE, no findings)

First task of the 12_bastion polish flow (camera / packs / buttons / juice).
Small, focused change: make A/D + drag a pure yaw orbit with fixed pitch.

## What went well

- Read the follow-up section of the previous bastion retro
  (20260704-220736) FIRST and it paid off immediately: it named the two exact
  camera bugs a past cycle shipped (missing `out.0 -> transform.rotation` copy;
  a state-gated driver spinning on its stale last delta) and the "verify the
  control's observable effect, not a proxy" lesson. Both were honoured up front
  rather than rediscovered.
- Verified by observable effect, not a proxy: a temporary log proved
  `forward_y` (pitch) stayed pinned at 0.0000 for the whole autopilot run while
  yaw swept a full 360 range under a held `D`. That is the precise mistake the
  prior cycle made (pressed `D`, never confirmed the view moved); this time the
  camera pose was actually measured frame to frame.
- The change stayed minimal: removing pitch let the whole clamp block and its
  two constants go, so the diff is a net simplification, not an addition.

## What went wrong

- One wasted run: the first verification piped `cargo run ... | grep > file`
  and captured nothing (empty log), so I could not tell if the app ran. Root
  cause: filtering a build+run pipeline in one shot hides whether the miss was
  "no matches" or "run failed". Re-running with a full-capture to a file, then
  grepping the file, immediately showed 85 matching lines. Lesson: capture the
  full run to a file first, then grep the file - do not filter inline when you
  still need to know the run happened at all. (Mirrors the AGENTS "never judge a
  build by a piped tail" gotcha.)

## What to improve next time

- For headless verification, default to `cargo run ... > full.log 2>&1` then
  grep `full.log`. It costs one file and removes the "was it no-match or
  no-run" ambiguity in a single step.
