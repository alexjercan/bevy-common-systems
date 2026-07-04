# Retro: spawn + time/cooldown - ship Cooldown, drop spawn

- TASK: 20260704-134730
- BRANCH: feat/spawn-cooldown (squash-merged to master as 8846d0f)
- REVIEW ROUNDS: 1 (APPROVE, no findings)

The last juice-kit Wave 2 item, and a clean two-way sketch-then-commit: one of
the two proposed helpers earned its keep, the other did not.

## What went well

- Sketched both against the actual game code before writing either, and let the
  evidence split them. The gate was "beats a raw `Timer`", and the answer was
  different for each: 06's spawner is a plain `Repeating` `Timer` with
  `set_duration` for the ramp, and no game uses temporal jitter -- so `spawn` is
  a thin wrapper and became a documented recipe. But `Cooldown` has a concrete
  win a `Timer` gets backwards: starts-ready. A fresh `Timer` in `Once` mode is
  not finished, so a weapon built on one spawns unable to fire; a fresh
  `Cooldown` is `ready()`. That footgun, plus deduping 10's two hand-rolled f32
  cooldowns, is what tipped it over the gate.
- Named the two constructors for the two real cases the code showed:
  `Cooldown::new` (starts ready, for fire) and `Cooldown::started` (starts on
  cooldown, for i-frames at spawn). Both fell straight out of reading 10's spawn
  code (`fire_cooldown: 0.0` vs `invuln: INVULN_TIME`), not invented up front.
- Verified the gate end-to-end, not by boot alone: the autopilot held Space every
  frame and the ship fired ~13 rate-limited shots over 3s (FIRE_COOLDOWN 0.16
  caps near 6/s; an ungated fire would be ~180). That is the difference between
  "it compiles" and "the cooldown actually gates".

## What went wrong

- Nothing of substance. `Cooldown` is admittedly close to the gate line (a
  `Timer(Once)` can be coerced into a cooldown), but the starts-ready ergonomics
  and the two-copy dedup are a real, if modest, win -- and the review agreed.

## What to improve next time

- Keep letting a "build two small helpers" task split on the evidence rather than
  building both or dropping both. The honest read here (ship one, document one)
  is the same discipline as the earlier scoring/radial-gravity/progress calls;
  the tell is whether the helper fixes something the raw primitive gets *wrong*
  (here: the Timer starts-not-ready footgun), not merely wraps it.

## Action items

- [x] `time::Cooldown` shipped; 10_asteroids refactored onto it; `spawn`
  documented as a `Timer` recipe. Juice-kit Wave 2 is complete
  (`tween`, `persist`, `time/cooldown` shipped; `spawn` a recipe).
- [ ] Remaining non-dev-harness work: the tween follow-up
  (tatr 20260704-201801, route ui/popup + feedback onto `Tween`). The dev-harness
  spike's Wave 2 (175422-425) stays the parallel session's.
