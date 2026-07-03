# iOS Safari: route WebAudio to media channel so mute switch does not silence SFX

- STATUS: OPEN
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

- [ ] Generate a small silent WAV and embed it as a base64 data URI in the
      shim (document how it was generated, e.g. scripts or an inline note).
- [ ] Add the looping silent <audio> element + `silent.play()` inside the
      existing gesture handler; add the visibilitychange pause/resume.
- [ ] Rebuild the web bundle (`cd web && npm run build`), confirm exit 0 and
      the <audio>/base64 lands in the emitted dist index.html (redirect output
      to a file, check `$?`).
- [ ] Update docs/wasm-web-builds.md "Audio and the autoplay policy": add the
      ringer-vs-media-channel note (WebKit bug 237322) and that the shim now
      also plays a silent <audio> to force the media channel.
- [ ] Native check suite green (web-only change, but keep CI green).

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
