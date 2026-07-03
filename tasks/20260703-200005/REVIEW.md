# Review: Fix mobile Safari silent audio in 06_fruitninja web build

- TASK: 20260703-200005
- BRANCH: fix/safari-audio-unlock

## Round 1

- VERDICT: APPROVE

Diff reviewed against master: the inline AudioContext-unlock shim in
`web/games/06_fruitninja/index.html`, the rewritten autoplay section of
`docs/wasm-web-builds.md`, and the TASK.md outcome. Verified independently:

- Root cause is correct. WebKit does not auto-resume a suspended AudioContext
  on `start()` the way Chrome/Firefox do; it needs `resume()` inside a real
  gesture. Bevy 0.18 does not expose its context, so a host-page shim is the
  only handle -- the approach is the right one, not framework machinery.
- Constructor-wrap correctness: `new Wrapped()` returns an explicit object, so
  `new` yields the real Native AudioContext instance; `Wrapped.prototype =
  Native.prototype` keeps `instanceof` intact. wasm-bindgen resolves the global
  `AudioContext` name at call-site, and the shim sets `window.AudioContext`
  before the wasm boots, so Bevy's context is captured. Both
  `AudioContext` and `webkitAudioContext` are wrapped.
- Ordering verified in the BUILT `dist` HTML: the synchronous inline shim is at
  lines 27-42; trunk's `type="module"` loader (deferred) is at line 86. The
  wrapper is installed before Bevy constructs its context. Not just asserted --
  read out of the emitted file.
- Gesture handling is sound: capture-phase listeners fire even if the canvas
  stops propagation; `resume()` promise + `detachIfRunning` removes the
  listeners once a context reaches `running`; empty-contexts case leaves the
  listeners attached to retry on a later gesture (handles late context
  creation). The silent-buffer kick covers WebKit's stricter node-start
  requirement and is a no-op on desktop.
- Checks re-run by reviewer: `cargo fmt --check`=0, `check-ascii`=0,
  `cargo test --example 06_fruitninja`=19 passed. Work phase also recorded
  clippy x2, test x2, `npm run build` (trunk+webpack) and `npm run ci` green,
  with the shim confirmed in the shipped `dist` HTML. No test was weakened.
- Docs are honest and now engine-specific (the previous "no code needed" claim
  was Chrome/Firefox-only); the shim is documented and pointed at.

Non-blocking:

- [ ] R1.1 (MINOR) web/games/06_fruitninja/index.html - audibility was not
  verified on a real device (headless box, no audio, no Safari). The wasm-side
  assumption (wasm-bindgen calls the wrapped global constructor, and `resume()`
  + silent buffer actually unlocks iOS Safari) is standard and well-supported
  but unconfirmed here. User must open the web build on an iOS/Safari device,
  tap to start, and confirm slice/combo/bomb/game-over sounds play; and confirm
  desktop Chrome/Firefox still plays sound with no console errors. Tracked as
  the task's hand-off, not a code defect.
  - Response:
- [ ] R1.2 (NIT) web/games/06_fruitninja/index.html - the shim is inline in the
  one sounded game. When a second web game gains sound, lift it into a shared
  snippet (e.g. web/games/_shared/audio-unlock.js copied via trunk). Already
  noted in the docs; no action now.
  - Response:

No BLOCKER or MAJOR findings; both open items are non-blocking. APPROVE.
