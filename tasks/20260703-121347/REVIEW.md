# Review: Fruit ninja bombs, health and lose condition

- TASK: 20260703-121347
- BRANCH: feature/fruitninja-bombs

## Round 1

- VERDICT: APPROVE

The lose mechanic is correctly wired through the crate's health system:
slicing a `Bomb` triggers `HealthApplyDamage` (lethal) on the `Player`,
`HealthPlugin` inserts `HealthZeroMarker`, and the `on_player_died` observer
(`On<Add, HealthZeroMarker>`, filtered to the player) sets `GameState::
GameOver`. Verified on real GPU via a throwaway auto-slicer: Playing -> sliced
a BOMB -> GameOver, no panic. The shared `Sliceable`/`Projectile` refactor with
a `Bomb` marker is clean and reads well; `on_damage`'s own guards make repeated
lethal triggers in one swipe idempotent. Checks clean (`fmt`, `clippy
--all-targets` both feature configs, `check-ascii`). Design matches the
confirmed spec (bomb = instant loss, only bombs affect health). Two NITs.

- [ ] R1.1 (NIT) examples/06_fruitninja.rs - `spawn_fruit` and `slice_fruit`
  now handle bombs too, but their sibling `move_fruit` was renamed to
  `move_projectiles`. Rename `spawn_fruit -> spawn_projectile` and
  `slice_fruit -> slice_objects` (or similar) so the launcher/slicer names
  match the shared `Sliceable`/`Projectile`/`Bomb` model instead of implying
  fruit-only.
  - Response: Done - renamed `spawn_fruit -> spawn_projectile` and
    `slice_fruit -> slice_objects`, updated the stale doc comment. Re-ran fmt /
    clippy (both configs) / check-ascii clean.

- [ ] R1.2 (NIT) examples/06_fruitninja.rs:slice bomb branch + on_player_died -
  a sliced bomb gets `ExplodeMesh` for feedback, but the resulting
  `GameState::GameOver` transition despawns the bomb's fragments via
  `DespawnOnExit(Playing)` on the very next state transition, so the burst is
  visible for roughly one frame. Acceptable (you just lost, the field clears),
  but if a beat of bomb explosion before the game-over screen is wanted, add a
  short delay before the transition or let fragments outlive the state. Left as
  a note, not required.
  - Response: Left as-is. Clearing the field on loss is acceptable game feel;
    not worth adding a timed transition to the example.
