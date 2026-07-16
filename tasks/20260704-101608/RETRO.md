# Retro: Make GitHub Pages deploy resilient to transient failures

- TASK: 20260704-101608
- BRANCH: fix/pages-deploy-flaky (merged to master, worktree removed)
- REVIEW ROUNDS: 2

See `tasks/20260704-101608/TASK.md` for the change and
`tasks/20260704-101608/NOTES.md` for the root-cause writeup; this retro
is about how the cycle went.

## What went well

- Diagnosis before code. The starting request was "why does deploy sometimes
  fail", which is an investigation, not a patch. Reading the actual failing-run
  logs with `gh run view <id> --log-failed` (not guessing) pinned the exact
  line - `Deployment failed, try again later.` after `syncing_files` - and
  confirmed it was the same on both failing runs and independent of our
  artifact. That turned a vague "flaky" into a one-cause fix.
- Separating real failures from noise. The run list also had `cancelled` runs;
  recognizing those as `concurrency: cancel-in-progress` doing its job (not
  failures) kept the fix scoped to the actual problem instead of chasing a
  non-bug.
- Validating CI YAML without a red push. `actionlint` via `nix run
  nixpkgs#actionlint` gave a real linter for the workflow change locally, so
  the retry/`if:` logic was checked before merge rather than by pushing and
  watching Actions.

## What went wrong

- R1.1 (timeout budget): the first pass set `timeout-minutes: 15` with a
  comment claiming it left "room for the retry", but two `deploy-pages`
  attempts can each burn the action's internal 600s timeout, so 15 < 2 x 10
  min. Root cause: the timeout number was picked for a single attempt, then the
  retry was added, and the guard was not re-derived from the new worst case.
  Caught in review and bumped to 25. Lesson: when you add a retry, recompute
  every time/timeout budget from "attempts x per-attempt cost", do not reuse
  the single-attempt number.

## What to improve next time

- The environment tooling has no `python3 yaml` or `actionlint` on PATH, but
  `nix run nixpkgs#actionlint` works offline-ish and fast. Reach for that
  first for any `.github/workflows/*` change instead of a hand YAML parse.

## Action items

- [ ] Follow-up (not filed as a task yet): the `build` job runs a full `nix
  develop` with no binary cache and takes 15-25 min per deploy. Out of scope
  here, but a Nix binary cache (e.g. Magic Nix Cache / cachix) would cut that
  substantially. File as its own task if deploy latency becomes a pain.
- [x] Proposed AGENTS.md note: validate workflow changes with `nix run
  nixpkgs#actionlint` (see below).
