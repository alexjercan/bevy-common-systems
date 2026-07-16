# Retro: tween - narrow duration-based Tween<T>

- TASK: 20260704-134630
- BRANCH: feat/tween (squash-merged to master as a7ca40f)
- REVIEW ROUNDS: 1 (APPROVE, one MINOR -> follow-up, one NIT accepted)

The juice-kit Wave 2 foundation: generalize `LerpSnap` into a real tween. The
first Wave 2 item to actually ship a substantial module (scoring shipped a small
one; radial-gravity and progress were recipes), so it carried the most design
weight of any recent task.

## What went well

- Resolved the design fork by picking the crate's OWN pattern over the obvious
  ones. The instinct for a tween is either a lens/closure system (bevy_tweening)
  or per-target adapter components -- both of which hit the
  `Tween<Vec3>`-for-scale-vs-translation collision or drag in framework
  machinery. Looking at how `transform/*` already works (compute an Output, the
  game applies it) gave a third option that is narrower than both: `Tween<T>`
  owns timing/easing/completion and exposes `value()`, the game writes it
  wherever. No lens, no adapters, no collision. Matching an existing convention
  beat inventing a new surface.
- Caught a real bug by reading my own guard: a zero-duration tween is `finished`
  from frame zero, so `if finished { continue }` would have skipped its
  completion policy forever -- silently stranding the entity. The `completed`
  flag fixes it, and the three headless policy tests (Keep/Remove/Despawn, driven
  through the plugin) exist specifically because a zero-duration tween completes
  deterministically in one `update()`, so I got real ECS coverage without a
  controllable clock.
- Verified the gameplay refactor with the autopilot (last cycle's lesson),
  pre-building `--features debug` first after the cold compile ate a 60s timeout.
  Clean `Menu -> Playing -> GameOver, no panic`.

## What went wrong

- Over-deliberated the design for a long time (component vs pure-struct vs
  adapters vs lens, and the `meth`-tag-vs-top-level-module home) before writing
  code. The tie-breakers were all already on disk -- the `transform/*` Output
  pattern and the "meth is pure, runtime behaviour gets its own plugin module"
  convention -- so the decision could have been made in one pass by consulting
  the crate, not from first principles.
- Could not inject a live fruit-slice headlessly (needs a cursor-swipe with
  motion, not just a held button), so the slice-pop -> burst path was verified by
  construction + the completion unit tests rather than an end-to-end slice. Fine
  here (behaviour-identical, mechanism tested), but noted honestly in the review.

## What to improve next time

- For a design fork on THIS crate, check the existing modules for a precedent
  before weighing abstract options. The Output pattern and the
  pure-vs-plugin-module split are established; reusing them is both faster to
  decide and more consistent than reasoning a design from scratch.
- When a headless end-to-end path needs input *motion* (a swipe, a drag), the
  autopilot's held-button trick is not enough; either accept mechanism-level test
  coverage (as here) or build a motion-injection helper -- do not pretend a
  held-button boot exercised the swipe.

## Action items

- [x] `tween::Tween<T>` shipped; 06 slice pop refactored onto it. Zero-duration
  completion bug fixed and tested.
- [ ] Follow-up tatr 20260704-201801: route `ui/popup` fade and `feedback` decay
  onto `Tween` to realize the "foundation, not a leaf" claim (review MINOR R1.1).
- [ ] Remaining juice-kit Wave 2: `persist` (tatr 20260704-134700), `spawn +
  time/cooldown` (tatr 20260704-134730), both sketch-then-commit. Dev-harness
  spike's Wave 2 (175422-425) stays the parallel session's.
