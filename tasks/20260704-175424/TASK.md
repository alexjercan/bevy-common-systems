# ui/menu screen builders: centered_screen/screen_text/menu_screen/game_over_screen/TitlePulse (Wave 2)

- STATUS: OPEN
- PRIORITY: 45
- TAGS: spike,ui,feature

> Spike: docs/spikes/20260704-175058-dev-harness-and-app-scaffolding.md (read
> first). Wave 2 -- the low-risk half of the deferred ui/menu proposal
> (tasks/20260704-134800); fold this evidence into that task rather than racing
> it.

## Goal

`centered_screen() -> Node` and `screen_text(text, size, color)` are duplicated
VERBATIM across five games (06:774,788; 07:686,700; 09:671,684; 10:609,623;
11:501,514), and on top of them every game hand-builds a `menu_screen` (title +
"tap to play" + best + controls) and a `game_over_screen` (title + score + the
identical `new_best` branch + "tap to return"). Provide opinion-light builders
mirroring `ui/status::status_bar_item()`, plus a `TitlePulse` component for the
`pulse_menu_title` sine breathe copied in 5/6 games.

This is exactly the "menu_button() builder" half that tasks/20260704-134800
already flagged as low-risk and in-scope; this task adds the verbatim-dupe
evidence for the screen/text builders. It does NOT touch the state-machine half
of that proposal, which still needs the user decision. Coordinate: prefer
extending tasks/20260704-134800 over duplicating it.

Prove it by refactoring the games' menu/game-over screens onto the builders.
This task is stepless on purpose (spike output); run /plan to break it into
steps before /work.
