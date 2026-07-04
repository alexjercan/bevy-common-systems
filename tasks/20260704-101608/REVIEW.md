# Review: Make GitHub Pages deploy resilient to transient failures

- TASK: 20260704-101608
- BRANCH: fix/pages-deploy-flaky

## Round 1

- VERDICT: REQUEST_CHANGES

Scope: `.github/workflows/pages.yml` (+ docs note, TASK.md). Root-cause
analysis matches the failing-run logs (28678828555, 28676721121): the `build`
job and artifact are fine; `actions/deploy-pages@v4` returns a transient
"Deployment failed, try again later." during `syncing_files`. The retry design
is the right mitigation and the control flow is correct: `continue-on-error`
on attempt 1 + `if: steps.deployment.outcome == 'failure'` gate on the retry
means the job is green when the retry succeeds and red only when it also
fails. `outcome` (not `conclusion`) is the correct property to gate on.
actionlint passes clean. The environment-url fallback is correct (empty
`page_url` from a skipped retry falls through to attempt 1's URL).

One finding to address before merge:

- [ ] R1.1 (MINOR) .github/workflows/pages.yml:56 - `timeout-minutes: 15` is
  smaller than two full action timeouts. `actions/deploy-pages` has an
  internal 600s (10 min) timeout per attempt; if attempt 1 ever fails by
  hitting that internal timeout rather than the observed fast "try again
  later" (~13s), the retry only has ~5 min left and could be cut off, making
  the job red for a timeout reason instead of exhausting the retry. The
  observed failures are fast-fail so this is an edge case, not a regression
  (worst case is the same red job today), but the guard should comfortably fit
  both attempts. Raise to `timeout-minutes: 25` (2 x 10 min action timeout +
  overhead), and adjust the accompanying comment so the number and the "leaving
  room for the retry" claim actually agree.
  - Response: Done. Raised `timeout-minutes` to 25 and rewrote the comment to
    state the 2 x 600s reasoning explicitly. Updated the docs note to match.
    actionlint still clean.

Considered and explicitly NOT blocking:
- No backoff/sleep before the retry. `deploy-pages` creates a fresh
  deployment on the retry, which is what clears the transient error; an
  immediate second attempt is fine. (NIT at most.)
- Single retry rather than a backoff loop. Reasonable, documented tradeoff;
  trivially bumped to two attempts later if needed.
