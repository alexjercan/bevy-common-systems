# Review: Add non-ASCII guard (CI) enforcing plain-ASCII rule

- TASK: 20260703-103940
- BRANCH: chore/ci-workflow

## Round 1

- VERDICT: REQUEST_CHANGES

Verified independently:

- actionlint 1.7.12 clean (schema + shellcheck on `run:` blocks);
  shellcheck clean on the standalone script; `cargo fmt --check` and the
  ascii guard pass. The six workflow commands match the AGENTS.md suite
  exactly, which is the right scope for the maintainer's "full CI" choice.
- The workflow structure is sound: correct Bevy Linux deps (libasound2-dev,
  libudev-dev), nightly + rustfmt/clippy, rust-cache, concurrency
  cancellation. The documented "confirmed only on first push" limit is the
  honest and correct call for a headless environment.
- The ascii guard's happy path and match path both behave correctly
  (passes clean, fails with a file:line listing on an injected en-dash).

One robustness finding on the guard, and one optional note:

- [x] R1.1 (MINOR) scripts/check-ascii.sh:22 - the `if matches=$(grep ...)`
  construct treats every non-zero grep exit as "no non-ASCII found", but
  grep exits 2 on error (e.g. a scanned directory renamed or removed). I
  reproduced this: pointing the roots at a missing dir prints
  `grep: ...: No such file or directory` to stderr and the script still
  reports success (exit 0). A guard that silently stops enforcing under a
  plausible future refactor is worse than no guard, because CI stays green.
  Suggested change: branch on grep's exact exit code - 0 => matches found
  (fail), 1 => clean (pass), >=2 => grep error (fail loudly). For example:
      set +e
      matches=$(grep -rnP '[^\x00-\x7F]' "${roots[@]}")
      status=$?
      set -e
      case $status in
        0) echo "..."; echo "$matches" >&2; exit 1 ;;
        1) echo "check-ascii: no non-ASCII ..." ;;
        *) echo "check-ascii: grep failed (exit $status)" >&2; exit "$status" ;;
      esac
  - Response: fixed in ebeddfe. Rewrote with the exact case-on-exit-code
    structure suggested; verified all three branches (0 fails with listing,
    1 passes, 2 on a missing root now aborts with exit 2 instead of
    reporting success). shellcheck still clean.
- [x] R1.2 (NIT) .github/workflows/ci.yml:3-5 - `on: [push, pull_request]`
  with no branch filter double-runs for branches pushed inside this repo
  that also have an open PR (one `push` run, one `pull_request` run) - the
  concurrency groups differ (`github.ref` resolves differently per event),
  so neither cancels the other. Harmless, just wasted minutes. Optional:
  scope `push` to `branches: [master]` so intra-repo PRs run once via the
  `pull_request` event and master still gets post-merge coverage. Leave as
  is if you prefer every push validated.
  - Response: keeping `on: [push, pull_request]` as-is (pushing back). This
    repo's workflow merges feature branches to master *locally*, often
    without opening a PR (see the flow this session ran). Scoping push to
    `branches: [master]` would leave those feature branches with no CI at
    all - no PR to trigger `pull_request`, and push filtered out. Validating
    every pushed branch is the intended behavior here; the double-run only
    occurs in the rare push+open-PR overlap, which is an acceptable cost for
    a solo-maintained repo. Documented rather than changed.

R1.1 is a MINOR, so REQUEST_CHANGES: the guard should be trustworthy under
refactors before it ships. R1.2 is discretionary.

## Round 2

- VERDICT: APPROVE

- R1.1 verified fixed in ebeddfe: independently ran all three branches -
  clean tree exits 0, injected en-dash exits 1 with the file:line listing,
  and a missing root now exits 2 with a "grep failed" message instead of
  falsely reporting success. shellcheck clean; actionlint still clean.
- R1.2: accepted the pushback. For a repo that merges feature branches to
  master locally (as this session did), validating every pushed branch is
  the right default and scoping `push` to master would leave feature
  branches uncovered. The rare push+PR double-run is an acceptable cost.
- No new findings. The one irreducible unknown (the runner installing apt
  deps + nightly and Bevy linking headlessly) remains confirmable only on
  first push, which is the maintainer's call and is documented as such.
