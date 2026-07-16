# Retro: 10_asteroids - slice-on-hit shooter with physics-body fragments

- TASK: 20260703-170744
- BRANCH: feature/10-asteroids (squash-merged to master as a3f7ef7)
- REVIEW ROUNDS: 2 (R1 REQUEST_CHANGES with 3 MAJORs, R2 APPROVE)

See `tasks/20260703-170744/TASK.md` and `tasks/20260703-170744/NOTES.md`
for what was built and why. This retro is about how the working went.

## What went well

- Front-loaded the avian 0.7 source reading. This example used a large slice of
  avian that no prior example touched (sensors, the `PhysicsLayer` derive +
  `CollisionLayers` filter combining, `Restitution`/`Friction`, `LockedAxes`,
  and crucially whether a kinematic body advances by its `LinearVelocity`).
  Every one of those was verified against the avian source before writing --
  reading `integrate_positions`, the collision-event docs, the layer `From`
  impls -- so a ~1050-line physics-heavy example compiled first try with zero
  API errors. Same win as the dropzone cycle: read the engine, do not guess.
- The headline feature got a real headless integration test
  (`splitting_a_rock_spawns_smaller_physics_bodies`) that drives the actual
  `ExplodeMesh` -> `on_fragments_spawned` path and asserts the sliced rock is
  replaced by generation-1 dynamic bodies. That is the CI-visible proof the
  graphical example cannot be, and it is worth more than the pure-helper tests.
- Ran the example on the display and it caught a `B0004` visibility warning
  (mesh-less ship parent) that a pure `cargo build` would have missed -- the
  standing "run it, do not just build it" lesson paid off again.
- The review step did its job: it found three real bugs, each verified against
  Bevy source (the `unlit`/emissive one by reading `pbr.wgsl`), not hand-waved.

## What went wrong

All three review findings were in the **visual / rendering layer**, and all
three share one root cause: the logic and physics were verified meticulously
against source, but the material / hierarchy / camera details were written from
memory or improvised instead of checked or copied from a known-good example.

- R1.3 (`unlit: true` suppresses emissive, so the bullets and thruster flame do
  not bloom): I assumed `unlit` still adds emissive. Bevy's PBR shader takes
  `out.color = base_color` and skips the whole lighting pass (where emissive is
  applied) when unlit. This defeats the task's explicit bloom requirement and
  was pure guessing about a rendering detail.
- R1.1 (camera shake accumulates and drifts off-center): I wrote the shake as
  `translation += offset` with an early return that never re-centers, instead of
  copying `06_fruitninja`'s known-good `translation = CAMERA_BASE + offset`
  (absolute from a fixed base). I improvised a relative version of a pattern that
  already existed, correct, three files over.
- R1.2 (thruster flame did not rotate with the ship): a hierarchy miss -- I made
  the flame a sibling of the model child that gets rotated, so it never turned
  with the heading.

This is a REPEAT of the dropzone cycle's meta-lesson at a new layer. That retro
(`20260703-165432`) logged "verified the exotic APIs meticulously but wrote the
text/light idioms from memory" and added an AGENTS.md gotcha to copy Bevy visual
idioms from an existing example. Same pattern here, now in materials, visibility
and camera math rather than text/light -- so the gotcha was broadened from
"text/light" to the visual layer generally.

Note also: running the example catches boot / panic / hierarchy-warning issues,
but NOT visual-correctness issues (bloom, flame orientation, slow camera drift),
because a background session cannot see the screen. Those must be caught by
reviewing against the rendering model, which is exactly what happened -- but
ideally implementation catches them first.

## What to improve next time

- Treat the visual layer with the same "verify before writing" rigor as the
  logic layer. Before shipping emissive/bloom, confirm the material is not
  `unlit` (unlit skips emissive). Before shipping a rotating/authored entity
  with mesh children, give the parent an explicit `Visibility` and put children
  that must follow its rotation UNDER the rotated node, not as siblings.
- Copy known-good visual patterns verbatim from an existing example rather than
  improvising: camera shake is `base + offset` (absolute), copied from
  `06_fruitninja`, not `+= offset`.

## Action items

- [x] Broadened the AGENTS.md Bevy-idioms gotcha from "text/light" to the visual
  layer, with the three specific traps this cycle hit (emissive must not be
  `unlit` to bloom; a mesh-less parent needs `Visibility`; camera shake is an
  absolute offset from a fixed base).
- [ ] No follow-up code tasks. The example is self-contained and merged; the
  wasm gallery entry is wired and the trunk build was verified.
