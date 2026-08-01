# Review: check-stale-refs.sh: a fail-loud staleness gate for file moves and deletions

- TASK: 20260801-100108
- BRANCH: chore/check-stale-refs
- WORKTREE: /home/alex/.cache/sprouts/bevy-common-systems/chore/check-stale-refs
- BASE: master

## Round 1

- REVIEWER: out-of-context
- VERDICT: REQUEST_CHANGES

Out-of-context general-purpose subagent (`a075fbec63b57cef9`), given only the
task ID, branch/worktree, review dimensions and record format, and told to be
adversarial about the new checker itself. The primary independently re-derived
findings R1.1, R1.2 and R1.3 before accepting them.

All eight DoD `cmd:` proofs re-run green by the reviewer, including the
historical probe at `809057c^` (exit 1; 18 hits in 9 `web/games/*/index.html`
plus 7 elsewhere -- matching the Close-out exactly). `bash -n`, `shellcheck`
and the direct non-ASCII grep clean.

### Findings

- [x] R1.1 (MAJOR) `scripts/check-stale-refs.sh:45` -- `-x ''` builds the bare
  pathspec `:(exclude)`, which excludes the entire tree; `git grep` then exits
  1 and the script prints the all-clear. `-x "$SKIP"` with `SKIP` unset
  silently disables the checker -- the exact failure class this task exists to
  prevent. Evidence: `./scripts/check-stale-refs.sh -x '' AGENTS.md` ->
  "no stale references", exit 0, versus 42 hits and exit 1 without `-x`.
  **Change:** reject an empty `OPTARG` with exit 2; add it as a probe in the
  header and the Close-out.
  **Response:** done. `-z "$OPTARG"` -> usage, exit 2, with a `NOTE:` naming
  the whole-tree hazard. Re-run: `-x '' AGENTS.md` -> exit 2.

- [x] R1.2 (MAJOR) `tasks/20260801-100108/TASK.md:108` -- Close-out claimed the
  positive control yields "38 hits". Actual is 42 at the commit that shipped
  the sentence (`git grep -n -F -e 'AGENTS.md' d01e331 -- ':(exclude)tasks/' |
  wc -l` -> 42; no revision on this branch yields 38). An unverified number
  inside the promotion of "probe, do not assert".
  **Change:** re-run and record the printed number with its unit.
  **Response:** done. Close-out now reads "42 hit lines across 13 files, none
  under `tasks/`", re-derived at the final tree state after all fixes.

- [x] R1.3 (MINOR) `scripts/check-stale-refs.sh:57,85` -- outside a repo,
  `cd "$(git rev-parse --show-toplevel)"` degrades to `cd ""` and `set -e`
  exits **1**, the same code as "stale references found"; the DoD's own
  `! script <needle>` idiom therefore converts the crash into a pass. The
  documented `>=2` branch was never shown reachable.
  **Change:** `root=$(git rev-parse --show-toplevel) || exit 2; cd "$root"`,
  and probe a real non-0/1 exit.
  **Response:** done. Outside a repo -> exit 2. The `>=2` branch was probed
  with a `git` shim exiting 128 on `grep`: prints "cannot verify", exits 128.
  Header records that the branch is NOT reachable through `-x`, since every
  `-x` value nests inside `:(exclude)` and a bad magic there parses as an inert
  literal path (verified: `-x ':(bogus)x'` -> normal result, not an error).

- [x] R1.4 (MINOR) `scripts/check-stale-refs.sh:43` -- `getopts` stops at the
  first non-option, so `script AGENTS.md -x web` silently treated `-x` and
  `web` as needles and excluded nothing.
  **Change:** reject a dash-prefixed operand after `shift`, honouring `--`.
  **Response:** done. `AGENTS.md -x web` -> exit 2; `-- -x` still searches the
  literal needle `-x` -> exit 1. Note `getopts` consumes an explicit `--`
  itself, so the script inspects `${*:$((OPTIND-1)):1}` before shifting.

- [x] R1.5 (MINOR) `scripts/check-stale-refs.sh:6-7` -- the usage examples
  hardcoded the real deleted `docs/wasm-web-builds.md`, making the file a
  permanent hit for that very check; the staleness gate shipped a stale
  `docs/` reference.
  **Change:** use a fictional example path.
  **Response:** done, `old/dir/renamed.md`, with a `NOTE:` on why.
  `./scripts/check-stale-refs.sh docs/wasm-web-builds.md` -> exit 0.

- [x] R1.6 (MINOR) `scripts/check-stale-refs.sh:70`, `AGENTS.md:196` --
  untracked files are invisible to `git grep`; "tracked tree" was stated as
  design, not as a caveat, so a move plus a new un-`git add`ed doc checks green.
  **Change:** one clause in the AGENTS.md line.
  **Response:** done, in both AGENTS.md and the script header.

- [x] R1.7 (NIT) `tasks/20260801-100108/TASK.md:112` -- `check-ascii.sh` scans
  `src bevy_common_systems_macros/src examples` only, so it is not evidence for
  a file under `scripts/`. The claim was true, the cited proof was not.
  **Change:** cite the direct `grep -nP '[^\x00-\x7F]'` run.
  **Response:** done, with the gap named explicitly.

- [x] R1.8 (NIT) `tasks/20260801-100108/TASK.md:5` -- `TAGS` whitespace
  reflowed.
  **Change:** revert the churn.
  **Response:** waived -- written by `tatr flow`, not authored churn.

Pending `manual:` items: none. The task's only `manual:` proof proved fully
machine-checkable during work and was rewritten as a `cmd:`.
