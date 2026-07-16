# Spike: turn examples 01-05 into small games

- STATUS: CLOSED
- PRIORITY: 60
- TAGS: spike,docs,example

## Goal

`examples/06_fruitninja` proved that a numbered example can grow from a tech
demo into a small, self-contained game that exercises several crate modules at
once and doubles as a showcase for the wasm gallery. This spike asks: which of
the other tech-demo examples (01-05) could get the same treatment, and what
would each game look like?

This is an ideation / research spike, not an implementation task. The
deliverable is a design doc under `docs/` that, for each example:

- proposes a concrete small game the example could become;
- lists the crate modules the game would exercise (the point is coverage:
  a good candidate touches modules that 06 does not);
- calls out gaps -- anything the crate is missing that the game would need,
  which becomes a candidate `feature` task;
- gives a rough effort / payoff read so we can pick what to build next.

## Deliverable

- `tasks/20260703-165138/NOTES.md` with one section per example
  (01-05), a coverage matrix, and a recommendation of the 1-2 strongest
  candidates to turn into real games.
- Any strong follow-up game ideas filed as `feature`/`example` tatr tasks so
  they land in the backlog instead of only living in the doc.

## Out of scope

- Actually building any of the games. Each recommended game gets its own
  feature task (or task tree) later, following the 06_fruitninja pattern
  (numbered example, clap CLI header, states, wasm build, sounds).
- Reworking the existing 01-05 examples in place. The tech demos stay as the
  minimal quickstart; the games are new numbered examples (07+).

## Notes

- Follow the 06_fruitninja shape for any recommended build: Bevy states for
  menu/playing/game-over, `SfxPlugin` sounds, a wasm/trunk build wired into
  the web gallery.
- Prefer games that cover modules 06 skips: the cameras (chase/skybox/wasd),
  the `transform/*` orbit drivers, the `physics/pd_controller`, the
  `modding` event bus, and the `ui/status` bar.
