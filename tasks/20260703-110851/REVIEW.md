# Review: Add runnable mesh slicer / explode example

- TASK: 20260703-110851
- BRANCH: feature/explode-example

## Round 1

- VERDICT: APPROVE

Verified independently:

- Delivers the goal: `examples/05_explode.rs` spawns a target, Space inserts
  `ExplodeMesh`, and an `On<Insert, ExplodeFragments>` observer spawns each
  fragment flying along its direction, despawns the shell, and respawns a
  target so it is re-runnable. Fragment cleanup reuses the crate's
  `TempEntity`, which is a nice bit of dogfooding.
- The observer/system wiring is correct: the exploded shell keeps its
  `Transform` (so `q_fragments.get` succeeds), fragment handles come from
  `Assets<Mesh>` (populated by the plugin), and only the shell carries
  `ExplodeFragments`, so the observer cannot fire for fragment entities.
  Removing the `Target` marker in the same command as inserting
  `ExplodeMesh` prevents a double-trigger.
- The headless `test_explode_mesh_plugin_produces_fragments` is the standout
  addition: it drives the real `ExplodeMesh -> ExplodeFragments` observer
  path in a `MinimalPlugins` app and asserts finite fragment meshes, so the
  example's core interaction is covered in CI even though the graphical app
  cannot run there. Combined with Task 20260703-110915's finiteness tests,
  the "won't crash" goal is genuinely proven headlessly.
- I re-ran the suite: fmt clean, clippy `--all-targets` clean, `cargo test`
  20 (unit) + doctests, ascii guard clean. The example compiles under
  `--features debug` too.
- Accept the reported manual verification (real-GPU boot + throwaway
  auto-explode run) as the runtime evidence; it is consistent with the
  headless test and the honest limits are documented.

Two non-blocking observations:

- [x] R1.1 (NIT) examples/05_explode.rs:16 - `PhysicsPlugins::default()` is
  added but unused; fragments use manual velocity, not avian bodies. It
  matches the other examples' habit of adding physics, so it is defensible,
  but dropping it (and the `avian3d` import) would make this example
  minimal and honest about what it exercises. Take it or leave it.
  - Response: rejected with evidence - do NOT remove it. I tested removing
    `PhysicsPlugins` and running `--features debug`: it fails at runtime
    with "Parameter `Collisions::contact_graph` failed validation: Resource
    does not exist" because `InspectorDebugPlugin` adds avian's
    `PhysicsDebugPlugin` whose `debug_render_contacts` system needs the
    physics resources `PhysicsPlugins` installs. That is why every example
    adds it. Keeping it is correct, not incidental.
- [x] R1.2 (NIT) examples/05_explode.rs:166 - `fragment.direction` is the
  last cut-plane normal, so fragments cluster into a few opposing
  directions rather than bursting radially. Fine for a slicer demo, but
  deriving each fragment's direction from its centroid relative to the
  origin would read more like an explosion. Enhancement only; the goal is
  to exercise the slicer, which it does.
  - Response: kept as-is. Using the slicer's own `direction` output is the
    point of the demo (it shows what the API returns); a centroid-based
    burst would be a nicer effect but is future enhancement, not this
    task's goal. Left for a possible follow-up.

No BLOCKER/MAJOR/MINOR findings. APPROVE.

## Round 2

- VERDICT: APPROVE

Both NITs resolved without code change: R1.1 was investigated and rejected
with a reproduced runtime error (removing `PhysicsPlugins` breaks
`--features debug`), so the example is correct as written; R1.2 is an
accepted future enhancement. No committed change since Round 1, so the
Round 1 suite result stands. Nothing further.
