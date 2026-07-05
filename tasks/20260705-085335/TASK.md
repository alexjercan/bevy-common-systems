# Bastion: enemies spawn in packs + steeper difficulty ramp

- STATUS: OPEN
- PRIORITY: 85
- TAGS: feature,example,bastion

> Part of the 12_bastion polish goal. Today enemies trickle out one at a time
> every `SPAWN_INTERVAL` seconds within a wave, and difficulty scales only gently
> (hp +18%/wave, speed +5%/wave, count +2/wave). The user wants enemies to spawn
> in PACKS (a burst of several appear together) and a more pronounced difficulty
> ramp as the game goes on.

## Goal

Rework the wave scheduler in `examples/12_bastion.rs` so a wave releases enemies
in packs (groups that spawn together at spread-out bearings), and make the ramp
steeper and more felt: bigger packs, more packs, tighter pacing and tougher
enemies as waves climb. Keep it data-driven-friendly (still iterate the catalog,
no hard-coded roster) and keep the pure scheduling math in testable functions.

## Steps

- [ ] Add pack tuning constants near the existing wave constants
      (`12_bastion.rs:109-116`): e.g. `PACK_BASE` (enemies per pack at wave 1),
      `PACK_PER_WAVE` growth, `PACKS_BASE`/`PACKS_PER_WAVE` (packs per wave), and
      `PACK_GAP` (seconds between packs, distinct from `WAVE_GAP`). Reduce
      `SPAWN_INTERVAL`'s role to the fast intra-pack stagger (or spawn a pack
      instantly with a tiny per-enemy jitter of angle).
- [ ] Extract pure functions and unit-test them (per retro: test the formula,
      not a tautology against the same const):
      - `pack_size(wave) -> usize` and `packs_in_wave(wave) -> usize`, so
        `wave_size(wave) == pack_size * packs_in_wave` (redefine or keep
        `wave_size` consistent - the game-over screen and HUD read the wave
        number, and `advance_waves` needs the total).
      - Assert packs and pack size both grow with wave, and total enemies grow
        faster than the old linear `WAVE_BASE + wave*WAVE_PER`.
- [ ] Rewrite `WaveState` + `advance_waves` (`12_bastion.rs:464-1046`) to a
      pack model: a wave has N packs; when the pack timer elapses, spawn a whole
      pack at once (loop `spawn_enemy` `pack_size` times, each at an independent
      random ring bearing so the pack fans across the border), decrement the pack
      counter, and reset the pack timer to `PACK_GAP`. When all packs are spawned
      AND the field is clear, wait `WAVE_GAP` then open the next (bigger) wave.
- [ ] Steepen the stat ramp: bump `WAVE_HP_PER` / `WAVE_SPEED_PER` (or make hp
      ramp mildly super-linear) so late waves feel harder, without making wave 1
      unwinnable. Keep the values in named constants with doc comments.
- [ ] Play a `Sfx::Wave` cue and (optional, coordinate with the juice task) a
      small shake at each new wave; do not double-add shake that the juice task
      will own - leave a note. Keep the existing `Sfx::Wave` on wave start.
- [ ] Update the module `//!` doc line about waves ("Waves ramp the count, speed
      and toughness") to mention packs, and the existing `wave_size_grows` test
      stays meaningful (update it for the new formula).

## Verification

- `cargo clippy --all-targets` clean; `cargo test --examples` runs the new pure
  tests; `cargo fmt --check`; `./scripts/check-ascii.sh`.
- Headless autopilot run (`BCS_AUTOPILOT=1 ... --features debug` under `timeout`)
  reaches Playing, and the log/behaviour shows multiple enemies appearing
  together (add a temporary `debug!` counting a pack spawn if needed, then
  remove it). Confirm the run still completes the autopilot cycle with no panic.
