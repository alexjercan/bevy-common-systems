# iOS Safari: route WebAudio to media channel so mute switch does not silence SFX

- STATUS: CLOSED
- PRIORITY: 85
- TAGS: web,audio,wasm,safari,ios
- Depends on: 20260703-200005 (the AudioContext unlock shim this extends)

## Goal

After the unlock shim (task 20260703-200005), mobile Safari shows a "mute this
tab" indicator -- audio IS being produced -- but it is still inaudible with the
iPhone Ring/Silent switch on Silent. Make 06_fruitninja SFX audible on iOS
regardless of the hardware mute switch.

Done = with the Ring/Silent switch on Silent and media volume up, slicing a
fruit is audible on iOS Safari; desktop unchanged.

## Background / root cause

WebKit bug 237322: iOS sends Web Audio API output to the RINGER channel, which
the physical mute switch silences even at full media volume. HTML5 <audio>
elements go to the MEDIA/playback channel, which ignores the switch. The
existing shim only plays a silent WebAudio *buffer* to resume the context; that
unlocks playback but does not move it off the ringer channel.

The established workaround (swevans/unmute, feross/unmute-ios-audio): on the
first user gesture, start a CONTINUOUS looping silent HTML <audio> element.
While a real <audio> element is playing, iOS promotes the audio session to the
media channel, so Web Audio then also plays through media and ignores the mute
switch.

## Approach

Extend the inline shim in web/games/06_fruitninja/index.html:

- Add a hidden looping <audio> with a tiny silent WAV data URI, `loop`,
  `playsinline`, `preload="auto"`. Generate the silent WAV deterministically
  (mono, a few ms of zero samples) and embed as base64 -- keep it small.
- In the existing gesture `unlock()` (already runs inside a user gesture, which
  iOS requires for <audio>.play()), call `silent.play()` and swallow the
  promise rejection. Keep the existing AudioContext resume + WebAudio kick.
- Lifecycle: on `visibilitychange`, pause the silent track when hidden and
  resume it (best-effort) when visible, so iOS does not show a persistent media
  widget / waste battery in the background (matches unmute's behavior).
- Only needed on iOS/WebKit but harmless elsewhere; gate lightly (a silent
  looping audio element is inaudible everywhere) rather than sniffing UA, or do
  a minimal WebKit check -- decide during work, prefer simplest correct.

## Steps

- [x] Generate a small silent WAV and embed it as a base64 data URI in the
      shim (document how it was generated, e.g. scripts or an inline note).
- [x] Add the looping silent <audio> element + `silent.play()` inside the
      existing gesture handler; add the visibilitychange pause/resume.
- [x] Rebuild the web bundle (`cd web && npm run build`), confirm exit 0 and
      the <audio>/base64 lands in the emitted dist index.html (redirect output
      to a file, check `$?`).
- [x] Update docs/wasm-web-builds.md "Audio and the autoplay policy": add the
      ringer-vs-media-channel note (WebKit bug 237322) and that the shim now
      also plays a silent <audio> to force the media channel.
- [x] Native check suite green (web-only change, but keep CI green).

## Verification / honest limits

- Headless box: cannot hear audio or run iOS Safari. Verify the bundle builds
  and the silent <audio> + base64 is present in the emitted HTML.
- Hand off: user tests on a real iPhone with the Ring/Silent switch on SILENT
  and media volume up -- slicing a fruit should now be audible. Confirm the
  10-second pre-fix test first (flipping the switch to Ring makes sound), which
  confirms the diagnosis before the fix is even needed.

## Notes

- References: WebKit bug 237322; github.com/swevans/unmute;
  github.com/feross/unmute-ios-audio.
- This is the second half of the mobile-Safari audio story: 20260703-200005
  fixed "context suspended" (Chrome/FF + the resume path); this fixes "audible
  on iOS despite the mute switch".

## Outcome (CLOSED)

Extended the inline shim in `web/games/06_fruitninja/index.html` (from task
20260703-200005) with the iOS media-channel workaround. On iOS only (UA +
`navigator.maxTouchPoints > 1` to catch iPadOS-as-Mac), the first gesture now
also starts a continuous looping, inaudible `<audio>` element sourced from a
tiny base64 silent-WAV data URI. While a real `<audio>` element plays, iOS
promotes the whole audio session (Web Audio included) to the media channel,
which the physical Ring/Silent switch does not mute -- fixing WebKit bug 237322.
A `visibilitychange` handler pauses the track when the tab is hidden and
resumes it on return. On iOS the resume/kick listeners are kept attached (not
detached) so an interrupted silent track can be re-armed by a later gesture.

The silent WAV is mono, 8 kHz, 16-bit PCM, ~50 ms of zero samples (844 bytes),
embedded as a `data:audio/wav;base64,...` literal. It was generated
deterministically and the embedded literal was verified to decode back to a
valid all-zero RIFF/WAVE both in source and in the built `dist` HTML.

### Why this shape

- HTML `<audio>`, not a WebAudio buffer: only a real media element moves iOS off
  the ringer channel. The existing 1-sample WebAudio kick (kept, for the
  context-resume half) does not.
- Gated to iOS: a looping silent `<audio>` on desktop would add a spurious OS
  "now playing" widget for no benefit; the ringer switch is iOS-only.
- Inline data URI, not a shipped file: keeps the shim self-contained (no extra
  trunk copy-dir, no fetch that could 404) and it is only ~1.1 KB of base64.

### Difficulties

- First attempt built the base64 with a `'A'.repeat(n)` shortcut, assuming the
  header/data boundary fell on a base64 char boundary. It does not (the 44-byte
  header is not a multiple of 3), so the reconstruction was wrong (1566 vs 1128
  chars). Fixed by generating and embedding the exact full literal via script
  and asserting it round-trips to an all-zero RIFF/WAVE -- in both the source
  and the emitted dist HTML.

### Verification

- `npm run build` (trunk + webpack) exit 0; `createElement('audio')` and the
  base64 data URI both present in `web/dist/games/06_fruitninja/index.html`;
  embedded base64 decodes to a valid 844-byte silent WAV in the dist.
- Native suite green: fmt=0, clippy=0, test --example 06_fruitninja=0 (19),
  check-ascii=0.

### Honest limit (hand-off)

Not verifiable on this headless box (no iOS device). User check: on an iPhone
with the Ring/Silent switch on SILENT and media volume up, slicing a fruit
should now be audible. Pre-fix sanity check that confirms the diagnosis:
flipping the switch to Ring made sound play even before this fix.

### Self-reflection

- Do not hand-optimize base64 across a header/data boundary; generate the exact
  literal and assert the round-trip. The shortcut looked obviously-right and was
  wrong.
- The two-part nature of iOS web audio (suspended-context AND ringer-channel) is
  now fully documented in docs/wasm-web-builds.md so a future web game does not
  rediscover only half of it (as task 20260703-200005 did).
