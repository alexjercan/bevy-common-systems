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

- [ ] R1.1 (MINOR) scripts/check-ascii.sh:22 - the `if matches=$(grep ...)`
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
- [ ] R1.2 (NIT) .github/workflows/ci.yml:3-5 - `on: [push, pull_request]`
  with no branch filter double-runs for branches pushed inside this repo
  that also have an open PR (one `push` run, one `pull_request` run) - the
  concurrency groups differ (`github.ref` resolves differently per event),
  so neither cancels the other. Harmless, just wasted minutes. Optional:
  scope `push` to `branches: [master]` so intra-repo PRs run once via the
  `pull_request` event and master still gets post-merge coverage. Leave as
  is if you prefer every push validated.

R1.1 is a MINOR, so REQUEST_CHANGES: the guard should be trustworthy under
refactors before it ships. R1.2 is discretionary.
