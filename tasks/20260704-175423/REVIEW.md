# Review: scoring::HighScore<T> best-score resource

- TASK: 20260704-175423
- BRANCH: feat/high-score

## Round 1

- VERDICT: APPROVE

A clean leaf harvest with the right design calls.

**Folding `NewBest` into `HighScore<T>` is the right move.** The games kept two
resources (`HighScore` + `NewBest`) that are always updated together;
`record(score) -> bool` computes and stores both at once (`is_new_best()` for the
"New best!" branch, `best()` for "Best: N"), and a strict `>` correctly makes a
tie *not* a new best -- the exact rule 09's comment spelled out. Generic over
`PartialOrd + Copy` fits the usize/f64/f32 spread.

**serde/persist composition is handled correctly.** `#[serde(skip)]` +
`#[reflect(ignore)]` on `new_best` means only the best value is persisted (it is
per-run state), verified by a test asserting the JSON is `{"best":42}` and that
the flag defaults to false on load. So `PersistPlugin::<HighScore<usize>>` works,
which is the intended pairing.

**Refactors are faithful.** 06 (usize) keeps persisting via the crate type and
drops its `HighScore`/`NewBest` structs; its record chain was already
record-then-spawn, so `is_new_best()` reads correctly. 09 (f64) is the better
second example -- it stresses the generic *and* its game-over chain was reordered
from spawn-then-record (reading the stale best) to record-then-read, which is
behaviour-equivalent (both compute `final_score > old_best` strictly) and cleaner.

**No collateral breakage.** The un-migrated games (07/10/11) still carry a local
`struct HighScore` that now shadows the prelude's `HighScore<T>` glob import;
local items shadow glob imports without ambiguity, so they compile untouched
(confirmed by a clean `clippy --all-targets`). Naming does not collide with
`bevy::prelude`.

Tests are meaningful: default, record-updates-and-reports, a-tie-is-not-a-new-
best, works-for-float-scores, clear-new-best, and the serde-skip round-trip.

Verified in the worktree: `cargo fmt --check`, `cargo clippy --all-targets`
(default and `--features debug`), `cargo test` (91 unit + 46 doctests) and
`scripts/check-ascii.sh` all pass; 06 and 09 boot with no panic.

- [x] R1.1 (NIT) - the persisted format for 06 changed from a bare `7`
  (`HighScore(usize)`) to `{"best":7}` (`HighScore<usize>`), so an existing save
  will fail to parse and reset to the default once. Harmless for the example
  (placeholder saves, and `PersistPlugin` falls back to default on a parse
  error), but note it: a real game upgrading types would drop the stored score.
  Informational, no action.
  - Response: Acknowledged; accepted as the intended type going forward. Example
    saves are placeholders and persist falls back to default on a parse error.
- [x] R1.2 (NIT) examples/ - only 06 and 09 were migrated (the task's "a
  couple"); 07/08/10/11 still hand-roll `HighScore`/`NewBest`. Worth folding a
  `HighScore` migration into the existing SoundBank-migration follow-up (tatr
  20260704-223846) so the remaining games adopt both at once. Not blocking.
  - Response: Done -- folded a HighScore migration section into tatr
    20260704-223846 (now covers both SoundBank and HighScore for 07/08/10/11).
