# Review: 07_orbit - surface-dodge game on a sphere (Orbit Runner)

- TASK: 20260703-165427
- BRANCH: feature/07_orbit

## Round 1

- VERDICT: REQUEST_CHANGES

Scope check: the diff adds `examples/07_orbit.rs` plus placeholder sounds, the
web-gallery wiring, and docs. It delivers the Goal -- steer a marker with
`DirectionalSphereOrbit`, wander hazards/orbs with `RandomSphereOrbit`, follow
with a `LerpSnap` `ChaseCamera`, lose via `HealthPlugin`, in the `06_fruitninja`
menu/playing/game-over + `SfxPlugin` + wasm/trunk shape. Full check suite is
green locally: `cargo fmt --check`, `cargo clippy --all-targets` (both with and
without `--features debug`), `cargo test --example 07_orbit` (10 pass) and the
full `cargo test`, and `scripts/check-ascii.sh`. The one blocking issue is a
gameplay-correctness defect around spawning.

- [ ] R1.1 (MAJOR) examples/07_orbit.rs:`spawn_wanderer` / `maintain_objects` -
  hazards and orbs spawn at fully random `(theta, phi)` with no guard against
  landing on the marker. `maintain_objects` tops the field up every frame, so a
  new hazard added on a level-up (or any of the initial hazards on the first
  frame of a run) can materialize within collision range of the marker and deal
  unavoidable damage the player had no way to dodge. For a "basic but fun"
  example this reads as a bug, not difficulty. Fix: pass the marker's current
  surface direction into `spawn_wanderer` and reject candidate positions whose
  unit direction is within a minimum arc of it (resample a few times), so
  nothing ever spawns on or right next to the player. Using
  `meth::spherical_to_cartesian` for the candidate direction also makes the
  example exercise `meth` directly, which the task calls out.
  - Response:

- [ ] R1.2 (NIT) examples/07_orbit.rs:`frame_rotation` - builds the basis with
  bare `Vec3::normalize()`, which yields a NaN quaternion if `up`/`forward` are
  ever degenerate (parallel or zero). `step_runner_frame` is careful to keep the
  frame orthonormal and uses `try_normalize` with a fallback, so this cannot
  trigger today, but the asymmetry is a latent trap. Consider `try_normalize`
  with an identity/fallback here too, or a short comment stating the invariant
  the caller guarantees.
  - Response:

- [ ] R1.3 (MINOR) examples/07_orbit.rs - the file is ~1230 lines against the
  task's "~1000 LoC" scope note. Roughly 130 of those are unit tests and a large
  share is doc comments, so it is not bloated, but it is over the stated budget.
  Acceptable as-is; flag only so the overage is a conscious call, not drift.
  - Response:

- [ ] R1.4 (MINOR) verification gap - the 10 in-file example tests are not run
  by CI (`cargo test` does not execute `examples/` tests; this is the
  pre-existing gap tracked by follow-up task 20260703-175735, not introduced
  here). They were run locally this round. Separately, the wasm/trunk build for
  `07_orbit` is being verified through the real `trunk build` entry point this
  round rather than assumed from the mirrored 06 setup. No code change required;
  recorded so the coverage boundary is explicit.
  - Response:
