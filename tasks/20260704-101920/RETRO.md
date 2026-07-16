# Retro: share the Web Audio unlock shim across all web games

- TASK: 20260704-101920
- BRANCH: fix/mobile-web-audio
- REVIEW ROUNDS: 1 (APPROVE)

See tasks/20260704-101920/TASK.md for what changed and why. Process notes:

## What went well

- **Diagnosed drift by comparing, not guessing.** "Sounds don't work on mobile
  for all games" could have sent me hunting in the Rust audio plugin. Instead I
  diffed the three games' `index.html` shims against each other first; the diff
  showed 06_fruitninja carrying an iOS media-channel block that 07/08 lacked.
  The bug was duplication drift, and the diff said so in one step.
- **Read the prior retros before deciding the fix.** The 20260703-200005 retro
  had an open action item that literally specified this work and its intended
  path (`web/games/_shared/audio-unlock.js`). Following that instead of
  inventing a path kept the change aligned with prior intent and let me close
  that dangling item.
- **Verified the mechanism through the real entry point and the built output.**
  Ran `npm run build:games` (not a hand-rolled trunk invocation), then asserted
  in each *built* dist that `audio-unlock.js` was staged and its classic
  `<script src>` preceded trunk's deferred module loader -- the crate's
  "verify through the real entry point" discipline, which caught nothing broken
  this time precisely because it was applied.
- **Root-cause fix, not a copy-paste.** Copying 06's shim into 07/08 would have
  fixed the symptom and left three copies to drift again. Extracting to one
  shared file removes the failure mode that caused the bug.

## What went wrong

- **Hand-transcribed the base64 literal and corrupted it.** The first cut of the
  shared file reproduced the ~1.1 KB `SILENCE` data URI by typing it into the
  Write content; it came out 1291 chars instead of 1128 and failed to decode.
  Root cause: treated a long opaque literal as ordinary text to retype rather
  than data to copy programmatically. This is the *same family* as the
  20260703-212303 retro's "never reconstruct base64 by string surgery" -- second
  occurrence. Caught immediately by a decode + equality assertion against 06's
  literal, then fixed by substituting the exact string via a script. Cost: one
  extra step, no wrong result shipped.
- **A positional HTML tag check gave a false negative.** My first ordering check
  did `s.find('<script type="module"')`, which matched the string *inside my own
  explanatory comment* (the comment quotes those tags), reporting the shim as
  ordered after the loader. Root cause: searching HTML for tag substrings
  without stripping comments. Re-checked with comments stripped: correct.

## What to improve next time

- When copying a large opaque literal (base64, data URI, minified blob) between
  files, extract it programmatically from the source and assert equality /
  round-trip; never hand-transcribe it into a Write. (Reinforces the
  20260703-212303 lesson; now twice.)
- Before asserting HTML/XML structure by byte position, strip comments first --
  or match on the parsed element, not a substring -- so prose that quotes tag
  syntax cannot spoof the check.

## Action items

- [ ] AGENTS.md Gotchas: add a note -- "copying a large opaque literal (base64 /
      data URI / minified blob) between files: pull it programmatically and
      assert equality, never hand-retype it." Deferred to apply on master, not
      this branch: master's AGENTS.md was changed by a concurrent cycle
      (cbe034d) while this branch left AGENTS.md untouched, so editing it here
      would manufacture a merge conflict where there is currently none.
- [ ] On-device confirmation still pending: verify SFX audible on a real iPhone
      with the Ring/Silent switch on Silent, for 07_orbit and 08_dropzone. The
      change makes them byte-identical to 06's shim (already confirmed working
      on iOS), so this is last-mile confirmation, not a re-test of the approach.
- [x] Closed the 20260703-200005 retro's "lift the shim into a shared snippet"
      action item (done by this task).
