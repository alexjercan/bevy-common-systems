# Review - 20260801-100108

Branch `chore/check-stale-refs` against `master`.

## Round 1 - REQUEST_CHANGES

Reviewer: out-of-context general-purpose subagent (`a075fbec63b57cef9`), given
only the task ID, branch/worktree, dimensions and record format. Primary re-ran
all DoD proofs and independently re-derived findings 1, 2 and 3.

All eight DoD `cmd:` proofs re-run green by the reviewer, including the
historical probe at `809057c^` (exit 1; 18 hits in 9 `web/games/*/index.html`
plus 7 elsewhere -- matches the Close-out exactly). `bash -n`, `shellcheck`
and the direct non-ASCII grep are clean.

### Findings

| # | Sev | Where | Finding | Change |
| - | - | - | - | - |
| 1 | MAJOR | `scripts/check-stale-refs.sh:45` | `-x ''` builds the pathspec `:(exclude)` with no path, excluding the whole tree; `git grep` exits 1 and the script prints the all-clear. `-x "$SKIP"` with `SKIP` unset silently disables the checker -- the exact failure class the task exists to prevent. | Reject an empty `OPTARG` with exit 2; add it as a third probe in the header and the Close-out. |
| 2 | MAJOR | `TASK.md:108` | Close-out claims the positive control yields "38 hits". Actual is 42 at the commit that shipped the sentence. An unverified number inside the promotion of "probe, don't assert". | Re-run and record the printed number with its unit and a per-file breakdown. |
| 3 | MINOR | `scripts/check-stale-refs.sh:57,85` | Outside a repo, `cd "$(git rev-parse --show-toplevel)"` degrades to `cd ""` and `set -e` exits **1** -- the same code as "stale references found", so the DoD's own `! script ...` idiom turns the crash into a pass. The documented `>=2` branch was never shown reachable. | `root=$(git rev-parse --show-toplevel) \|\| exit 2; cd "$root"`, and probe a real non-0/1 exit. |
| 4 | MINOR | `scripts/check-stale-refs.sh:43` | `getopts` stops at the first non-option, so `script AGENTS.md -x web` silently treats `-x` and `web` as needles and excludes nothing. | Reject a remaining `-`-prefixed argument after `shift`, honouring an explicit `--`. |
| 5 | MINOR | `scripts/check-stale-refs.sh:6-7` | The header hardcodes `docs/wasm-web-builds.md`, so checking that real deleted path reports the checker itself -- the staleness gate introduces a stale `docs/` reference. | Use a fictional example path. |
| 6 | MINOR | `scripts/check-stale-refs.sh:70`, `AGENTS.md:196` | Untracked files are invisible to `git grep`; "tracked tree" is stated as design, not as a caveat. A move plus a new un-`git add`ed doc checks green. | One clause in AGENTS.md: run it after `git add`. |
| 7 | NIT | `TASK.md:112` | `check-ascii.sh` scans `src bevy_common_systems_macros/src examples` only -- it does not cover `scripts/`, so it is not evidence for the new file. The claim is true (direct grep confirms) but the cited proof is not. | Cite the direct `grep -nP '[^\x00-\x7F]'` run. |
| 8 | NIT | `TASK.md:5` | `TAGS` whitespace reflowed by `tatr edit`. | Waived: tool-normalised metadata, not authored churn. |

Verdict: **REQUEST_CHANGES** (findings 1, 2 open MAJOR).

Pending `manual:` items: none. The task's only `manual:` proof was found to be
fully machine-checkable during work and was rewritten as a `cmd:`.
