# breach -- ground pickups (HP / speed / fire-rate buffs)

- STATUS: CLOSED
- PRIORITY: 65
- TAGS: spike,breach,example

## Goal

Add ground pickups to `14_breach` that grant the player short-term power: an HP
restore (via `HealthPlugin`), a movement-speed buff, and a fire-rate buff (shorten
the `time/cooldown` gate on the gun). This is the biggest new-fun lever and pairs
with the combo loop -- reward the aggressive player who wades into the swarm.

Delivery model to settle at plan time (see spike open questions): lean
drop-on-kill (a chance for a slain enemy to drop a pickup) to reinforce the combo
loop, possibly plus occasional ground spawns. Buffs are timed (expire after N
seconds via a small buff-timer component) rather than permanent stacking, to stay
arcade-y and avoid runaway scaling -- confirm at plan time.

Keep the pickup entity + pickup-on-proximity + timed-buff systems GAME-LOCAL for
now (like the FP controller was). Only flag a possible `powerup`/pickup crate
module as an explicit follow-up if a second example wants it.

## Notes

- Spike: docs/spikes/20260705-132024-breach-fun-pass.md
- Reuse: `health` (heal path), `time/cooldown` (fire-rate the buff shortens),
  `helpers/temp` (despawn un-grabbed ground pickups), `ui/popup`/`ui/status`
  (buff feedback), `audio` (pickup SFX -- see the sound-pass task).
- Pickup-on-proximity is a distance check or a sensor collider; the gun/enemy
  already use avian layers, so a sensor is idiomatic here.
- Pure logic (buff duration/decay math, drop-chance roll) gets `#[cfg(test)]`
  tests. Determinism note: examples avoid `Math.random`-style nondeterminism in
  tests; seed or make the roll injectable.
- Copy Bevy 0.19 UI/material idioms from an existing example; an HDR emissive
  pickup must NOT set `unlit: true` or it will not bloom.
- Verify: `cargo clippy --all-targets`, headless `BCS_AUTOPILOT` run, then run for
  real. If the pickup adds a game-driven state/resource change worth asserting,
  cover it with a headless `App` unit test.

## Steps

- [x] **Pickup kinds + buff state.** `PickupKind { Health, Speed, FireRate }`;
  `Pickup { kind }` component; `Buffs { speed_secs, firerate_secs }` resource; consts
  (drop chance, radius, lifetime, heal amount, buff durations + multipliers). Register
  `Buffs` and a `PickupDrops(Vec<(Vec3, PickupKind)>)` buffer; reset both in `start_run`.
- [x] **Drop on kill (observer stays logic-only).** In `on_health_zero`'s enemy branch,
  roll `PICKUP_DROP_CHANCE`; on a hit push (enemy `Transform.translation`, random kind)
  into `PickupDrops`. Query `&Transform` so the observer needs no Assets/UI -- the
  spawn system owns meshes. Init `PickupDrops` in the `death_app` test.
- [x] **Spawn + collect + animate (Playing systems).** `spawn_pickups` drains the buffer
  into glowing per-kind emissive meshes (green/cyan/orange, bloom via `camera/post`) at
  ground level, with `TempEntity(PICKUP_LIFETIME)` + `DespawnOnExit(Playing)`.
  `collect_pickups`: within `PICKUP_RADIUS` of the player, call the pure
  `apply_pickup(kind, &mut Health, &mut Buffs)` (heal caps at max; speed/firerate set
  their timers), play a pickup SFX, float a popup, despawn the pickup. `animate_pickups`
  spins/bobs them for juice.
- [x] **Wire buffs into movement + gun.** `tick_buffs` decrements the timers (clamped 0).
  `apply_speed_buff` sets `DoomController.move_speed = PLAYER_SPEED * SPEED_MULT` while
  active else base. `player_shoot` ticks `Gun.cooldown` by `dt * firerate_tick_scale(..)`
  so the fire-rate buff shortens the gate. Minimal buff HUD text (hidden when none).
- [x] **Tests + verify.** Unit-test `apply_pickup` (heal caps at max, each buff sets its
  timer) and the pure `buffed_speed`/`firerate_tick_scale` helpers, plus a `tick_buffs`
  decrement/clamp test. `cargo fmt`, `cargo clippy --all-targets`, `cargo test --example
  14_breach`, ascii check, headless `BCS_AUTOPILOT` run, real `cargo run` once. Update
  the `//!` header and the AGENTS 14_breach note.
