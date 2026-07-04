# Review: assets - SoundBank registry + opt-in ready-gate

- TASK: 20260704-175422
- BRANCH: feat/assets-registry

## Round 1

- VERDICT: APPROVE

Delivers the harvest the spike asked for, and the design choices are sound.

**Generic key.** `SoundBank<K>` over a game-defined `Copy` enum is the right call
-- the sound *set* genuinely differs per game, so a fixed enum could not fit all
six. The `SoundKey` blanket trait (`Copy + Eq + Hash + Debug + Send + Sync +
'static`, auto-impl'd) means a game just derives those on its enum and never
names the trait. `load([(key, name)])` applying the `sounds/<name>.wav`
convention is the real dedup: it removes the `"sounds/"` prefix and `".wav"`
suffix that every game repeated, and centralises the load loop.

**Does it beat the struct-bag?** On its own the map is close to a lateral move
(it trades the struct's compile-time field access for a runtime `get()` that
panics on a missing key). What tips it over the line is the opt-in gate:
`all_loaded` + the `sounds_loaded::<K>` run-condition are genuinely new -- no game
has a loading gate today, and async handle-readiness (especially on wasm) is
exactly the plumbing a utility crate should own once. Keeping it a run-condition
rather than a forced plugin honours the spike's "make the gate opt-in" answer.
The `get()` panic is a deliberate, documented tradeoff, softened by `try_get`
and the exhaustive-load-list convention.

**Refactor.** 06 (8 sounds) and 09 (6) are cleanly moved off `SfxAssets`; 09 is a
nice demonstration that the semantic key decouples from the file
(`Tap -> pickup.wav`, `Buy -> golden.wav`), which the struct did by field name
and the bank does by the `(key, name)` pair. Verified statically that every
`get(Sfx::X)` key is present in each game's load list (so no runtime panic), and
both boot to the render loop with no panic.

Tests are meaningful: `load` builds a keyed bank with distinct handles, `get`
panics on a missing key, and the gate is covered on both branches (empty bank ->
vacuously loaded; pending handle -> not loaded). The per-handle `is_loaded`
check is the trusted Bevy API boundary. Naming does not collide with
`bevy::prelude`.

Verified in the worktree: `cargo fmt --check`, `cargo clippy --all-targets`
(default and `--features debug`), `cargo test` (85 unit + 45 doctests),
`scripts/check-ascii.sh`; 06 and 09 boot with no panic.

- [ ] R1.1 (NIT) examples/ - only 06 and 09 were migrated (the task asked for "a
  couple"), so 07/08/10/11 still hand-roll `SfxAssets`. The payoff grows with
  adoption and the split is momentarily inconsistent; worth a follow-up task to
  migrate the remaining four. Not blocking.
  - Response: Filed as follow-up tatr 20260704-223846 (migrate 07/08/10/11),
    per the flow "new work becomes a task" rule. 06/09 meet the task's "a couple"
    proof bar.
- [x] R1.2 (NIT) src/audio/registry.rs:143 - the `sounds_loaded` gate is
  unit-tested for the empty/pending cases and shown compiling in a doctest, but
  no test drives an actual `Loading -> Menu` transition through it end to end.
  The primitive is sound (the state-transition half is pure Bevy); noted for
  completeness, no action required unless a game adopts the gate.
  - Response: Accepted as-is. The gate's logic is unit-tested on both branches and
    the wiring compiles in the doctest; a real Loading->Menu adoption (with its
    own integration test) is left to the follow-up when a game opts in.
