# Retro: verify web audio playback and the autoplay policy

- TASK: 20260703-163329 (CLOSED, review APPROVE round 1)
- BRANCH: feature/web-audio
- SCOPE: confirm 06_fruitninja's SFX play in the browser and handle the
  autoplay/user-gesture policy; add unlock code only if needed.

## What went well

- **Resisted speculative code.** The task was framed to add a reusable unlock
  "only if needed." Grounded research (MDN, Chrome autoplay docs, cpal PR #774,
  bevy#15273) showed the browser auto-resumes the suspended AudioContext once
  the user interacts and a source node `start()`s, and that the game's first
  sound already fires on an in-canvas gesture -- so no code was warranted. It
  also turned out Bevy does not expose its `AudioContext` (no clean Rust resume
  anyway) and the iframe already had `allow="autoplay"`. Adding a plugin would
  have been net-negative framework machinery. The right answer to "add a
  capability" was sometimes "the capability already exists; document it."
- **Grounded, not remembered.** Browser autoplay behavior is exactly the kind
  of thing that is easy to misremember; a research agent with real sources
  (cited in the Outcome) made the no-code decision defensible rather than a
  guess.

## What was tricky / honest limits

- **Headless environment, no audio device.** The Goal wants evidence that SFX
  are *audible*, which cannot be produced without a graphical session with
  sound. The honest move was to (a) verify the half that IS testable headlessly
  -- serve `dist/` and confirm every sound URL returns HTTP 200 at the exact
  path Bevy fetches -- and (b) hand off the aural check to the user with exact,
  runnable steps, explicitly NOT claiming to have heard it. The review tracked
  this as the one open user action (MINOR), not a fake pass.

## Notes for future sessions

- When a task says "add a capability," first check whether the platform / an
  existing config already provides it. Here `allow="autoplay"` was already on
  the iframe and the browser auto-resume already covered the gesture case; the
  deliverable collapsed to documentation.
- For any "does it work in a browser" task on this headless box: the network /
  loading half is verifiable (static server + curl for HTTP 200 and
  content-type); the audible / visual half is not. Split the claim accordingly
  and hand off the rest -- do not let "I couldn't test it here" masquerade as
  "it works" or as a blocker.
- Autoplay reference now lives in docs/wasm-web-builds.md ("Audio and the
  autoplay policy") so the next web game with sound does not re-research it.
