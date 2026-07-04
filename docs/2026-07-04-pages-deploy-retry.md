# GitHub Pages deploy: transient failures and the automatic retry

Date: 2026-07-04

## Symptom

The "Deploy web showcase to GitHub Pages" workflow
(`.github/workflows/pages.yml`) would go red intermittently with no change to
the site or the build. Two representative failing runs: 28678828555 and
28676721121.

## Investigation

`gh run view <id> --log-failed` showed the same shape in every failure:

- The `build` job succeeded and uploaded a valid `github-pages` artifact.
- The `deploy` job's `actions/deploy-pages@v4` step reached
  `Current status: syncing_files` and then errored with:

  ```
  ##[error]Deployment failed, try again later.
  ```

That is a transient server-side error from the GitHub Pages API sync, not a
problem with our artifact: manually re-running the same job succeeds. The
build output is identical between the failing and passing runs.

Two things that look like failures but are not:

- **`cancelled` runs.** The workflow sets `concurrency: { group: pages,
  cancel-in-progress: true }`, so a newer push cancels an older, still-running
  Pages run. That is intended and shows up as `cancelled`, not `failure`.
- **Long `build` times (15-25 min).** The build runs a full `nix develop`
  with no binary cache, so it is slow, but it is not the cause of the deploy
  failures. That is a separate performance concern (a possible follow-up:
  add a Nix binary cache / Magic Nix Cache), deliberately left out of this
  fix.

## Fix

Made the `deploy` job tolerate a single transient failure by retrying the
deploy step once:

- The first `actions/deploy-pages@v4` step runs with
  `continue-on-error: true`, so a transient error does not immediately fail
  the job.
- A second `actions/deploy-pages@v4` step runs only
  `if: steps.deployment.outcome == 'failure'`. If this retry succeeds the
  workflow is green; if it also fails the job goes red, so a genuinely broken
  deploy is still surfaced rather than hidden.
- The environment URL is reported from whichever attempt deployed:
  `steps.deployment-retry.outputs.page_url || steps.deployment.outputs.page_url`.
- Added `timeout-minutes: 25` to the job so a hung sync cannot run
  indefinitely. Each `deploy-pages` attempt has its own internal 600s (10 min)
  timeout and we run it up to twice, so the job cap is set above 2 x 10 min
  plus overhead rather than a single attempt's worth.

Validated with `actionlint` (clean).

## Why a retry instead of something fancier

The Pages API error is transient and clears on a re-attempt, so a single
bounded retry removes essentially all of these spurious red runs with almost
no added complexity or wall-clock cost (the retry only runs when the first
attempt failed). A full exponential-backoff loop would be more machinery than
the failure rate justifies; if one retry turns out not to be enough in
practice, bumping to two attempts is a one-line change.
