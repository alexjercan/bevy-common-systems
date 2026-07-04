# Retro: ui/menu screen builders + TitlePulse

- TASK: 20260704-175424
- BRANCH: feat/ui-menu (squash-merged to master)
- REVIEW ROUNDS: 1 (APPROVE, three informational NITs)

Third dev-harness Wave 2 task. Harvest the menu / game-over overlay
boilerplate five games copy verbatim.

## What went well

- Let the evidence draw the line between "harvest" and "framework". The two
  builders (`centered_screen`, `screen_text`) are byte-identical across the
  games, so lifting them is pure dedupe. The whole-screen `menu_screen` /
  `game_over_screen` the task also named are NOT identical -- title text,
  colours, controls string and best-score format all vary -- so a builder there
  would be all-parameters. Shipped the two safe builders + `TitlePulse` and
  dropped the screen builders as a documented negative result, the same
  sketch-then-commit discipline the earlier Wave 2 tasks used.
- Reproduced the hand-rolled pulse exactly instead of approximating. Worked the
  algebra: `TitlePulse::new` defaults (min 0.3, max 1.0, speed 2.5) expand to
  `0.3 + 0.7 * (0.5 + 0.5 * sin(t*2.5)) = 0.65 + 0.35 * sin(t*2.5)`, byte-for-byte
  the old ramp. A unit test pins the defaults so a later tweak cannot silently
  drift the feel.
- Turned the pulse from a state-gated system into a component-driven one. The old
  `pulse_menu_title` ran `in_state(Menu)`; the new `pulse_titles` runs on
  component presence, and `DespawnOnExit(Menu)` empties the query elsewhere. Less
  coupling, and any screen can reuse the breathe.
- Caught the one behaviour delta myself before review: `screen_text` now always
  sets `TextLayout::Center` (10's superset), which the 06/07 helpers omitted.
  Verified it only affects the multi-line controls line (an improvement) and left
  it in on purpose.

## What went wrong

- First draft of `pulse_titles` was broken nonsense: I hoisted the sine out of
  the loop into a `phase` binding and then called it like a function
  (`phase(pulse.speed)`), plus an unused `SinePhase` trait. Rewrote it inline
  before building. Root cause: over-engineering a three-line system -- the
  per-entity speed means the sine simply belongs inside the loop.

## What to improve next time

- When a task names more builders than the evidence supports, split "verbatim
  dupe" from "varies per game" up front and say which you are shipping -- naming
  the negative result in the review and retro is cheaper than a reviewer asking
  where `menu_screen` went. Same call the assets/HighScore tasks made; it is now
  the wave's default.

## Action items

- [x] `ui/menu` (`centered_screen`, `screen_text`, `TitlePulse`, `MenuPlugin`)
  shipped; 06 and 07 refactored.
- [ ] Migrate 09/10/11 onto the menu builders alongside SoundBank/HighScore in
  tatr 20260704-223846.
- [ ] Last dev-harness Wave 2 task: input `AnyStartPress` + leaf helpers
  (20260704-175425).
