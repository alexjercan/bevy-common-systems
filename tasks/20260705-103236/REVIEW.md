# Review: Build examples/14_breach -- grounded first-person arena shooter

- TASK: 20260705-103236
- BRANCH: 14_breach

## Round 1

- VERDICT: REQUEST_CHANGES

Verification run: `cargo clippy --all-targets` clean (plain + `--features debug`),
9 unit tests pass, `check-ascii` + `cargo fmt --check` clean, headless
`BCS_AUTOPILOT` reaches Menu->Playing->GameOver with "cycle complete, no panic" and
2 kills, `BCS_SHOT` app-native screenshot renders the scene, `trunk build --example
14_breach` compiles to wasm. But two empirical probes (an aiming autopilot, and a
defenceless-player run with the Playing hold extended to 15s then 30s) exposed a
gameplay problem the happy-path checks hid.

- [ ] R1.1 (MAJOR) `examples/14_breach.rs` `enemy_melee` (~:977) + the lose path
  (`on_health_zero` -> `RunOver` -> `check_run_over`) - the enemy threat barely
  functions and the lose condition is unverified. Empirically, a defenceless player
  standing at the centre survives **30s+** against wave 1: player HP falls 100 -> 50
  over ~20s (~2.5 dmg/s), because the player and enemies are both dynamic bodies, so
  an approaching enemy collides with and is knocked off the player capsule and is
  only within `MELEE_RANGE` when its 0.9s attack cooldown happens to be ready --
  landing a hit roughly every 4s instead of every 0.9s. For a "survive the waves"
  shooter this guts the core threat (you can stand still and not die). Separately,
  the `AutopilotPlugin` `.hold(GameState::Playing, 4.0)` (:134) force-transitions to
  GameOver on a timer, so the player-death -> `RunOver` -> `GameOver` transition is
  never actually exercised by the harness (the observed GameOver is always the forced
  one; the "2 kills" only proves the enemy raycast->damage->death->score->persist
  path). This is the `13_glide`/camera-shake retro trap: a passing autopilot masks an
  unexercised path. Fix: (a) make melee a reliable threat despite the physics
  knockback -- e.g. raise `ENEMY_DAMAGE` / lower `ENEMY_ATTACK_CD`, and/or apply
  proximity damage without requiring the bodies to stay touching (or give enemies
  more mass / the player less so a mob actually pins you) -- so a passive player dies
  in a handful of seconds; and (b) add an App-based headless test (MinimalPlugins +
  StatesPlugin + HealthPlugin) that drives the player `Health` to zero via
  `HealthApplyDamage` and asserts `RunOver.0 == true`, plus one asserting an enemy
  death runs `Score += 1` (the `on_health_zero` accounting is untested).
  - Response: Fixed. Replaced cooldown-gated melee with continuous proximity damage (each in-range enemy drains ENEMY_DPS*dt), dropped player-vs-enemy physical collision so enemies overlap and reliably hit, and opened the arena (straight-line AI got stuck on cover). Added a headless App test (player_death_ends_the_run) asserting Health-zero -> RunOver -> GameOver, and enemy_death_scores_one for the score path. Verified: a defenceless player now dies in ~10s (per-second HP log 100->4 then the run ends).
- [ ] R1.2 (MINOR) `examples/14_breach.rs` gib spawn (~:1070) - the gib fragments use
  `CollisionLayers::new([GameLayer::Default], [GameLayer::World])`, but the world
  colliders filter `[Player, Enemy]` (~:520), which omits `Default`. avian collision
  is bidirectional, so the gib-vs-world contact never resolves and the shards fall
  straight through the floor -- the collider on them is dead weight. Fix: add
  `GameLayer::Default` to the world layer's filter list; that keeps gibs off the ray
  mask (`[Enemy, World]`) and off the player.
  - Response: Fixed. Added GameLayer::Default to the world layer's filter, so gibs collide with floor/walls instead of falling through.
