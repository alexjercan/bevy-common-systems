# Retro: check-stale-refs.sh: a fail-loud staleness gate for file moves and deletions

- TASK: 20260801-100108
- BRANCH: chore/check-stale-refs
- REVIEW ROUNDS: 2

## What went well

- Copying `check-ascii.sh` wholesale -- the exit-code `case` and its comment --
  gave the script its one genuinely load-bearing property for free. Every
  reviewer probe of the `>=2` path passed because that shape was inherited,
  not invented.
- The `manual:` DoD proof turned out to be fully machine-checkable (a
  throwaway `git worktree` at `809057c^` with the script copied in), so it was
  rewritten as a `cmd:` during work instead of being deferred to the user. The
  historical case then became the strongest single piece of evidence: 18 hits
  in 9 files that the original one-liner proof could not see.
- Round 1 was genuinely adversarial because the reviewer prompt named the
  attack surface (empty args, getopts contract, `set -u` array guard, running
  from a subdirectory, untracked files) rather than asking for "a review".
  Both MAJORs came from that list.

## What went wrong

- The script shipped two silent-clean paths -- exactly the defect class the
  task exists to close. `-x ''` builds the bare pathspec `:(exclude)`, which
  excludes the whole tree and prints the all-clear; and outside a repo, `cd
  "$(git rev-parse --show-toplevel)"` degrades to `cd ""` whose `set -e` exit
  of 1 is indistinguishable from "hits found" -- fatal under the DoD's own
  `! script <needle>` idiom.
  The failed decision: "probe both ways" was read as a statement about the
  MATCHER, so the probes were all needle-shaped (a needle that hits, one that
  does not). That seemed sound because the ledger lesson is written entirely
  in matcher terms -- every one of its three occurrences was a grep whose
  pattern covered too little. Nothing in it points at argument handling, and
  both bugs entered through an argument, not a pattern.
- The Close-out asserted "38 hits" for the positive control when the command
  printed 42, in the retro-able irony position: a number written from memory
  of a scrolled-past run, inside the task promoting "probe, do not assert".
  `record-numbers-from-a-rerun` (x2 at the time) names this exactly.
- The usage examples hardcoded `docs/wasm-web-builds.md`, a path the
  IMMEDIATELY PRECEDING task deleted. The Close-out had already reasoned about
  the self-scan hazard for the negative control and still missed the positive
  instance in the same header, so the staleness gate landed carrying a stale
  reference.

## What to improve next time

- Breadth: the diff is small and single-purpose (one script, one AGENTS.md
  paragraph, one ledger line) -- no split was available or wanted.
- Churn: the plan-time question that would have prevented both MAJORs is not
  in `plan`'s from-scratch challenge but one step earlier -- when the
  deliverable IS a checker, its argument surface is part of the matcher.
  Enumerate the arguments and probe each one empty, missing, mis-ordered and
  out-of-context, before probing any needle. The DoD listed two probes
  (positive, negative); it should have listed the argument matrix.
- A file that scans the tree scans ITSELF. Every literal path in such a
  script's own prose is a permanent hit for that path. Use fictional example
  paths, and check the file against its own examples before committing.
- Context: no pressure observed -- no compaction warning, no checkpoint, no
  handoff. Review round 1 was delegated by design, not by pressure.

## Action items

- Ledger: bump `record-numbers-from-a-rerun` to x3 (-> Pending promotions).
- Ledger: new `probe-the-argument-surface-too` (x1), the argument-side
  complement to the now-promoted `probe-a-new-checker-both-ways`.
- Ledger: new `a-tree-scanner-scans-itself` (x1).
- No follow-up tasks: N2.1 was fixed in round 2 rather than deferred, and
  nothing out of scope was discovered.
