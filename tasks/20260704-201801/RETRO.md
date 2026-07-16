# Retro: route ui/popup + feedback onto Tween (realize foundation)

- TASK: 20260704-201801
- BRANCH: feat/tween-adoption (squash-merged to master as 286cae2)
- REVIEW ROUNDS: 1 (APPROVE, one informational NIT accepted)

The follow-up that made `tween` a real foundation: three shared, well-tested
modules (`ui/popup`, `feedback/flash`, `feedback/screen_flash`) refactored to
consume `Tween<f32>` instead of hand-rolling their own decay. A consistency
refactor of visual feedback used by four games -- the kind with real regression
risk and no new capability -- so the discipline was "behaviour identical, prove
it".

## What went well

- Kept the config components and builders (`Popup`, `ScreenFlash`, `Flash`, and
  their bundle builders) UNCHANGED, so the four games needed zero edits. Only the
  private state + animate internals changed. That is what made a three-module
  refactor land without touching the examples.
- Leaned on the existing tests as the safety net and treated them as the spec:
  every ECS test (rise/fade/despawn, spike, decay, persistent re-spike,
  clone-not-shared, restore-and-free, reflash-reuses-clone) had to keep passing
  with its exact-value assertions. Refactoring one module at a time and running
  its tests before moving on caught mistakes early and kept the blast radius
  small.
- Mapped each `Tween` completion policy to the module's real need rather than
  forcing one shape: popup = `Despawn`, screen_flash = `Despawn`/`Keep` by
  `despawn_on_end`, flash = `Keep` + an `On<Add, TweenFinished>` observer to do
  the material-restore side effect. The `decay == 0` "hold forever" case fell out
  neatly as an infinite-duration tween.
- Caught the one real integration risk by booting, not just testing: multiple
  fade plugins each guard-add `TweenPlugin`, and 06 also adds it explicitly for
  its slice pop. A double-add would panic; all four games booted clean because
  the `is_plugin_added` guards hold and 06's explicit add comes first.

## What went wrong

- Nothing broke, but I over-deliberated whether flash (material-clone lifecycle)
  was worth routing onto Tween before just trying it. The completion-as-observer
  pattern turned out clean (the k-ease is a `Tween<f32>`, the clone/restore stays
  in `FlashState` + the `TweenFinished` observer), and all its tests passed first
  try. The sketch anxiety was larger than the actual difficulty.

## What to improve next time

- For a "make X consume Y" consistency refactor across several modules, do them
  one at a time and gate each on its own tests before the next; the existing
  tests are the contract. It kept a 5-file, 3-module change to a single clean
  review round.
- When several plugins guard-add a shared dependency, boot a game that adds the
  most of them (plus any explicit add of the dep) -- the duplicate-plugin panic
  is a runtime-only failure a compile/test pass will not catch.

## Action items

- [x] `ui/popup`, `feedback/flash`, `feedback/screen_flash` now consume
  `tween::Tween`; the "foundation, not a leaf" claim is closed. Juice-kit Wave 2
  is fully realized.
- [ ] No non-dev-harness backlog remains that is mine. The dev-harness spike's
  Wave 2 (tatr 20260704-175422-425) stays the parallel session's; the two
  needs-user PROPOSAL tasks (particles, real audio) await a user call.
