# Retro: data-driven towers/enemies for 12_bastion + SpecCatalog spike

- TASK: 20260704-220719
- BRANCH: spike/bastion-data-catalog (squash-merged to master)
- REVIEW ROUNDS: 1 (APPROVE, five informational notes)

A spike task with two halves: make 12_bastion's stats data-driven (concrete),
and decide whether the mechanism becomes a crate module (open question). The
answer to the second half is a negative result, which is a success for a spike.

## What went well

- Proved "no recompile" instead of claiming it. The temptation was to say "it
  reads a file, therefore no recompile". Instead I built the binary once, then
  ran it against 2/3/4-tower JSON with the mtime pinned, and captured the startup
  log changing each run. That turned a plausible assertion into evidence a
  reviewer could not wave away, and it caught nothing broken -- but it would have
  caught an accidental `include_str!`-only path.
- Made the loader honest about platforms up front. Native reads the file (the
  whole point), wasm has no filesystem, so `#[cfg(not(wasm))]` fs with an
  `include_str!` fallback -- and a parse-error fallback so a fat-fingered JSON edit
  logs and keeps playing rather than panicking. Checking `cargo check --target
  wasm32` (which the nix devshell supports even though rustup does not list it)
  confirmed the gating before review, not after.
- Generalized the two roster-dependent code paths so the data-driven claim is
  real, not cosmetic. It would have been easy to move the stats to JSON but leave
  `spawn_enemy` hard-coded to "index 0 vs index 1" -- then a third enemy loads but
  never spawns. Adding `spawn_weight`/`wave_weight` and a weighted pick (plus
  `digit_key` for the build bindings) is what makes a new JSON entry actually
  play. Unit-tested that an appended enemy is selectable.
- Answered the spike's real question with a distinction, not a vibe: the catalog
  names *data* (serde structs), the reactor's registry names *behavior*
  (constructors -> trait objects). They look adjacent in the file tree but share
  no machinery, so "reuse modding/registry" was a false lead, and the only
  reusable nugget is the tiny loader. Naming that kept me from building a
  `SpecCatalog<T>` nobody has a second use for.

## What went wrong

- Shipped a test that the "prove it" step then broke. `embedded_catalog_has_the
  _starter_specs` asserted 2 towers / 2 enemies, but the embedded catalog IS
  `catalog.json` via `include_str!`, and step 4 (add Sniper + Swarm) changed that
  file to 3+3. `cargo test --example` passed while the JSON was still 2+2 and
  failed only after the proof edit. Root cause: I wrote the roster-count
  assertion against the file's *current* contents instead of against what the
  game *ships*, so editing the shipped data (the whole feature) invalidated it.
  Fixed by asserting the intended shipped roster (Gun/Cannon/Sniper,
  Runner/Brute/Swarm) explicitly.

## What to improve next time

- When a test asserts the contents of a data file that the same task edits,
  assert the *final shipped* state, and re-run `cargo test --examples` AFTER the
  data change, not just after the code change. A green example-test run before the
  data edit is a false green for anything that embeds that data. (Sibling of the
  existing "bare cargo build skips examples" gotcha: here it was "example tests
  run before the data they embed was finalized".)

## Action items

- [x] 12_bastion towers/enemies data-driven from JSON; Sniper + Swarm added in
  JSON; loader + selection generalized; spike evaluation written.
- No follow-up task seeded on purpose: promoting the JSON loader waits for a
  second concrete user (the two-user rule the spike enforced). If a second game
  wants edit-JSON-no-recompile asset loading, extract the ~25-line loader (not a
  `SpecCatalog<T>` type) then.
