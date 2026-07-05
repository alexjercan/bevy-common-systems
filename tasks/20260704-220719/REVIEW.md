# Review: data-driven towers/enemies for 12_bastion + SpecCatalog evaluation

- TASK: 20260704-220719
- BRANCH: spike/bastion-data-catalog

## Round 1

- VERDICT: APPROVE

Reviewed `git diff master...HEAD`: the catalog JSON, the `Catalog` resource +
serde specs + loader, the threading of `Res<Catalog>` through the systems, the
generalized `select_build`/`spawn_enemy`, the doc updates, and the spike
write-up. Ran the full suite in the worktree: `cargo fmt --check`,
`cargo clippy --all-targets` (clean), `--features debug` (clean),
`cargo test`/`--examples` (94 unit + 51 doctests + example tests, incl. the 3
new catalog tests), `check-ascii.sh`, and `cargo check --target
wasm32-unknown-unknown` (the cfg-gated fs fallback compiles on wasm). Verified
gameplay: the autopilot cycle completes with no panic and a mid-run screenshot
renders correctly.

The two task halves are both delivered: (1) the stats are genuinely data-driven
and (2) the spike question is answered with a clear negative result and a
smaller reusable nugget identified. Notes are informational.

- [x] R1.1 (verified) The "no recompile" claim is real and was proven, not
  asserted: the same binary (unchanged mtime) logged `2 -> 3 -> 4 towers` across
  three runs as only the on-disk JSON changed. On wasm the embedded copy is used
  (no filesystem), and `cargo check --target wasm32-unknown-unknown` confirms the
  cfg gating compiles. Honest and correct.
- [x] R1.2 (verified) Behaviour is preserved for the existing roster: Gun /
  Cannon / Runner / Brute stats and colors are copied verbatim from the old
  arrays. The one deliberate change is the enemy spawn mix -- a weighted pick
  (`weighted_enemy_index`) replaces the old hard-coded `brute_chance` flip -- which
  is necessary to make a new enemy participate by data alone, and is documented.
- [x] R1.3 (NIT) The shipped `catalog.json` now carries a Sniper tower and a
  Swarm enemy (3+3), so the example ships with more content than before. This is
  intentional -- the added types are the living proof that the catalog is
  data-driven -- and the stats are plausibly balanced (Sniper: long-range,
  high-damage, slow; Swarm: fast, fragile, cheap). Called out as a content change,
  not a defect.
- [x] R1.4 (verified) The spike evaluation (`docs/2026-07-05-...`) answers the
  actual question: it reuses nothing from `modding/registry` (data vs behavior),
  the reusable nugget is the ~25-line loader not a `SpecCatalog<T>` type, and
  there is no second concrete user (09 uses the event registry). Keeping it
  game-local and seeding no speculative follow-up honours the task's explicit
  two-user rule.
- [x] R1.5 (verified) Test quality: `weighted_enemy_index` is tested including
  that an appended, otherwise-zero-weight enemy is always selected (proving
  selection is not hard-coded), and the embedded catalog parse + level-scaled
  upgrade cost are covered. Meaningful assertions, not execution-only.
