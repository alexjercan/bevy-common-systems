# Retro: keep the losing scene visible behind game over

- TASK: 20260703-143615 (merged, 1 round, APPROVE)

## What went well
- Read the actual despawn sites and the overlay before planning, so the plan
  was correctly "two-line rescope", not a rebuild: the overlay was already
  transparent and movement was already Playing-only, so the scene freezes for
  free. Understanding the existing structure turned a vague ask into a tiny
  change.
- Verified the state-graph assumption (Playing only exits to GameOver) that
  makes DespawnOnExit(GameOver) safe, and proved cleanup with an auto-driver
  logging scene-entity counts across every transition - no leak into a fresh
  run.

## What went wrong
- Nothing. The fragment TempEntity lifetimes still expire on the game-over
  screen (fragments fade over ~3s while fruit stay frozen); noted as acceptable
  rather than adding scope to pause TempEntity.

## Improve next time
- When an ask sounds like "add a background/rework the screen", first check
  whether the desired effect is really "stop despawning what's already there" -
  the cheapest fix is often removing a cleanup, not adding rendering.
