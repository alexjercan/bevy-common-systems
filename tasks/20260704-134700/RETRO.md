# Retro: persist - cross-platform PersistPlugin<T>

- TASK: 20260704-134700
- BRANCH: feat/persist (squash-merged to master as dae2e03)
- REVIEW ROUNDS: 1 (APPROVE; MINOR + 2 NITs, all addressed/accepted)

The juice-kit's "one primitive every game lacks": save/load a Resource across
launches on native and wasm. The first crate module to add a real cross-platform
dependency surface, so the build-vs-depend call and the wasm verification carried
the weight.

## What went well

- Surfaced the build-vs-depend fork to the user before implementing (the task
  asked for it), with the dependency implications laid out. "Hand-roll" was the
  charter-consistent answer and the user confirmed it, so no rework.
- Booting 06 caught a genuine ordering panic that no test would have: `spawn_menu`
  (`OnEnter(Menu)`) reads the persisted resource, but a `PreStartup` load system
  runs AFTER the initial state transition, so the resource did not exist yet.
  Moving the load into `Plugin::build` (a synchronous file read, the right place
  to seed initial resource state) fixed it. This is the "an example is not done
  until it has run" rule earning its keep a second time this session.
- Verified the cross-platform surface as far as the environment allows:
  `cargo check --target wasm32-unknown-unknown` proved the web-sys backend
  compiles (the nix devshell has the wasm target), and a hermetic two-`App`-run
  test -- redirected to a temp dir via a `BCS_PERSIST_DIR` override -- is the
  deterministic "survives across launches" proof, not just a boot.
- The review's own path-traversal MINOR was cheap to close (an `is_safe_key`
  guard + a pure test), and I caught that adding an env-touching test for it would
  reintroduce the very `BCS_PERSIST_DIR` race the review had just flagged -- so I
  kept the guard test pure. The review fed back into itself.

## What went wrong

- The first cut loaded in a `PreStartup` system, assuming that runs before
  `OnEnter`. It does not (state transitions run first). Root cause: assumed a
  schedule ordering instead of checking it, and only the boot exposed it. A
  headless test with states + a resource-reading `OnEnter` would have caught it
  too, but I did not think to write one until the panic pointed at it.

## What to improve next time

- When a plugin must make a resource available "before the game reads it", prefer
  seeding it in `Plugin::build` (immediate, ordering-proof) over a startup system,
  unless the load genuinely needs the running `World`. Startup-system timing vs
  state transitions is a known trap.
- The `$BCS_PERSIST_DIR` override doubled as the test seam; designing the escape
  hatch and the test fixture together (rather than reaching for `set_var` on the
  real path) is what made the end-to-end test hermetic. Reach for an injectable
  root up front for anything touching the filesystem.

## Action items

- [x] `persist::PersistPlugin<T>` shipped (native `dirs`+serde_json, wasm web-sys
  localStorage); 06 high score persists. Key guard + two-launch test added.
- [ ] Remaining juice-kit Wave 2: `spawn + time/cooldown` (tatr 20260704-134730),
  sketch-then-commit. Plus the tween follow-up (tatr 20260704-201801, popup/flash
  onto Tween). Dev-harness spike's Wave 2 (175422-425) stays the parallel
  session's.
