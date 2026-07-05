# breach -- ground pickups (HP / speed / fire-rate buffs)

- STATUS: OPEN
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
