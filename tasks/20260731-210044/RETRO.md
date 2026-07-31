# Retro: Re-size the cargo test peak-RAM fix for 15 examples / 60 doctests

- TASK: 20260731-210044
- BRANCH: fix/test-peak-ram
- REVIEW ROUNDS: 2

## What went well

- **The review caught a wrong fix, not a thin record.** Round 1 read as two
  documentation gaps -- three of four DoD configurations unmeasured. Running
  them showed the cap itself was too high: 16.4 GB for `--doc` alone and
  18.4 GB for `--features debug`, both over target. A review that had accepted
  "the code verifies clean, the record is candid" would have shipped a fix that
  did not fix the problem.
- **Deriving the cap instead of hardcoding it survived the change.** Moving the
  divisor 4 -> 6 was a one-character edit that correctly re-derived on both
  machine profiles. A hardcoded `jobs = 7` would have needed a second decision
  about CI.
- **Building the measurement tool first paid off.** `sample-peak-rss.sh` made
  eight runs cheap and comparable. The 2026-07-03 fix was measured ad hoc,
  which is exactly why nobody noticed the workload outgrowing it.
- **Cross-checking against another session was worth the coordination.** The
  nova-protocol session, debugging the same failure on the same box, reached
  the same ranking (concurrency dominates, profile knobs secondary) from the
  opposite debuginfo-ownership regime. Its independent cap formula landed one
  job away from the shipped value. That is stronger evidence than either
  session had alone, and it cost two messages.

## What went wrong

- **The cap was sized against the default feature set.** `--features debug`
  links bevy-inspector-egui and egui into every binary and is the heaviest
  configuration this repo has. It was the one over the line and the one never
  measured. The DoD named it; the implementation skipped it.
- **A headline number was reported from a single unrepeated run.** The 13.1 GB
  figure could not be reproduced and was withdrawn. It was internally
  falsifiable at the time it was written: `--doc` is a strict subset of
  `cargo test`, so any `--doc` measurement above 13.1 GB would have refuted it.
  Nobody ran the subset.
- **A dead pointer was published to another session.** `sample-peak-rss.sh` was
  cited by its master path while it existed only on this unlanded branch. The
  other session went looking, did not find it, and wrote its own. Cheap here,
  but the same reflex put a stale doc path in the 2026-07-03 record too.
- **The plan gate was self-set.** `PLAN STATUS: APPROVED` was set by the
  implementing agent on its own plan, and round 2 was self-reviewed because no
  subagent was available. Both are recorded as pending user checks rather than
  quietly absorbed.

## What to improve next time

- Measure the heaviest configuration, not the default one. For this repo that
  means `--features debug` is the number that decides a resource limit; the
  default-feature number is informational.
- When a measurement bounds a whole, measure one part too. A subset that
  exceeds the claimed total is a free falsification test, and it is the cheapest
  possible check on a resource-limit claim.
- Before citing a path to anyone outside this worktree, check it against the
  landed tree, not the branch.

## Action items

- Ledger: `size-limits-against-the-heaviest-config` and
  `falsify-a-bound-with-a-subset` appended.
- Follow-up task `20260731-210413` (edition 2024 merged doctests) carries the
  now-real baseline it depends on.
- Open for the user: the self-set `PLAN STATUS: APPROVED` and the self-reviewed
  round 2.
