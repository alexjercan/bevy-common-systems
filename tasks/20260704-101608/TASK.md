# Make GitHub Pages deploy resilient to transient 'try again later' failures

- STATUS: IN_PROGRESS
- PRIORITY: 90
- TAGS: bug,ci,web

## Problem

The "Deploy web showcase to GitHub Pages" workflow (`.github/workflows/pages.yml`)
sometimes fails even though nothing about the site changed. Investigation of the
failing runs (28678828555, 28676721121) shows the same pattern every time:

- The `build` job succeeds and uploads a valid `github-pages` artifact.
- The `deploy` job's `actions/deploy-pages@v4` step reaches
  `Current status: syncing_files` and then errors with
  `##[error]Deployment failed, try again later.`

This is a transient server-side GitHub Pages API error, not a problem with our
build output: re-running the same job succeeds. (The separate `cancelled` runs
are just `concurrency: cancel-in-progress` superseding older runs on rapid
pushes -- expected, not a failure.)

## Goal / Done

The deploy job survives a single transient "try again later" Pages API error
without a red workflow, by automatically retrying the deploy once (or a small
bounded number of times) before giving up. Observable: a transient failure on
the first `deploy-pages` attempt no longer fails the workflow; a genuinely
broken deploy still fails after the retries are exhausted.

## Steps

- [x] Add resilience to the `deploy` job in `.github/workflows/pages.yml`:
      run `actions/deploy-pages@v4` with `continue-on-error: true`, capture its
      outcome via a step `id`, and add a second `actions/deploy-pages@v4` step
      guarded by `if: steps.<first>.outcome == 'failure'` so a transient failure
      is retried once automatically. The final job status must reflect the retry
      (the workflow is green if the retry succeeds, red only if the retry also
      fails).
- [x] Add a `timeout-minutes` guard to the deploy job so a hung Pages sync
      cannot run indefinitely (the action's internal timeout is 600s; pick a
      job timeout comfortably above that, e.g. 15).
- [x] Add a short comment in the workflow explaining why the retry exists
      (transient "Deployment failed, try again later" Pages API errors), so a
      future reader does not remove it as redundant.
- [x] Validate the workflow YAML parses (actionlint, clean, exit 0) and re-read
      the file to confirm the retry logic and `if:` conditions are correct.
- [x] Update `docs/` with a short note on the transient Pages failure and the
      retry mitigation (docs/2026-07-04-pages-deploy-retry.md).

## Notes / Out of scope

- The long `build` job time (15-25 min, uninstrumented nix build with no
  binary cache) is a separate performance concern, not the cause of the deploy
  failures. Do not fold a nix-cache change into this task; if it is worth
  doing, file it as its own follow-up task.
- No push/deploy from this session; changes are merged locally on master by the
  flow and left for the user to push.
