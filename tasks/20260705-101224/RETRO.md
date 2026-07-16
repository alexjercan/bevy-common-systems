# Retro: 12_bastion juice - build particles, Core explosion, extra shake

- TASK: 20260705-085338
- BRANCH: feature/bastion-juice (squash-merged to master)
- REVIEW ROUNDS: 1 (APPROVE with 1 NIT, fixed in-round)

Fourth and final task of the 12_bastion polish flow: spark bursts on build /
upgrade, a Core that shatters on death, and shake on build / new waves.

## What went well

- Went straight at the highest-risk design decision (how to explode a PERSISTENT
  Core without breaking its revive) by reading the setup + start_game + the prior
  retro FIRST. The chosen shape -- explode a short-lived mesh COPY and merely HIDE
  the real Core, restored by start_game -- fell out cleanly and never touched the
  revive gate (health reset + HealthZeroMarker removal). The prior cycle
  explicitly flagged Core-revive as the top risk; treating it as the first
  question, not the last, paid off.
- Verified that exact risk deterministically with a focused integration test
  (`start_game_revives_hidden_core`): a hidden/zero-health/marked Core is revived
  to Visible + full health + unmarked. That is the "verify the effect, not a
  proxy" lesson turned into a permanent regression guard, and it needed no state
  machinery (start_game does not read State), so it stayed a cheap, fast test.
- Reused the crate kit end to end (TempEntity, glowing_material,
  ExplodeMesh/on_fragments_spawned, CameraShakeInput) with zero new deps, and
  used HDR `glowing_material` for the sparks so they bloom -- avoiding the
  documented `unlit`-kills-bloom footgun by habit now.
- The review NIT (per-placement material allocation) turned into a real
  improvement that also matched the crate convention: cached per-spec burst
  materials in GameAssets exactly like `enemy_materials`, removing an allocation
  on every build.

## What went wrong

- Burned two runs re-learning the same headless-run lesson from earlier this
  flow: first I ran the raw `./target/debug/examples/12_bastion` binary, which
  resolves assets relative to the binary dir (not the crate root), so every sound
  failed to load and the run timed out; then a `cargo run --features debug`
  timed out because the debug-feature rebuild + shader warmup ate the whole
  timeout budget. Root cause both times: not separating "build" from "run" and
  not using cargo's crate-root CWD. The fix that worked -- `cargo build
  --features debug` first, THEN a short `cargo run` -- is the same pattern the
  packs task already established. I should have reached for it immediately.
- The build-bar integration test broke when I widened `try_upgrade_selected`'s
  query to `(&mut Tower, &Transform)` and added a `GameAssets` dependency; the
  test's Tower had no Transform and there was no GameAssets resource. Caught by
  running the tests, not by review, but it is a reminder that changing a shared
  helper's system-param surface ripples into every test that drives it.

## What to improve next time

- For any headless example verification: `cargo build --example NN [--features X]`
  as a distinct step, then `cargo run` (never the raw binary -- it breaks asset
  CWD). Use a timeout that assumes a cold shader cache on first run. This is now
  the third time this flow that the build-vs-run / CWD distinction cost a run;
  it belongs in muscle memory.
- When widening a shared helper's query or params, grep its call sites AND its
  tests in the same edit, and update the test fixtures before running.
