# Retro: generic HighScore<T> best-score resource

- TASK: 20260704-175423
- BRANCH: feat/high-score (squash-merged to master as 090d98a)
- REVIEW ROUNDS: 1 (APPROVE, two informational NITs)

Second dev-harness Wave 2 task; a clean leaf harvest of the best-score resource
all six games copy.

## What went well

- Folded the two always-together resources (`HighScore` + `NewBest`) into one
  with a `record() -> bool`. The API fell straight out of the shared idiom
  (`new_best = score > high; high = high.max(score)`), and a strict `>` gave the
  "a tie is not a new best" rule 09 had spelled out in a comment for free.
- Designed for the persist composition the spike named: `#[serde(skip)]` on the
  per-run `new_best` flag means only the best value is stored, so
  `PersistPlugin::<HighScore<usize>>` just works. Tested the exact JSON
  (`{"best":42}`) rather than trusting the derive.
- Picked 09 as the second example because it *stressed* the generic (f64, not
  another usize) AND had a quirk worth fixing: its game-over chain read the stale
  best before recording. Moving to `record`-then-`is_new_best()` is both
  behaviour-equivalent and simpler than the old read-the-old-value trick.
- Re-checked the name-shadowing before assuming breakage: the un-migrated games
  keep a local `struct HighScore` that now shadows the prelude's generic one, and
  local items shadow glob imports without ambiguity, so they compiled untouched
  (confirmed by `clippy --all-targets`). This is the benign twin of the earlier
  `Pointer`/`bevy::prelude` collision -- worth pausing to tell them apart.

## What went wrong

- Nothing of substance. The only wrinkle was the persist format change for 06
  (`7` -> `{"best":7}`), which resets a stored score once; harmless for the
  placeholder-save example, flagged as informational.

## What to improve next time

- Keep choosing the second refactor example to exercise a *different* degree of
  freedom than the first (here f64 vs usize, and a different chain order) -- it
  proves more of the design than a same-shaped port.

## Action items

- [x] `scoring::HighScore<T>` shipped; 06 and 09 refactored.
- [ ] Migration of the remaining games (07/08/10/11) onto both `HighScore` and
  `SoundBank` folded into tatr 20260704-223846.
- [ ] Remaining dev-harness Wave 2: `ui/menu` builders (175424), input
  `AnyStartPress` + leaf helpers (175425).
