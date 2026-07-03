# Retro: Add non-ASCII guard + first CI workflow

- TASK: 20260703-103940
- BRANCH: chore/ci-workflow (merged to master, deleted)
- REVIEW ROUNDS: 2 (R1 REQUEST_CHANGES on a guard-reliability bug, R2 APPROVE)

See TASK.md close-out for what shipped; this is process only.

## What went well

- Stopping to ask before building was the right call. The task hinted "CI
  preferred", but standing up the repo's first CI workflow is a
  maintainer-level decision that materially changed scope (from one grep to
  a full pipeline). Asking turned an assumption into an explicit choice.
- Verified every separable piece in a headless box instead of hand-waving:
  ran actionlint and shellcheck via `nix run nixpkgs#...` (the repo already
  ships a flake), and ran all six workflow commands locally. That is real
  validation of the schema, the shell, and the commands - everything except
  the runner OS itself, which was documented as the one honest unknown.
- The review caught a genuine reliability bug in my own guard (grep exit 2
  masking as success), reproduced it before writing the finding, and the
  fix made the tool trustworthy under future refactors. The guard existing
  is worth little if it can silently stop guarding.

## What went wrong

- I shipped the guard with the `if grep` idiom that conflates "no matches"
  with "grep errored". Root cause: wrote the happy path (match vs no-match)
  and did not think about the third exit code until reviewing adversarially.
  A guard's failure modes matter more than its happy path - I reviewed it
  as "does it catch a bad char" not "can it ever wrongly pass".

## What to improve next time

- For any check/guard/gate, enumerate its exit conditions up front,
  including tool-error paths, and assert the fail-closed behavior. "What
  makes this pass when it should not?" is the question that matters for a
  gate, and it is exactly what the adversarial review lens supplied.

## Action items

- [x] Guard now fails closed on grep errors (R1.1).
- [x] CI runs the full documented suite + ascii guard on push/PR; the ascii
  script is shared between CI and local use and documented in AGENTS.md.
- [ ] Open, maintainer's call: push to GitHub to confirm the runner side
  (apt deps, nightly install, headless Bevy link). This is the one thing
  no local check could prove; the workflow is otherwise validated.
- [ ] Pattern watch: this is the second retro touching the plain-ASCII rule
  (see 20260703-101712). It now has a mechanical guard, so the loop should
  be closed - no further action unless it recurs despite CI.
