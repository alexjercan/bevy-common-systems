# Review: camera/shake CameraShake trauma module

- TASK: 20260704-134500
- BRANCH: worktree-camera-shake

## Round 1

- VERDICT: APPROVE (with MINOR/NIT findings addressed at implementer discretion)

Independent review traced the Restore/Apply two-phase logic for both a static
frame and a chase frame and confirmed `driver_base + offset` with no
accumulation/drift; first frame is safe (default state -> zero offset, identity
kick); rotation restore order is correct; the 06 refactor preserves behavior
(same decay/offset, z-offset stays 0, trauma routed through `CameraShakeInput`,
`reset` on start); conventions match `chase.rs`/AGENTS.md. Full check suite
passes.

- [x] R1.1 (NIT) src/camera/shake.rs:263 - the comment "reset wins over an add
  in the same frame" is inaccurate: the code clears trauma on `reset` and then
  still applies a same-frame `add_trauma` on top, so the add lands. Fix the
  comment (or `else if` the add if reset should truly win). Harmless in 06.
  - Response: Fixed the comment to describe the actual behavior (reset clears
    first, a same-frame add still lands). This is the intended behavior -- a
    reset is a floor, not a veto -- so the code is left as is.
- [x] R1.2 (MINOR) src/camera/shake.rs:401-427 - the anti-drift test only
  exercises the static base case; the "composes with a moving base written
  between Restore and Apply" claim (chase / custom driver) has no test. Add a
  test with a dummy driver system `.after(Restore).before(Apply)` that rewrites
  translation each frame, and assert the camera tracks the moving base within
  `bound` and settles exactly on it.
  - Response: Added `composes_with_a_moving_base_driver`, which inserts a driver
    that sets translation to a per-frame-varying base between Restore and Apply
    and asserts the camera stays within `bound` of that base and, once trauma
    decays, sits exactly on it.
- [x] R1.3 (NIT) src/camera/shake.rs:429-453 - rotational kick has only a
  pure-math test; no system-level test proves the kick recenters to the base
  rotation after decay (guards the `inverse()` restore order). Optional given
  v1 scope.
  - Response: Added `kick_recenters_rotation_after_decay` with a non-zero
    `max_kick`, asserting the camera rotation returns to the base rotation once
    trauma decays.

## Self-found (implementer), folded into Round 1

- [x] R1.4 (MAJOR) src/camera/shake.rs:191-204 - `Restore` is ordered only
  `.before(ChaseCameraSystems::Sync)` and `Apply` `.after` it. When the chase
  plugin is absent (e.g. 06, and any static-camera game), that set is empty, so
  there is no ordering edge between Restore and Apply -- they are unordered yet
  both mutably access `Transform`. The tests pass only because the executor's
  ambiguity resolution happens to run them in insertion order on this build;
  Bevy does not contract that. If `Apply` ran before `Restore`, the shake would
  drift -- reintroducing exactly the bug this module exists to prevent. Make the
  ordering explicit and independent of chase.
  - Response: Added `app.configure_sets(PostUpdate,
    CameraShakeSystems::Apply.after(CameraShakeSystems::Restore))` so Restore
    always precedes Apply regardless of whether any base driver sits between
    them. The new moving-base test (R1.2) also fails fast if that ordering
    breaks.

## Round 2

- VERDICT: APPROVE

All Round 1 findings verified resolved on the updated diff:

- R1.1 comment now matches behavior.
- R1.2 `composes_with_a_moving_base_driver` added and passing -- drives a base
  rewritten between Restore and Apply, asserts tracking within bound and exact
  settle.
- R1.3 `kick_recenters_rotation_after_decay` added and passing.
- R1.4 `configure_sets(PostUpdate, Apply.after(Restore))` added, so the
  Restore-before-Apply invariant no longer depends on the chase set being
  populated.

Full check suite green: fmt, clippy (both configs), 38 lib tests + 14 doctests,
`cargo test --examples`, check-ascii. Module unit + ECS tests: 11 passing.