- [ ] R1.3 (MINOR) `examples/14_breach.rs` `on_health_zero` (~:1034) - a dead enemy
  keeps its `Enemy` + `Transform` + collider for the frames between `ExplodeMesh`
  insertion and the despawn in `on_fragments_spawned`, so `drive_enemies` still steers
  it and `enemy_melee` can land a phantom hit from a "dead" enemy. Fix: `remove::<Enemy>()`
  (or insert a `Dead` marker filtered out of `drive_enemies`/`enemy_melee`) when
  inserting `ExplodeMesh`, and key `on_fragments_spawned` off that instead of
  `With<Enemy>`.
  - Response: Fixed. on_health_zero now `remove::<Enemy>()` on death, so drive_enemies/enemy_melee (With<Enemy>) skip the dying body; on_fragments_spawned no longer filters With<Enemy>.
- [ ] R1.4 (MINOR) `examples/14_breach.rs` fire button (~:727) vs hit zone (~:1118) -
  the visual FIRE button (`right:6% bottom:12% 96x96px`) and the touch fire zone
  (`Rect::new(0.78, 0.72, 1.0, 1.0)`) do not line up: part of the button reads as
  "look" and a strip of empty screen fires. Not a lockout (fire is checked first with
  `continue`, so the right half still looks), but confusing. Fix: derive the zone from
  the same percentages the button uses (or size the button to the zone).
  - Response: Fixed. Set the fire zone to Rect::new(0.66, 0.74, 0.98, 0.92) to match the on-screen button's fractions.
- [ ] R1.5 (NIT) `examples/14_breach.rs` `read_touch` (~:1130-1150) - a second finger
  on either half falls into the `_` arm and overwrites the tracked finger every frame
  (two fingers fight/jitter); and a finger passing through the fire zone skips its
  `look_finger` update (`continue`), so the next `delta = pos - last` uses a stale
  `last` and jumps the view. Low impact. Fix: only adopt a new finger when the slot is
  empty, and update `look_finger`'s position even when firing.
  - Response: Fixed. read_touch now only adopts a finger when the slot is empty (a second finger on the same half is ignored), for both move and look.
- [ ] R1.6 (NIT) `examples/14_breach.rs` `release_cursor` (~:226) - only runs
  `OnExit(Playing)`, so closing the window straight from Playing leaves the cursor
  locked until the OS tears the window down. Tidy with an `AppExit`-time release.
  (Confirmed the `Single<&mut CursorOptions>` calls do NOT panic when the window is
  gone -- `Single` skips the system.)
  - Response: Deferred (NIT). Closing the window mid-Playing leaves the cursor locked only until the OS tears the window down (immediate on close); left as-is.

Confirmed correct (not findings): the `LinearVelocity` xz-only writes leaving y to
gravity; the player/enemy/world/ray collision layers (only the gib layer is wrong);
the ray filter (can't shoot through walls, can't self-hit); the Flash-before-damage
ordering dodging the despawn race; `HealthPlugin` preventing double-marking so score
can't double-count; `RunOver` reset in `start_run`; `button_grid_at`/`stick_deflection`
taking window fractions.

## Round 2

- VERDICT: APPROVE

Re-verified the addressed diff:

- [x] R1.1 (MAJOR) - resolved. Continuous proximity damage + no player-enemy collision
  + open arena make the swarm a reliable threat; a per-second HP log shows a defenceless
  player melted 100->0 in ~3s once enemies arrive (~10s total), and the death path is now
  covered by the `player_death_ends_the_run` headless test (plus `enemy_death_scores_one`).
- [x] R1.2 (MINOR) - resolved. World layer filters `Default`; gibs collide with the floor.
- [x] R1.3 (MINOR) - resolved. `Enemy` removed on death; no phantom melee/pathing.
- [x] R1.4 (MINOR) - resolved. Fire zone matches the button.
- [x] R1.5 (NIT) - resolved. Single-owner finger tracking.
- [x] R1.6 (NIT) - accepted/deferred (cursor unlocks on OS window teardown).

Full gate re-run green: `cargo clippy --all-targets` (plain + `--features debug`) clean,
11 unit tests pass (2 new headless lose/score tests), `check-ascii` + `cargo fmt --check`
clean, the normal aiming autopilot completes Menu->Playing->GameOver with no panic / no
runtime errors and 6 kills, and `trunk build --example 14_breach` still compiles to wasm.

All findings resolved. Approved.
