# 08_dropzone: Tier-A fun pass (landing pad, fuel pickups, landing/crash visuals)

- STATUS: IN_PROGRESS
- PRIORITY: 5
- TAGS: feature,dropzone

## Goal

First and smallest of three follow-ups from the fun+mobile spike
(`tasks/20260704-102022`, read Part 1 Tier A first). Turn `examples/08_dropzone.rs`
from a physics demo into a small *game* by adding the whole Tier-A set, WITHOUT
touching the physics / PD controller / gravity or turning it into a bigger game
(no upgrades, no cargo, no multi-leg refuel runs -- those were explicitly cut).
Stay a small, copy-pastable crate demo.

Scope = spike Tier A only: A1 landing pad + positional score, A2 fuel pickups,
A3 optional efficiency/time term, A4 juice (visible landing + crash). No hazards
here (that is `tasks/20260704-103553`); no touch controls here (that is
`tasks/20260704-103517`).

## Baseline (from the spike, for grounding)

- Scoring is a pure function `landing_score(fuel, speed, tilt)` =
  `100 + fuel*3 + (4.5-speed)*40 + (0.35-tilt)*200`. Adding a positional term is
  a small, isolated change.
- Game ends on FIRST contact (Menu / Playing / Result states). Everything below
  avoids reworking that state machine.
- `START_FUEL` 100, `FUEL_BURN` 14/s. Fuel already dominates the score, so
  pickups meaningfully change routing.

## Steps

- [x] A1 -- landing pad + positional score. Spawn a glowing landing-pad / beacon
      marker on the surface (reuse the `07_orbit` glowing-marker look). Add a
      `proximity_bonus = (pad_radius - touchdown_dist).max(0) * k` term to the
      landing score so landing on/near the pad scores more than landing
      anywhere. Show the pad clearly (beacon + optional HUD direction hint).
      Decision to make + note: single pad vs a few; how "distance" is measured
      (great-circle on the surface vs straight-line).
- [x] A2 -- fuel pickups (fuel cans). Spawn a small set of floating fuel cells on
      the descent path, placed slightly OFF the efficient line so grabbing one
      trades altitude/control for fuel (a real risk/reward routing choice, not
      free candy). Detect fly-through (avian sensor collider or a simple
      distance check), add to `Fuel` (decide: cap at `START_FUEL` or allow
      overfill -- note the choice), play the existing `pickup` sound, and pop a
      rising "+FUEL" popup (reuse the `07_orbit` "+N" pattern via `helpers/temp`
      + `ui`).
- [x] A3 (optional, cheap) -- efficiency/time term. A visible descent timer or
      "descent score" that gently rewards committing to a line instead of
      hovering; one resource + one `ui/status` line. Skip if it muddies the
      scoring; note the decision either way.
- [x] A4 -- landing + crash visuals (juice). Make the outcome legible:
      - On a good landing, the ship visibly stays put / "sticks" on the pad
        (kill residual motion, maybe a small settle + a landing flash/dust
        puff), rather than the game just cutting to the Result screen.
      - On a crash, the hull explodes -- the `mesh/explode` crash path already
        exists; make sure it reads well (camera punch on impact, reuse the
        fragment system, maybe a soft dust puff even on a good touchdown).
      - Reuse `camera/post` bloom, `mesh/explode`, `helpers/temp`, `audio`; this
        is the `07_orbit` polish template (`feature/07-orbit-polish`).
- [x] Verify: `cargo fmt --check`, `cargo clippy --all-targets`, `cargo test`,
      `./scripts/check-ascii.sh`, then actually RUN the example (not just build:
      AGENTS.md gotcha) and confirm it reaches the render loop and each new
      mechanic works (pad scores, pickup adds fuel, land sticks, crash
      explodes). Rebuild the web/wasm showcase (`npm run build`) so the demo
      stays current. Update `docs/2026-07-03-dropzone-example.md` (or a new doc)
      with what changed and why.

## Added scope (user, 2026-07-04, during review)

The first pass shipped a fixed pad and fixed fuel-can positions. The user asked
to make each run fresh and easier to navigate. Acceptance criteria for this
branch (tracked as REVIEW.md R1.1-R1.3):

- [ ] Randomize the landing pad position each run (within the reachable cap
      around the +Y spawn pole, clear of the antipode singularity). Pad moves
      from a persistent setup() entity to a per-run entity; still visible on the
      result screen, cleaned up on leaving Result.
- [ ] Randomize fuel-can positions each run AND keep roughly 3 on the map by
      spawning replacements over time (never zero, never a swarm), like
      `07_orbit`'s maintain-objects pattern.
- [ ] Add a direction indicator toward the pad -- a diegetic guide (e.g. a
      world-space arrow that hovers by the ship and points along the ground
      track to the pad), not just the numeric "pad Nm" readout.

## Notes

- Distance-for-points from the original request is realized as A1
  (close-to-a-hard-pad), which the spike argues beats rewarding raw distance
  (raw distance perversely rewards flying off to nowhere).
- Deliberately OUT of scope (user cut): pickups-beyond-fuel, ship upgrades,
  cargo/weight, multi-leg refuel, multiple planets. Do not scope-creep.
- Keep faithful to AGENTS.md: additive, physics path untouched, example is the
  integration test, stay wasm-friendly, plain ASCII.
- Supersedes the earlier combined follow-up `tasks/20260704-102342` (now split
  into this + hazards + mobile tasks).

## Close-out

Implemented all four Tier-A items on branch `feature/08-dropzone-fun`
(`examples/08_dropzone.rs`), documented in
`docs/2026-07-04-dropzone-tier-a-fun.md`:

- A1 landing pad: emissive ring+beacon placed flush on the real terrain
  (evaluates the same noise the mesh uses); `proximity_bonus` in `landing_score`
  by great-circle distance; live "pad Nm" HUD hint. Single pad (decision noted).
- A2 fuel cans: 3 emissive canisters off the descent line; distance-check
  pickup; fuel capped at `START_FUEL` (decision noted); `pickup.wav` + a "+FUEL"
  floating popup ported from `07_orbit`.
- A3 timer: "t Ns" HUD + a `time_bonus` that never penalises a slow landing.
- A4 juice: dust puff (reusing `FragmentMotion`/`move_fragments` + `helpers/temp`)
  and a camera punch (`07_orbit` `CameraShake`, applied after
  `ChaseCameraSystems::Sync`); a soft landing freezes the hull (`RigidBody::Static`)
  and keeps it visible on the result screen. Crash keeps its proven
  spawn-fragments-then-`DespawnOnExit(Playing)` ordering.

Verified: 4 new in-module unit tests pass; fmt/clippy(--all-targets)/ascii
clean; ran the example (reaches render loop), and a temporary env-gated
autopilot (since removed) flew 741 physics frames to a clean soft landing
(score 497) with no gameplay panic. Web showcase rebuilt via `npm run build`.

Scope held: physics/PD/gravity and the state-machine shape untouched; no
out-of-scope mechanics (upgrades, cargo, multi-leg, extra pickups) added.
