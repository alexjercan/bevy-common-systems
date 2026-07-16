# Retro: feedback material hit-flash module

- TASK: 20260704-134600
- BRANCH: feat/feedback-flash (squash-merged to master as f7bf379)
- REVIEW ROUNDS: 2 (APPROVE round 1 with 4 MINORs, APPROVE round 2)

See TASK.md and `tasks/20260704-134600/NOTES.md` for what changed.

## What went well

- Caught the task's false premise *before* writing code. The spike/task claim
  ("06/07/10 hand-roll a material flash") did not match reality -- reading the
  code, those games hand-roll a full-screen UI overlay, not a material flash.
  Because I verified the motivation against the actual code first, I surfaced the
  discrepancy and asked the user which module to build (AskUserQuestion) instead
  of either silently building the wrong thing or force-fitting a "refactor" onto
  code that had nothing to refactor. This is exactly the flow "stop and ask when
  the goal means something different than assumed" trigger, and it paid off.
- Reasoned about the demo target before coding: a rock despawns the instant it
  is hit, so flashing it would never render -- the ship (survives via i-frames)
  is the only persistent target. Picking that up front avoided a dead-end demo.
- The leak-free design got real tests for the hard paths: shared-material
  isolation (bystander untouched), restore+free on completion, and free-on-
  despawn. The `On<Remove, FlashState>` observer covering both completion and
  despawn is a clean single mechanism.
- Handled a mid-cycle concurrent master move (two commits landed from other work
  during the cycle) without incident -- branch from HEAD, check ancestry, merge.

## What went wrong

- Review found 4 MINORs, two of which I should have caught myself:
  - `FlashState` not `register_type`'d, while `CameraShakeState` and `PopupState`
    both are. This is a pure convention-consistency miss -- and it is precisely
    the "match the siblings" check. A quick `grep register_type src/**/` against
    the private-state pattern would have caught it.
  - Re-flash did not restart the animation. I actually *noticed* this edge while
    reasoning (I concluded "not a leak, acceptable for v1") and then left it
    untested and unfixed. The reviewer rightly pushed to make the primitive
    correct (switch to `On<Insert>`, reset elapsed). Root cause: I treated a
    known edge case as "acceptable" without either testing it or documenting the
    limitation -- the soft version of the "test the claim you wrote" gap.

## What to improve next time

- When I catch myself thinking "edge case X is acceptable for v1", treat that as
  a decision that must be *recorded*: either write a test that pins the intended
  behavior, or a doc sentence stating the limitation. Silently leaving a noticed
  edge is what the reviewer catches every time.
- For a module that mirrors an existing sibling pattern (Config/State/Plugin),
  diff it against the sibling before review: `register_type` of the private
  state, prelude wiring, `*Systems` set, observer setup. The misses are always
  the small consistency items, not the core logic.

## Action items

- [x] Fixed all 4 MINORs in review round 2 (On<Insert> re-pop + test, register
  FlashState, drop stray Flash + test, doc the mid-flash material-swap caveat).
- [ ] tatr 20260704-155505: promote the full-screen damage OVERLAY 06/07/10
  actually duplicate into `feedback/screen_flash` (the real dedup this task's
  premise was pointing at).
- [ ] Wave-2 kit tasks remain (tween 20260704-134630, persist 20260704-134700,
  spawn/cooldown 20260704-134730) plus the two open ui/popup + feedback ports.
