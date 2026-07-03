# Retro: Add runnable mesh slicer / explode example

- TASK: 20260703-110851
- BRANCH: feature/explode-example (merged to master, deleted)
- REVIEW ROUNDS: 2 (R1 APPROVE + 2 NITs, R2 both NITs resolved, no code change)

See TASK.md close-out for what shipped; this is process only.

## What went well

- "Graphical example, can't run it in a headless session" did not become an
  excuse for zero verification. Three complementary checks covered it: a
  MinimalPlugins integration test for the ECS observer path (CI-able), a
  real-GPU boot, and a throwaway 2s auto-explode run that pushed sliced
  fragments through the actual GPU upload. That last one closed the exact
  gap (live keypress) the headless test could not.
- Reusing `TempEntity` for fragment cleanup instead of hand-rolling a timer
  was the crate dogfooding the example should demonstrate.

## What went wrong

- Nothing shipped wrong, but I nearly applied review finding R1.1 (drop the
  "unused" `PhysicsPlugins`) on reasoning alone. It would have broken
  `--features debug`: the inspector pulls in avian's debug render systems
  that need the physics resources `PhysicsPlugins` installs. Root cause of
  the near-miss: the plugin looked unused from the example's own code, but
  its necessity comes from a *different* plugin added conditionally under a
  feature. Cross-feature/cross-plugin coupling is invisible from the call
  site.

## What to improve next time

- Before removing a plugin/dependency that "looks unused", run the build and
  run configs that could depend on it transitively - here that was one
  `cargo run --features debug`. The cost of the check (one run) was far
  below the cost of shipping a broken debug build. This is the same lesson
  as the earlier "verify before removing" retros, now extended to
  feature-gated transitive dependencies.

## Action items

- [x] Example shipped and covered by a headless plugin integration test.
- [ ] Optional enhancement (R1.2, not filed as a task): derive fragment
  explosion direction from each fragment's centroid relative to the origin
  for a more radial burst. Cosmetic; do only if a nicer demo is wanted.
- [ ] Still open from 20260703-110915: slice at the mesh centroid instead of
  the origin so off-origin meshes always explode. The example sidesteps
  this by centering its target at the origin; revisit only if a real game
  needs off-origin explosions.
