# check-stale-refs.sh: a fail-loud staleness gate for file moves and deletions

- STATUS: OPEN
- PRIORITY: 45
- TAGS: chore,tooling,lessons
- KIND: TASK
- FLOW STEP: BACKLOG
- PLAN STATUS: DRAFT

## Context

Promotion of ledger lesson `probe-a-new-checker-both-ways` (x3:
20260731-172208, 20260731-172232, 20260801-094300), disposition PROMOTE ->
tool, decided 2026-08-01.

The recurring failure is a matcher trusted without a positive control. Third
occurrence: task 20260801-094300 (retire `docs/`) carried a DoD proof

```
! grep -rn "docs/" --include='*.md' --include='*.sh' ... . | grep -v '^tasks/' ...
```

which exited clean while 18 references to the deleted file survived in nine
`web/games/*/index.html` files -- `.html` was not in the allowlist. Being red
on the base branch proved nothing: it was red for the hits it COULD see. The
same task also shipped a wrong `grep -v '^./tasks/'` prefix filter (`grep -rn
... .` emits paths without `./`).

Both halves are what a script gets right once: `git grep` for repo-root
relative paths, `--exclude-dir`/pathspec exclusions (fail loud on a new file
type) instead of an `--include` allowlist (fails silent).

## Steps

1. Write `scripts/check-stale-refs.sh <needle>...`: for each needle, run
   `git grep -n -- "<needle>"` over the tracked tree, excluding `tasks/` (task
   records are historical by convention) and any path the caller passes via a
   repeated `-x <pathspec>`. Exit 1 and print `file:line:text` on any hit, 0
   and a one-line "no stale references to X" otherwise. Branch on git grep's
   exact exit code the way `scripts/check-ascii.sh` does: 0 = hits
   (violation), 1 = clean, >=2 = the tool itself failed, which must NOT be
   reported as clean.
2. Self-test both ways, in the script's own header comment and in the task
   record: a needle that must hit (e.g. `AGENTS.md`, which is referenced all
   over) and one that must not (a random string). A checker that has only ever
   been run clean is the exact thing this lesson is about.
3. Document it in `AGENTS.md` "Build, Verify, Run" beside `check-ascii.sh` and
   `check-comment-tags.sh`, one line: run it after any file move, rename or
   deletion.
4. Update the `probe-a-new-checker-both-ways` ledger entry to
   `PROMOTED 2026-08-01 -> scripts/check-stale-refs.sh (task <this id>)` and
   clear it from "Pending promotions" (that section returns to `None`).

## Definition of Done

- [ ] Script exists and is executable
      (cmd: `test -x scripts/check-stale-refs.sh`)
- [ ] Positive control: a needle that exists in the tree is reported
      (cmd: `! ./scripts/check-stale-refs.sh AGENTS.md`)
- [ ] Negative control: an absent needle exits clean
      (cmd: `./scripts/check-stale-refs.sh zzz-no-such-reference-zzz`)
- [ ] The 20260801-094300 case would have been caught: with the script in
      place, `git stash`-free check against that commit's parent state
      (cmd: `git show 809057c^:web/games/06_fruitninja/index.html | grep -c 'docs/wasm-web-builds.md'` returns 1, confirming the reference the old proof missed)
      (manual: in a `git worktree` at 809057c with the script copied in, run
      `./scripts/check-stale-refs.sh docs/wasm-web-builds.md` and confirm it
      exits 1 listing all 18 `web/games/*/index.html` hits)
- [ ] Documented in AGENTS.md
      (cmd: `grep -q check-stale-refs AGENTS.md`)
- [ ] Ledger entry marked PROMOTED, Pending promotions back to `None`
      (cmd: `grep -q 'probe-a-new-checker-both-ways.*PROMOTED' LESSONS.md`)
- [ ] Tracker clean (cmd: `tatr check --ledger LESSONS.md`)

## Notes

- Keep it dumb: needles are literal strings, not regexes, and there is no
  config file. The value is the two fail-loud choices (`git grep`,
  exclude-not-include) plus a checker that was probed both ways -- not
  generality.
- `scripts/check-ascii.sh` is the shape to copy, including its exit-code
  `case` and the comment explaining why `>=2` must fail loudly.
