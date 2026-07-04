# Share Web Audio unlock shim across all web games (fix mobile/iOS sound)

- STATUS: CLOSED
- PRIORITY: 100
- TAGS: bug,web,audio

## Goal

Sound effects are silent on mobile (iOS Safari) in the 07_orbit and
08_dropzone web builds, while 06_fruitninja plays. Root cause: the Web Audio
unlock shim is inline-copied into each game's `web/games/<name>/index.html`
and has drifted -- 06_fruitninja has the full shim (AudioContext resume +
silent-buffer node kick + iOS ringer-channel silent-`<audio>` media-channel
promotion for WebKit bug 237322 + visibilitychange re-arm), but 07_orbit and
08_dropzone only have the older partial shim (resume + node kick, no iOS
silent track). Extract the full shim into ONE shared JS file that all three
games load, so it can never drift again, and bring 07/08 to parity.

## Steps

- [x] Create `web/games/audio-unlock.js` containing the full shim currently
      inline in `web/games/06_fruitninja/index.html` (the IIFE: wrapped
      AudioContext constructor capturing every context; `isIOS` detection;
      `SILENCE` data-URI looping silent `<audio>` on the media channel;
      `unlock()` = resume + 1-sample buffer kick + `playSilentTrack()`;
      `detachIfRunning()` that keeps listening on iOS; `visibilitychange`
      pause/resume). Move the explanatory comment block into the file as a
      top-of-file comment so it lives with the code.
- [x] In `web/games/06_fruitninja/index.html`, replace the inline `<script>`
      shim (and its preceding explanatory comment) with a single
      `<script src="audio-unlock.js"></script>` in the same `<head>` position
      (before the `data-trunk rel="rust"` link), plus a
      `<link data-trunk rel="copy-file" href="audio-unlock.js" />` so trunk
      stages the file into the game's dist root. Keep it a plain (non-module,
      non-defer) script so it runs synchronously before trunk's injected
      `<script type="module">` wasm loader and wraps AudioContext in time.
- [x] Apply the identical replacement to `web/games/07_orbit/index.html` and
      `web/games/08_dropzone/index.html` (they get the same `<script src>` +
      `copy-file` link; drop their old partial inline shim and its comment).
      The relative `href`/`src` resolve under each game's `--public-url`
      (`<public>/games/<name>/audio-unlock.js`), so no per-game path edits.
- [x] Build the games with trunk from the nix devshell
      (`OUT_ROOT` staging via `web/scripts/build-games.sh`, or a single
      `trunk build --release --example 06_fruitninja --public-url /games/06_fruitninja/ --dist <tmp> web/games/06_fruitninja/index.html`
      for a quick check). Verify in the generated dist HTML that:
      (a) `audio-unlock.js` was copied into the dist root of each game;
      (b) the `<script src="audio-unlock.js">` appears BEFORE trunk's injected
      `<script type="module">`/`init(` call (byte-offset check like the one
      used during planning);
      (c) all three games now reference the same shared file.
- [x] Update the "Audio and the autoplay policy" section of
      `docs/wasm-web-builds.md` (around lines 98-165): it currently says the
      shim is copied per game and "any future web game with sound should copy
      this shim". Change it to describe the single shared
      `web/games/audio-unlock.js` and the two-line include
      (`<script src>` + `copy-file`) each new game adds instead.
- [x] Run the repo check suite that applies to these changes:
      `./scripts/check-ascii.sh` (the new JS/HTML must stay plain ASCII),
      `cargo fmt --check` and `cargo clippy --all-targets` (no Rust changed,
      but confirm green), and `npm run lint` / prettier in `web/` if the JS
      file is subject to eslint/prettier (run `npm ci` first in the worktree's
      `web/` -- fresh worktrees have no node_modules).

## Notes

- Relevant files:
  - `web/games/06_fruitninja/index.html` (has the GOOD full shim to extract),
  - `web/games/07_orbit/index.html`, `web/games/08_dropzone/index.html`
    (partial shims to replace),
  - `web/scripts/build-games.sh` (trunk driver; `games=(...)` array),
  - `docs/wasm-web-builds.md` (audio section to update),
  - `src/audio/mod.rs` (SfxPlugin; no change needed).
- Confirmed during planning: in the current built
  `web/dist/games/06_fruitninja/index.html`, the inline shim
  (`function Wrapped`) sits at byte ~1937 and trunk's injected
  `<script type="module">` / `init(` at ~6792, i.e. the head script already
  runs first. A non-module external `<script src>` in `<head>` preserves that
  ordering (render-blocking, executes before deferred module scripts).
- No Rust/wasm change is possible or needed: Bevy/cpal owns the AudioContext
  and does not expose it; the unlock must be JS page-side inside the iframe
  document (the in-canvas start gesture provides the user gesture).
- No parent-page (`web/src`) change needed; the iframe already carries
  `allow="autoplay; fullscreen; gamepad"`.
- Prettier/eslint may reformat the extracted JS; that is fine as long as the
  logic is byte-equivalent to 06_fruitninja's known-good shim.
- Cannot fully verify on a real iOS device from here; verification is the
  build-output HTML inspection plus logic-parity with the shim that is already
  confirmed working in 06_fruitninja.
- The `SILENCE` base64 data URI must be copied verbatim from
  06_fruitninja/index.html (a mono 8kHz 16-bit PCM WAV of zero samples,
  generated in task 20260703-212303).

## Outcome

Extracted the full 06_fruitninja shim into `web/games/_shared/audio-unlock.js`
(the path the earlier audio retro predicted) and replaced the inline shim in
all three games with two lines: `<script src="audio-unlock.js"></script>` plus
`<link data-trunk rel="copy-file" href="../_shared/audio-unlock.js" />`. Doc
`docs/wasm-web-builds.md` updated to describe the shared file + the two-line
include and to warn against re-inlining.

Verified: the `SILENCE` base64 is byte-identical to 06's (asserted equal and
that it decodes to a valid all-zero RIFF/WAVE). `npm run build:games` (the real
entry point) built all three games green; in every built dist,
`audio-unlock.js` is staged into the dist root byte-identical to source
(sha256), and its classic `<script src>` precedes trunk's deferred
`<script type="module">` wasm loader (so the AudioContext constructor wrap
installs first). `check-ascii.sh`, `cargo fmt --check` pass; no Rust/toml
changed so clippy is unaffected. Not device-tested on a real iPhone, but the
shim is now byte-identical to the one already confirmed working on iOS for
06_fruitninja.
