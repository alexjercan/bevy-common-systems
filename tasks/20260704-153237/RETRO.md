# Retro: port 06/08 floating text onto ui/popup

- TASK: 20260704-153237
- BRANCH: feat/ui-popup-ports (squash-merged to master as efb47e0)
- REVIEW ROUNDS: 1 (APPROVE, clean)

A clean, mechanical dedup cycle -- short retro by design (a smooth cycle
deserves a short one).

## What went well

- Ran the two ports as parallel subagents (06 and 08, independent files) exactly
  like the camera/shake ports, and both came back clean in one round. The
  now-established port recipe (add plugin, route call sites through the builder,
  delete the local component/systems/consts, keep the projection in the example)
  is reliable enough to hand off verbatim.
- Applied the previous retro's lesson directly: the port-agent prompts told them
  to run `cargo fmt` before reporting, and there was zero formatting drift at the
  combined step (unlike the camera/shake ports, where a stray blank line slipped
  through). The mechanical fix worked.
- Called out the one real difference up front in the prompts -- 08's 0.9/60 feel
  vs the module's 0.8/70 default -- with the exact override pattern, so the agent
  preserved the feel instead of silently accepting the default. The reviewer
  confirmed it.

## What went wrong

- Nothing of substance. The review found no BLOCKER/MAJOR/MINOR, only an
  informational NIT (the entity `Name` changed from "Floating Text" to "Popup",
  not queried anywhere).

## What to improve next time

- Keep doing this for clean dedups: precise per-file prompts naming the one
  divergence (here 08's feel), `cargo fmt` in the agent, parent-side combined
  verification + boot. The pattern is now proven across two port tasks.

## Action items

- [x] ui/popup now promoted across all three games that had floating text
  (06/07/08); the module's dedup story is complete.
- [ ] Sibling task still open: promote the full-screen damage overlay into
  `feedback/screen_flash` (tatr 20260704-155505) -- the last of the
  "harvest what the games duplicate" cleanups. Then Wave 2 (tween/persist/spawn).
