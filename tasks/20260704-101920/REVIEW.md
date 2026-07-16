# Review: Share Web Audio unlock shim across all web games (fix mobile/iOS sound)

- TASK: 20260704-101920
- BRANCH: fix/mobile-web-audio

## Round 1

- VERDICT: APPROVE

Verified against master (`git diff master...fix/mobile-web-audio`). The diff
touches only `docs/wasm-web-builds.md`, the three game `index.html` files, the
new `web/games/_shared/audio-unlock.js`, and the task dir. No Rust/toml.

What I checked and confirmed:

- **Extracted shim is byte-equivalent to 06's known-good version.** Diffed the
  new `_shared/audio-unlock.js` IIFE against `master:06_fruitninja`'s inline
  shim: identical after normalizing indentation and comment reflow (2786 code
  chars each, comments/whitespace stripped). No logic changed -- 07/08 now get
  exactly the shim already shipping in 06 (constructor wrap + resume +
  1-sample node kick + iOS media-channel silent `<audio>` + visibilitychange
  re-arm).
- **`SILENCE` base64 is the real one.** Asserted equal to 06's literal and that
  it decodes to a valid all-zero RIFF/WAVE (800 zero payload bytes). Avoids the
  hand-reconstructed-base64 trap from the 20260703-212303 retro (an early cut
  here did mangle it; caught and replaced with the exact literal + round-trip
  assert).
- **trunk staging + script ordering, verified in the BUILT dist, not source.**
  Ran the real entry point `npm run build:games` (all three green). In every
  built game the `copy-file` link stages `audio-unlock.js` into the dist root
  sha256-identical to source, and the plain classic `<script src>` appears
  before trunk's deferred `<script type="module">` wasm loader -- so the
  AudioContext constructor wrap installs before Bevy builds its context.
  (Classic scripts run during parse, before deferred modules, so this holds
  even independent of byte order.)
- **No collateral.** `_shared/` is not treated as a game (build-games.sh and
  the TS registry enumerate games explicitly); it is tracked in the commit and
  not gitignored. prettier/eslint globs (`src/**`, root files) do not cover the
  new `games/**` html or `_shared/*.js`, so `npm run ci` is unaffected.
  `check-ascii.sh` and `cargo fmt --check` pass; the new web files are plain
  ASCII.
- **Docs match the code.** The autoplay section now describes the single shared
  file and the two-line include, and warns against re-inlining; the
  ringer-channel paragraph still applies verbatim.

Findings:

- [x] R1.1 (NIT) tasks/20260703-200005/RETRO.md:56 - that
  retro's open action item ("Future: when a second web game gains sound, lift
  the inline shim into a shared snippet ... web/games/_shared/audio-unlock.js")
  is exactly what this task did. Optional bookkeeping: tick it and note it was
  addressed by 20260704-101920. Leaving it is also fine (retros are historical
  records). Not blocking.
  - Response: Addressed -- ticked the action item in that retro and annotated
    it as done by task 20260704-101920.

Non-blocking caveat (not a finding): this cannot be device-confirmed on a real
iPhone from here. The mitigation is strong -- 07/08 are now byte-identical to
the shim already confirmed working on iOS for 06_fruitninja -- but a final
on-device check on an iPhone with the Ring/Silent switch on Silent is still the
last-mile confirmation, same as the 20260703-212303 task left for the user.
