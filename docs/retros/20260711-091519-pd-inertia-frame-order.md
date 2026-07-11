# Retro: PD controller inertia frame composition order

- TASK: 20260711-091519
- BRANCH: fix/pd-fast-spin-damping (squashed to master as 13e33e5)
- REVIEW ROUNDS: 2 (R1: 1 MAJOR, 1 MINOR, 1 NIT; R2: APPROVE)

## What went well

- Repro-before-fix paid off twice. The plan encoded a falsifiable
  contingency ("if the integration repro does NOT reproduce, the bug is in
  the consumer - record and stop") and that branch is the one that fired:
  the crate-level despin tests passed on the pre-fix code, which cleanly
  localized nova's corkscrew to nova's side instead of shipping a
  plausible-but-wrong "this fixes the game" claim.
- Plan-time math analysis (an SPD tensor drains energy under EITHER
  composition order) was written into the task before implementation, so
  the surprising repro result was expected rather than confusing.
- Verifying the frame convention against the defining dependency
  (bevy_heavy's `new_with_local_frame` source) instead of reasoning from
  the local code alone made the one-line fix defensible.

## What went wrong

- R1.1 (MAJOR): the planned step said "NON-identity principal local frame"
  but the implemented integration body was three axis-aligned cuboids -
  identity frame - and the step got ticked anyway. Root cause: while
  mirroring nova's evidence rig for fidelity, the step's distinct purpose
  (exercise the fixed code path end to end) was silently dropped; fidelity
  and coverage were two requirements hiding in one step.
- The closed-form test initially re-derived the same composition as the
  implementation (R1.2) - an oracle that cannot catch a shared
  misunderstanding. Cheap to avoid by building oracles from the dependency
  from the start.
- Two harness potholes cost a compile-run round each: avian needs
  AssetPlugin + MeshPlugin even for primitive colliders, and its
  diagnostics resources appear only after `app.finish()`. Recorded in
  TASK.md; the Gotchas section now has a place to point at (see Action
  items).

## What to improve next time

- When a plan step bundles two intents (mirror the consumer AND cover a
  specific code path), split them into two checkboxes so ticking one
  cannot silently satisfy the other.
- Test oracles for convention-sensitive math (frame order, sign, handedness)
  should be constructed by the library that defines the convention, never
  re-derived next to the code under test.
- Headless avian test apps in this crate: MinimalPlugins + TransformPlugin +
  AssetPlugin + MeshPlugin + PhysicsPlugins, then `app.finish()`. Copy
  `physics::pd_controller::tests::physics_app`.

## Action items

- [x] R1.1-R1.3 fixed in-round.
- [ ] nova-protocol task 20260709-125640 continues consumer-side: the
      corkscrew mechanism lives in nova's command shaping or torque
      application, not in this crate.
