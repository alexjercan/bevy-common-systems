# Review: iOS Safari media-channel fix (silent <audio>)

- TASK: 20260703-212303
- BRANCH: fix/ios-media-channel

## Round 1

- VERDICT: APPROVE

Reviewed the diff against master: the shim extension in
`web/games/06_fruitninja/index.html`, the docs addition, and the closed
TASK.md. Verified independently:

- **Diagnosis is right and sourced.** "mute this tab" indicator + inaudible =
  audio produced but on the ringer channel (WebKit bug 237322). The `<audio>`
  media-channel promotion is the established fix (unmute libraries), not a
  guess. The existing WebAudio kick could not have fixed this (wrong channel).
- **Base64 is correct.** Regenerated and confirmed the embedded literal decodes
  to a valid 844-byte all-zero RIFF/WAVE, in both source and the built `dist`
  HTML. The earlier `'A'.repeat()` bug was caught and replaced with the exact
  literal.
- **iOS gating is sound.** `/iP(hone|ad|od)/` plus `Mac` + `maxTouchPoints > 1`
  catches iPhone, iPod, iPad, and iPadOS-reporting-as-Mac; iOS Chrome/Firefox
  (CriOS/FxiOS) still match via "iPhone"/"iPad" and get the fix, which is
  correct since they share the WebKit ringer behavior. Desktop is excluded, so
  no spurious "now playing" widget there.
- **Gesture + lifecycle correct.** `silent.play()` runs inside the existing
  gesture handler (iOS requires that); the promise rejection is swallowed;
  `visibilitychange` pauses/resumes. On iOS the resume listeners are
  intentionally kept attached so an interrupted silent track can be re-armed.
- **Checks:** `npm run build` green with `createElement('audio')` + the data
  URI in the shipped dist; native suite green (fmt/clippy/example-test/ascii).
  No test weakened. Docs now explain both halves of the iOS story.

Non-blocking:

- [ ] R1.1 (MINOR) index.html - on-device audibility unverified (headless box,
  no iOS). User must confirm on a real iPhone with the Ring/Silent switch on
  SILENT and media volume up. Tracked as the task hand-off, not a code defect.
  - Response:
- [ ] R1.2 (NIT) index.html:105-110 - once a context is `running`, the 1-sample
  WebAudio kick still runs on every subsequent gesture on iOS (listeners stay
  attached there), a tiny per-swipe allocation. Could guard the kick behind
  `ctx.state === 'suspended'`. Harmless; left as-is to keep the diff minimal.
  - Response:
- [ ] R1.3 (NIT) index.html - the looping silent <audio> keeps Safari's tab
  audio indicator lit for the whole session. Inherent to the media-channel
  technique (unmute has the same trade-off); no fix, noted for expectations.
  - Response:

No BLOCKER/MAJOR. APPROVE.
