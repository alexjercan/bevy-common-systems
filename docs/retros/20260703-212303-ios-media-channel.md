# Retro: iOS Safari media-channel audio fix

- TASK: 20260703-212303
- BRANCH: fix/ios-media-channel
- REVIEW ROUNDS: 1 (APPROVE)

See tasks/20260703-212303/TASK.md for what changed and why. Process notes:

## What went well

- **Let the new symptom re-open a "closed" diagnosis.** The prior task fixed
  "context suspended" and I could have insisted it was done. The user's "mute
  this tab shows but no sound" was the tell that audio was already playing --
  a different problem (ringer channel), not a regression. Reading the symptom
  literally pointed straight at WebKit bug 237322.
- **Grounded the fix in sources, not memory.** WebAudio-on-iOS is exactly the
  kind of thing that is easy to misremember; a quick search (WebKit bug 237322,
  swevans/unmute, feross/unmute-ios-audio) confirmed the media-channel `<audio>`
  technique before writing code.
- **Asserted the risky artifact.** The embedded silent-WAV base64 was verified
  to decode to a valid all-zero RIFF/WAVE in both source and the built dist,
  not eyeballed.

## What went wrong

- **Hand-optimized base64 across a header/data boundary.** First cut built the
  data URI as a fixed prefix + `'A'.repeat(n)` + `==`, assuming the 44-byte WAV
  header ended on a base64 char boundary. It does not (44 is not a multiple of
  3), so the string was wrong (1566 vs 1128 chars). Root cause: treated a
  base64 body as if byte boundaries map to char boundaries. Fixed by generating
  and embedding the exact literal with a round-trip assertion.
- **Tried to merge into a master that was moving under me.** The main checkout
  is shared with the user, who was actively committing (master went
  f663dff -> 75090f0, new task dirs appearing) while I worked. `git merge` hit
  an add/add conflict on the task file and I aborted. Root cause: assumed the
  local default branch was stable, as in the earlier solo tasks this session.
  It was not, because a human was working in the same working tree concurrently.

## What to improve next time

- Never reconstruct base64 by string surgery; emit the whole literal from the
  encoder and assert `decode(embedded) == original`.
- Before merging a branch into the local default branch, check the branch is
  still a clean descendant (`git merge-base --is-ancestor` / compare HEAD to the
  value seen at branch creation). If the default branch has moved due to
  concurrent human activity in the SAME checkout, do NOT force the merge --
  leave the reviewed branch and hand off, since two actors running git in one
  working tree can corrupt the index.

## Action items

- [x] docs/wasm-web-builds.md: document the ringer-vs-media channel and the
      silent-<audio> workaround (done in this task).
- [ ] Branch fix/ios-media-channel is reviewed APPROVE but intentionally NOT
      merged (concurrent master activity + needs a real-device check). User to
      merge when the tree is stable and the iPhone test passes.
- [x] memory: recorded the shared-checkout concurrent-git hazard.
