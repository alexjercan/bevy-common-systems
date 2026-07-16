# Fix mobile Safari silent audio in 06_fruitninja web build

- STATUS: CLOSED
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

- [x] Add the audio-unlock JS shim to `web/games/06_fruitninja/index.html`
      (monkeypatch AudioContext constructors + resume-on-first-gesture +
      silent-buffer kick). Plain ASCII only; keep it small and commented.
- [x] Build the real web bundle (`cd web && npm run build`) and confirm it
      compiles and the shim lands in the built `index.html`. Redirect build
      output to a file and check `$?` (never judge by a piped tail).
- [x] Update docs/wasm-web-builds.md "Audio and the autoplay policy": correct
      the Chrome/Firefox-only claim, document the WebKit resume requirement and
      the shim now in place.
- [x] Run the native check suite (fmt, clippy x2, test x2, check-ascii) to
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
  task extends), tasks/20260703-152544/NOTES.md.

## Outcome (CLOSED)

Added an inline Web Audio unlock shim to `web/games/06_fruitninja/index.html`,
placed in `<head>` before trunk's injected wasm loader so it installs before
Bevy constructs its `AudioContext`. The shim wraps the `AudioContext` /
`webkitAudioContext` constructor to record every context Bevy/cpal builds, and
on the first `pointerdown` / `touchend` / `mousedown` / `keydown` it calls
`resume()` on each and starts a 1-sample silent buffer through it (WebKit's
stricter unlock wants a real node start inside the gesture), detaching its
listeners once a context reaches `running`. It is a no-op on Chrome/Firefox.

Also rewrote the "Audio and the autoplay policy" section of
docs/wasm-web-builds.md: the previous text claimed no unlock code was needed,
which held only for Chrome/Firefox (they auto-resume on `start()`). WebKit does
not, which is exactly why mobile Safari was silent; the doc now states the
WebKit resume requirement and points at the shim.

### Why this shape

- JS in the host page, not Rust: Bevy 0.18 does not expose its `AudioContext`,
  so there is no clean Rust `resume()`. The host HTML is the only place with a
  handle, and wrapping the constructor is the only way to capture the context
  Bevy makes internally.
- Constructor wrap + resume-on-gesture is the standard, well-worn Web Audio
  unlock pattern (howler/unmute-style). The silent-buffer kick covers the
  older WebKit quirk where a bare `resume()` on a pre-gesture context does not
  fully unlock without a node start in the same gesture.
- Kept it inline in this one game rather than a shared file: only 06_fruitninja
  has sound today. When a second sounded game appears, lift it into a shared
  snippet (noted in the docs).

### Difficulties

- Worktree had no `node_modules`, so the first `npm run build` did `build:games`
  (trunk, succeeded, shim present in emitted HTML) then failed `build:web` with
  `webpack: command not found` (exit 127). Ran `npm ci` in the worktree, then
  `build:web` passed and the shim is in the shipped `dist` HTML.

### Verification

- `npm run build` (trunk `--release` games + webpack) exit 0; shim confirmed in
  both `web/build/games/06_fruitninja/index.html` and the final
  `web/dist/games/06_fruitninja/index.html` (webkitAudioContext x2,
  createBufferSource x1).
- `npm run ci` (prettier format:check + eslint) exit 0.
- Native suite green: fmt=0, clippy=0, clippy --features debug=0, test=0,
  test --features debug=0, test --example 06_fruitninja=0, check-ascii=0.

### Honest limit (hand-off)

This headless box has no audio device and no mobile Safari, so audibility was
NOT heard here. User check: open the web build on an iOS/Safari device, tap to
start, confirm slice/combo/bomb/game-over sounds play; on desktop
Chrome/Firefox confirm sound still plays with no console errors.

### Self-reflection

- The earlier "no unlock code needed" decision (task 20260703-163329) was right
  for the browsers it tested but generalized past its evidence to "browsers".
  Lesson carried into the doc: state which engines a browser claim covers;
  WebKit almost always needs its own line.
- Could have run `npm ci` up front knowing worktrees start without
  `node_modules`; instead learned it from the exit-127. Minor, but worth a
  retro note so the next web task installs deps before building.
