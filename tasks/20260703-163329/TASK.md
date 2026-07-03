# Verify web audio playback and handle the browser autoplay policy

- STATUS: OPEN
- PRIORITY: 70
- TAGS: web,wasm,audio,verify

## Goal

With the audio assets now shipped into the wasm build (task
20260703-163328), confirm that the `06_fruitninja` web build actually
plays sound in a browser, and make the crate robust against the browser
autoplay policy so the first in-game sound is not silently dropped.

Browsers create a Web Audio `AudioContext` in the `suspended` state until a
user gesture occurs on the page, and Bevy's audio (rodio/cpal -> web-sys) does
nothing to resume it. In this showcase the game is embedded in an iframe whose
first same-document gesture is the in-canvas click that starts a run (which is
also when the first SFX, `menu_select`, fires), so in practice modern browsers
should auto-resume on that click. This task verifies that assumption and only
adds an explicit unlock if the assumption does not hold.

"Done" = we have concrete evidence about whether SFX are audible in the browser
build after the start click, the autoplay/gesture requirement is documented,
and -- only if verification shows the first sound is dropped -- a minimal,
reusable, opt-in unlock capability exists in the crate's `audio` module and is
wired into the fruitninja example.

## Steps

- [ ] Research the exact autoplay behavior for Bevy 0.19 audio on
      `wasm32-unknown-unknown`: does rodio/cpal create the `AudioContext`
      suspended, and do current Chrome/Firefox auto-resume a suspended context
      on any same-document user gesture (the in-canvas start click)? Cite
      sources (MDN autoplay policy, cpal/rodio wasm backend, any Bevy issue).
      Record the finding in the Outcome. Do NOT invent behavior -- if it cannot
      be established from sources, say so and rely on the manual check below.
- [ ] Manual browser check (needs a graphical session -- the user's machine).
      Serve the built site (`web/build/games/06_fruitninja/` via a static
      server, or the full `npm run` gallery) and open the game in a browser:
      confirm (a) the sound files are fetched with HTTP 200 in the network tab
      (this validates task 20260703-163328 end to end), and (b) SFX are
      audible starting from the menu start click. If this step cannot be run
      autonomously, state clearly in the Outcome that it needs the user, and
      hand off the exact commands/URLs to run rather than claiming success.
- [ ] Decision point based on the above:
      - If sound is audible after the start click: no code change. Document the
        gesture requirement (below) and close.
      - If the first sound is dropped (context stays suspended): add a minimal,
        opt-in unlock. Given Bevy does not expose its `AudioContext` handle,
        prefer the smallest correct mechanism -- e.g. a
        `#[cfg(target_arch = "wasm32")]` helper that resumes suspended audio on
        the first user gesture. Keep it out of `SfxPlugin`'s default path
        unless it is a no-op on native; if added as a plugin, follow crate
        conventions (`*Plugin`, `debug!("...Plugin: build")`, prelude export,
        `//!` module doc with a usage snippet) and make it a genuine no-op on
        non-wasm targets.
- [ ] Document the autoplay/gesture requirement in `docs/wasm-web-builds.md`
      (and cross-link from `assets/sounds/README.md` if useful): explain that
      web audio needs a user gesture, that the showcase satisfies it via the
      in-canvas start click, and how a game that plays sound before any gesture
      would need an explicit unlock.
- [ ] If any Rust code was added: keep CI green -- `cargo build`,
      `cargo clippy --all-targets`, `cargo clippy --all-targets --features
      debug`, `cargo fmt --check`, `cargo test`, `cargo test --features debug`,
      `./scripts/check-ascii.sh` -- and rebuild the wasm via
      `bash web/scripts/build-games.sh` to confirm it still compiles to wasm.

## Notes

- Depends on: 20260703-163328 (assets must ship before playback can be
  verified at all).
- Relevant code/files:
  - `src/audio/mod.rs` -- `SfxPlugin`; any unlock capability would live here or
    in a sibling module, exported via the audio prelude.
  - `examples/06_fruitninja.rs` ~L130-165 -- app setup with the existing
    `#[cfg(target_arch = "wasm32")]` window tweaks; a plugin would be added
    near there. Sound fires on the start click (`menu_click`) and later events.
  - `web/src/index.ts` -- the gallery opens a game by setting the iframe `src`
    on a card click in the PARENT document (not the iframe), so the iframe's
    first same-document gesture is the in-canvas start click.
- Bias toward NOT adding speculative code: the crate values small, composable,
  game-agnostic building blocks, not framework machinery. Add the unlock only
  if verification shows it is actually needed, or if it is a proven, zero-cost,
  clearly-reusable capability. Record the reasoning either way.
- Do not push. Work happens on branch `feature/web-audio` in this worktree.

## Outcome

(to be filled in by /work)
