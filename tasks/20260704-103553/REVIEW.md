# Review: 08_dropzone hazards pass

- TASK: 20260704-103553
- BRANCH: feature/08-dropzone-hazards

## Round 1

- VERDICT: APPROVE

An independent skeptical review pass (a subagent, prompted to hunt for
correctness bugs, ECS/query-conflict risks and logic errors, cross-checked
against `src/health/mod.rs`, `src/mesh/explode.rs` and
`src/transform/random_sphere_orbit.rs`) found **no CRITICAL or MAJOR issues**.
The implementation is well-guarded and the collision-classification refactor is
a genuine improvement over the old "any ship contact = landing" logic.

Verified in the review:

- **No double-resolution.** `HealthApplyDamage` is deferred; `on_damage` is
  idempotent once health <= 0, so `HealthZeroMarker` is inserted at most once and
  `on_ship_destroyed` fires exactly once. Every terminal path guards on
  `outcome.0.is_some()`, and `resolve_collisions`/`resolve_asteroid_hits` share
  `ResMut<CameraShake>` so they are serialized, never concurrent.
- **No query conflicts.** All three Ship-querying Update systems read Ship
  components immutably; `apply_asteroid_transforms` mutates `Transform
  With<Asteroid>`, disjoint from the camera-shake `Transform With<MainCamera>`.
- **Asteroid shatter is sound.** `Visibility::Hidden` leaves `Mesh3d` intact for
  `handle_explosion` to slice; fragments spawn as top-level entities (visible
  even though the shell is hidden); `TempEntity(0.2)` despawns the shell only
  after the synchronous Add/Insert observers have run.
- **Obstacle damage.** avian `CollisionStart` fires only on contact begin, so a
  ship resting on a rock does not take per-frame damage; the planet/obstacle
  markers sit on the same entities as their colliders, so classification is
  correct.
- **Wind** has no NaN path (all `normalize_or` fallbacks; the pole-degeneracy
  guard is adequate) and the one-frame Update-vs-FixedUpdate lag is negligible at
  the chosen slow gust/turn rates.
- **`on_ship_destroyed`** targets the ship (children have no `Health`; the marker
  lands on the top-level ship), the `add.entity != ship` guard is correct, and
  re-fire is impossible.

Three MINOR/cosmetic edges were noted, none requiring correctness fixes:

1. A lethal asteroid graze in the *same frame* as a soft touchdown resolves as
   LANDED (landing set first, deferred destroyed guarded out). Rare and the
   kinder outcome; documented with an inline comment.
2. A ship pinned against a rock with jittery make/break contacts could re-fire
   `CollisionStart` and slowly drain integrity while "resting" -- theoretical;
   stable manifolds do not repeat. Flagged for playtest.
3. One-frame wind lag (negligible).

## Verification

- `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test` (9 example
  tests incl. the new `impact_damage` one), `scripts/check-ascii.sh`: all pass.
- Ran the example (reaches the render loop). A temporary env-gated autopilot
  (`DROPZONE_SMOKE`, since removed) flew the real systems through
  Menu -> Playing -> Result with no panic, confirming each hazard path: 7
  asteroids spawn and drift, wind evolves and moves the ship, asteroid grazes
  drain hull 100 -> 47 and shatter (count 7 -> 5), a rock impact at 6.3 m/s
  emptied the remaining hull and ended the run, and a forced lethal damage ended
  a free-flying run via `on_ship_destroyed`.
- Web showcase rebuilt: `npm run build` exits 0, webpack compiles successfully,
  `web/dist/games/08_dropzone/` holds the fresh wasm bundle.

Scope held: the flight model (PD controller, gravity, thrust) and the
Menu/Playing/Result state machine are untouched; no out-of-scope mechanics
(pickups beyond fuel, upgrades, cargo, multi-leg) added. Terrain contact stays
instant-crash by decision; only obstacles and asteroids cost integrity.
