# Retro: assets SoundBank registry + opt-in loaded gate

- TASK: 20260704-175422
- BRANCH: feat/assets-registry (squash-merged to master as b962667)
- REVIEW ROUNDS: 1 (APPROVE, two NITs; one seeded a follow-up)

The first dev-harness-spike Wave 2 task, taken after the user lifted the
"leave it to the parallel session" boundary. A clean leaf harvest -- replace the
`SfxAssets` handle-bag six games copy with a keyed registry -- with one real
cross-cut concern (async asset readiness).

## What went well

- Let the evidence pick the design. The sound *set* differs per game (06 has
  slice/splat/combo, 09 has tap/buy/tier_up), so a fixed enum could not work;
  `SoundBank<K>` generic over the game's key enum was the only shape that fits
  all six. The `SoundKey` blanket trait means the game just derives the bounds
  and never names it.
- Found the thing that makes it more than a lateral struct->map move: the
  opt-in loading gate (`all_loaded` / `sounds_loaded::<K>`). No game has one
  today, and async handle readiness (especially on wasm) is exactly the plumbing
  a utility crate owns once. Keeping it a run-condition, not a forced plugin,
  honoured the spike's own answer.
- 09 turned out to be the better second example than a same-shaped game: it
  proves the semantic key decouples from the file (`Tap -> pickup.wav`,
  `Buy -> golden.wav`), which the struct did by field name and the bank does by
  the `(key, name)` pair. Picking a game that stresses the design beats picking
  an easy one.
- Guarded the one runtime regression (`get()` panics on a missing key, where the
  struct was a compile error) by statically diffing each game's `get()` keys
  against its load list before booting -- so the panic path is provably
  unreachable in 06/09.

## What went wrong

- Nothing broke, but the merge-back surfaced the shared-checkout hazard: while I
  worked, a parallel session advanced `master` by four commits (a game-ideas
  spike + a 12_bastion plan). Caught it at merge time because the branch's base
  was no longer an ancestor of `master`; the four commits were docs/tasks only
  (zero overlap with my audio/06/09 files), so `git merge master` into the branch
  was conflict-free, re-verified green, then squash-merged. The memory note about
  not force-merging a moving master is exactly why I checked `is-ancestor` before
  the squash rather than assuming a fast-forward.

## What to improve next time

- On a shared checkout, always re-fetch and check `git merge-base --is-ancestor
  master <branch>` right before the squash-merge, not just at sprout time --
  `master` can move under you, and the fix (merge master into the branch,
  re-verify, then squash) is cheap when the changes are disjoint and a real
  conflict-resolution when they are not.
- For a "replace the copied thing" harvest, pick the second proof example to
  stress the new design's degrees of freedom (here: key/file decoupling), not to
  be the easiest port.

## Action items

- [x] `audio::SoundBank<K>` + `sounds_loaded` gate shipped; 06 and 09 refactored.
- [ ] Follow-up tatr 20260704-223846: migrate 07/08/10/11 onto the registry
  (review NIT R1.1), optionally demoing the gate in one game (R1.2).
- [ ] Remaining dev-harness Wave 2: `HighScore<T>` (175423), `ui/menu` builders
  (175424), input `AnyStartPress` + leaf helpers (175425). Coordinate with the
  parallel session (it is actively committing to master) before taking more.
