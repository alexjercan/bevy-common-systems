# Retro: mobile touch controls for 09_reactor and 11_overload

- TASK: 20260704-142016 (09_reactor); 11_overload reused from an existing branch
- BRANCH: reactor-overload-mobile (not yet merged)
- REVIEW ROUNDS: reactor R1 REQUEST_CHANGES -> R2 APPROVE; overload inherited an
  already-approved implementation

Goal: make `examples/09_reactor.rs` and `examples/11_overload.rs` playable on a
phone in the wasm showcase. See `docs/2026-07-04-reactor-touch-controls.md` and
`docs/2026-07-04-overload-touch-controls.md` for what shipped. This is about how
the working went.

## What went well

- **Checked for existing work before building.** `git branch` surfaced an
  unmerged `asteroids-overload-touch` branch that already had a complete,
  tested, retro'd touch implementation for 11_overload. Rather than redo it (and
  create a divergent parallel implementation of the same feature), I surfaced it
  to the user, who chose to reuse it; I cherry-picked the one commit
  (`da90c4b`). Half the goal was met in one clean pick. Reading the branch list
  first turned a day of duplicate work into a 30-second decision.
- **Discovered the session CAN see the screen, and used it.** The recurring
  AGENTS.md gotcha says a background run "cannot see the screen", so visual bugs
  slip through. This time I checked: `scrot`, ImageMagick `import`, and
  `xdotool` (via `nix run nixpkgs#xdotool`) were all available under
  `DISPLAY=:0`. I built a screenshot harness (temporary `REACTOR_SHOT="WxH"` env
  to force a phone-portrait window + auto-start into Playing, `xdotool` to move
  the window to a known spot, `import` + `magick` to crop it) and actually
  LOOKED at the layout at 320/344/360/560. This caught a real regression that
  pure reasoning had underestimated and let me iterate the fix to a verified
  result instead of shipping blind.
- **The review caught the thing the enumerated steps missed.** The task steps
  were "menu/game-over touch nav + web canvas", and the in-game buttons genuinely
  work on touch via `bevy_ui`'s focus system -- so my first pass looked done and
  passed all automated checks. The independent reviewer pointed out that
  touch-capable buttons are worthless if they render below the fold: in the
  showcase's fixed 4:5 portrait frame, four of six shop cards fell off the bottom
  with no keyboard-digit fallback on a phone. Reachability, not just
  responsiveness. The honest review phase earned its keep.
- **Verified non-obvious framework claims against source, twice.** Confirmed
  `bevy_ui`'s `ui_focus_system` reads `Res<Touches>` directly (so `Interaction`
  fires on taps independent of `bevy_picking`), and confirmed Bevy 0.19 UI is
  `BoxSizing::BorderBox` so a `48%` card width is arithmetically stable at two
  columns. Both were checked in the crates.io source, not assumed.

## What went wrong

- **First pass implemented the literal steps and missed the goal's implication.**
  Same failure mode as the 08_dropzone touch retro
  (`docs/retros/20260704-103517-dropzone-touch-controls.md`): the enumerated
  steps omitted the connective tissue ("the controls must be ON SCREEN at phone
  size") that the goal ("playable on a phone") plainly required. I even wrote a
  doc section explaining why no virtual pad was needed -- correct -- while not
  checking that the existing buttons fit the phone frame. The lesson from that
  earlier retro (walk the ENTIRE journey on the target device) applies to the
  visual/layout dimension too, not just the input paths.
- **Underestimated a Bevy flexbox rule and had to iterate.** My first layout fix
  used fixed-pixel (`158px`) shop cards + `flex_wrap`, reasoning they would
  shrink to fit two columns on a narrow phone. They do not: flexbox WRAPS before
  it shrinks, so at ~320px the cards dropped to a single column and overflowed
  again. The screenshot caught it; the fix (percentage `48%` width) makes the
  column count independent of frame width. Worth remembering: for a "N columns at
  any width" grid in Bevy UI, use percentage widths, not fixed px + wrap.
- **Chased the layout across five window sizes by hand.** Getting the 4:5 frame
  to fit six descriptive cards + a full header took several screenshot-tune
  cycles (readout font, label length, paddings, card width model). Efficient in
  the end, but I could have budgeted the vertical space up front (header height
  vs 3 card rows in a 4:5 box) and hit the answer in two iterations instead of
  six.

## What to improve next time

- For any "playable/usable on X" goal, add an explicit REACHABILITY check to the
  journey walk: not just "can every action be triggered by touch" but "is every
  control on screen within the target viewport". For a UI-dense example, budget
  the target frame's pixel height against the content before writing Nodes.
- When a background session has a display (`echo $DISPLAY`), reach for the
  screenshot pipeline (`scrot`/`import` + `xdotool` via `nix run`) to VERIFY the
  visual layer, instead of treating "can't see the screen" as a hard limit. A
  temporary env-gated window-size + auto-start harness (like the smoke-autopilot
  pattern) makes any state and any viewport capturable. This is the single
  highest-leverage technique from this cycle.
- In Bevy 0.19 UI, a fixed-column responsive grid wants percentage item widths;
  fixed-px + `flex_wrap` collapses to one column on narrow frames because wrap
  precedes shrink.

## Action items

- [x] Added an AGENTS.md gotcha: a background session with `$DISPLAY` can
  screenshot the running app (`scrot`/`import` + `xdotool` via `nix run`) to
  verify the visual layer; use a temporary env-gated window-size + auto-start
  harness to capture any state/viewport. (Turns the "cannot see the screen"
  limitation into a verifiable step.)
- [ ] Follow-up not filed: touch-feel on a real phone / browser touch-emulator
  (button thumb-sizes, the 10px shop-card description text, the sub-340px frame
  vertical clip) still wants a hands-on pass, like the dropzone `*_PX` constants.
