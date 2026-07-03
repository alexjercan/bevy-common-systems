# Fruit ninja: keep the losing scene visible behind game over

- STATUS: CLOSED
- PRIORITY: 100
- TAGS: feature,example

## Goal

On game over, keep the frozen game scene (the fruit / bomb fragments as they
were when you lost) visible behind the game-over overlay, instead of despawning
everything and leaving an empty clear-color background. The overlay is already
transparent, and the movement systems are `Playing`-only, so the scene freezes
in place; the only change needed is to stop despawning the gameplay entities on
exit from `Playing`.

## Steps

- [x] In `spawn_projectile`, change the fruit/bomb entity's
      `DespawnOnExit(GameState::Playing)` to `DespawnOnExit(GameState::GameOver)`
      so it persists through the game-over screen and is cleared when leaving it
      (back to the menu).
- [x] In `on_fragments_spawned`, change the fragment entity's
      `DespawnOnExit(GameState::Playing)` to `DespawnOnExit(GameState::GameOver)`
      for the same reason (bomb/fruit fragments stay visible on the death
      screen).
- [x] Leave the HUD (score/combo), player, red flash and floating popups on
      `DespawnOnExit(GameState::Playing)` so they clear on game over -- the
      overlay itself shows the final score.
- [x] Confirm the flow clears correctly: `Playing -> GameOver` keeps entities
      (frozen), `GameOver -> Menu` despawns them (menu is clean), a new
      `Playing` starts empty. This relies on `Playing` only ever exiting to
      `GameOver` (bomb death or Escape give-up), which is the current graph.
- [x] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, and a real boot: reach game over and
      confirm the frozen scene shows behind the overlay, then that returning to
      the menu and starting again is clean (throwaway auto-driver that slices a
      bomb and logs entity counts across the state transitions).

## Notes

- Sites: `spawn_projectile` fruit/bomb spawn (`DespawnOnExit(Playing)` ~line
  982) and `on_fragments_spawned` fragment spawn (~line 1272).
- `centered_screen` (the game-over/menu overlay) has no `BackgroundColor`, so
  the 3D scene shows through it already -- no UI change needed.
- Fragments carry `TempEntity(FRAGMENT_LIFETIME)`; `TempEntityPlugin` is not
  state-gated, so frozen fragments will still expire after their lifetime on the
  game-over screen. Acceptable (they fade out); note it.
- Movement (`move_projectiles`, `move_fragments`) is `Playing`-only, so the
  scene is a frozen snapshot on game over -- the desired "see where you lost".
- No new dependencies.

## Close-out

Rescoped fruit/bomb (spawn_projectile) and fragments (on_fragments_spawned) from
DespawnOnExit(Playing) to DespawnOnExit(GameOver). HUD/player/flash/popups still
clear on Playing exit; the transparent centered overlay shows the frozen scene
through it (movement is Playing-only, so it freezes). Verified: GameOver keeps
the scene (3 entities), Menu clears it (0), new run starts fresh (0).
