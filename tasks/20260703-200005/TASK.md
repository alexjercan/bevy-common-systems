# Fix mobile Safari silent audio in 06_fruitninja web build

- STATUS: OPEN
- PRIORITY: 80
- TAGS: web,audio,wasm,safari

## Goal

`06_fruitninja` plays no sound on mobile Safari (iOS/iPadOS WebKit). Make SFX
audible there after the first in-canvas gesture, without regressing desktop
Chrome/Firefox (which already work).

Done = the game's web build resumes Bevy's suspended AudioContext on the first
user gesture inside the canvas, so `menu_select` and every later sound play on
mobile Safari; desktop audio unchanged; the mechanism documented.

## Background / root cause

Bevy/cpal creates its `AudioContext` eagerly at startup, before any user
gesture, so it comes up `suspended`. Chrome and Firefox auto-resume a suspended
context once the user has interacted AND a source node `start()`s (which
rodio/cpal do per sound) -- so the crate shipped no unlock code (retro
20260703-163329 documents that decision, correctly, for Chrome/Firefox only).

WebKit (macOS + iOS Safari) does NOT auto-resume on `start()`. It requires an
explicit `AudioContext.resume()` called synchronously inside a user-gesture
handler. Bevy never calls it and does not expose its context, so on Safari the
context stays suspended and every SFX is silent. This is the reported bug.

The fallback was already anticipated in docs/wasm-web-builds.md ("resume the
context from JS in the host HTML on a canvas pointerdown"). This task
implements that fallback.

## Approach

Add a small inline JS shim to `web/games/06_fruitninja/index.html`, placed
BEFORE trunk's injected wasm loader so it runs first:

- Monkeypatch `window.AudioContext` and `window.webkitAudioContext` so every
  context Bevy/cpal constructs is captured into a module-scoped array. (Bevy
  does not expose its context; capturing at construction is the only handle.)
- On the first `pointerdown` / `touchend` / `keydown` / `mousedown` on the
  document, call `resume()` on every captured context and play a 1-sample
  silent buffer through each (WebKit's stricter unlock wants an actual node
  start inside the gesture). Register the listeners self-removing so it runs
  once.
- No-op on desktop: `resume()` on a running context is harmless; the silent
  buffer is inaudible.

Consider factoring the shim into a shared snippet so future web games reuse it,
but a single inline `<script>` in this game's index.html is acceptable for now
(only one game has sound). Decide during work, keep it simple.

## Steps

- [ ] Add the audio-unlock JS shim to `web/games/06_fruitninja/index.html`
      (monkeypatch AudioContext constructors + resume-on-first-gesture +
      silent-buffer kick). Plain ASCII only; keep it small and commented.
- [ ] Build the real web bundle (`cd web && npm run build`, or
      `PUBLIC_PATH=/ bash web/scripts/build-games.sh`) and confirm it compiles
      and the shim lands in the built `index.html`. Redirect build output to a
      file and check `$?` (never judge by a piped tail).
- [ ] Update docs/wasm-web-builds.md "Audio and the autoplay policy": correct
      the Chrome/Firefox-only claim, document the WebKit resume requirement and
      the shim now in place.
- [ ] Run the native check suite (fmt, clippy x2, test x2, check-ascii) to
      confirm no Rust regression (web-only change, but keep CI green).

## Verification / honest limits

- Headless box: cannot hear audio or run mobile Safari. Verify what IS testable
  -- the bundle builds, the shim is present in the emitted index.html, HTML is
  well-formed, native suite green.
- Hand off the aural check: user opens the web build on an iOS/Safari device,
  taps to start, confirms slice/combo/bomb sounds play. Desktop Chrome/Firefox
  regression check: sounds still play, no console errors.

## Notes

- Do NOT try to resume from Rust; Bevy 0.18 does not expose the AudioContext.
- The iframe already carries `allow="autoplay"`; necessary but not sufficient
  on WebKit -- the explicit resume is the missing piece.
- Related: retro 20260703-163329 (the Chrome/Firefox no-code decision this
  task extends), docs/2026-07-03-audio-and-fruitninja-sounds.md.
