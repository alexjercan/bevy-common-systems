# check-stale-refs.sh: a fail-loud staleness gate for file moves and deletions

- STATUS: IN_PROGRESS
- PRIORITY: 45
- TAGS: chore, tooling, lessons
- KIND: TASK
- FLOW STEP: COMPOUNDING
- PLAN STATUS: APPROVED

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

- [x] Script exists and is executable
      (cmd: `test -x scripts/check-stale-refs.sh`)
- [x] Positive control: a needle that exists in the tree is reported
      (cmd: `! ./scripts/check-stale-refs.sh AGENTS.md`)
- [x] Negative control: an absent needle exits clean
      (cmd: `./scripts/check-stale-refs.sh zzz-no-such-reference-zzz`)
- [x] The 20260801-094300 case would have been caught: with the script in
      place, `git stash`-free check against that commit's parent state
      (cmd: `git show 809057c^:web/games/06_fruitninja/index.html | grep -c 'docs/wasm-web-builds.md'` returns 2, confirming the references the old proof missed)
      (cmd: in a `git worktree` at `809057c^` with the script copied in,
      `./scripts/check-stale-refs.sh docs/wasm-web-builds.md` exits 1 listing
      18 `web/games/*/index.html` hits -- 9 files x 2 -- plus 7 in
      `AGENTS.md`, `README.md`, `web/README.md`, `assets/sounds/README.md` and
      `web/games/_shared/audio-unlock.js`)
- [x] Documented in AGENTS.md
      (cmd: `grep -q check-stale-refs AGENTS.md`)
- [x] Ledger entry marked PROMOTED, Pending promotions back to `None`
      (cmd: `grep -q 'probe-a-new-checker-both-ways.*PROMOTED' LESSONS.md`)
- [x] Tracker clean (cmd: `tatr check --ledger LESSONS.md`)

## Notes

- Keep it dumb: needles are literal strings, not regexes, and there is no
  config file. The value is the two fail-loud choices (`git grep`,
  exclude-not-include) plus a checker that was probed both ways -- not
  generality.
- `scripts/check-ascii.sh` is the shape to copy, including its exit-code
  `case` and the comment explaining why `>=2` must fail loudly.

## Close-out

**What / why.** `scripts/check-stale-refs.sh <needle>...` turns the
`probe-a-new-checker-both-ways` lesson into a tool. Two fail-loud choices:
`git grep` from the repo toplevel (repo-root-relative output, so a path
prefix filter actually matches), and exclusions (`:(exclude)tasks/` by
convention, repeatable `-x <pathspec>`) instead of an `--include` allowlist
that goes silent on a new file type. Needles are literal (`--fixed-strings`);
no config, no regex.

**Alternatives.** Ledger listed tool > proofs-template field > skill prose;
the user picked the tool. Within the tool: an `--include` allowlist was the
failure mode, not an option. A CI-wired invocation was rejected -- the script
takes arguments, so it belongs beside `check-comment-tags.sh` as an
on-demand check, documented as such in AGENTS.md.

**Difficulties.** The negative control could not be spelled literally in the
script header: `git grep` scans the script itself, so a documented nonsense
needle would have made the negative-control proof fail. Header describes it
instead. The same self-scan trap bit the positive direction and was only
caught in review -- the usage examples originally named the real deleted
`docs/wasm-web-builds.md`, which made the file a permanent hit for that very
check; they are fictional paths now. Also corrected one DoD proof: the
06_fruitninja file holds 2 stale lines, not 1 (18 total = 9 files x 2), and
the `manual:` probe turned out to be fully machine-checkable, so it was
rewritten as a `cmd:`.

**Review fixes (round 1).** Two silent-clean paths that the first cut shipped,
both the exact failure class this task exists to close: `-x ''` built the bare
pathspec `:(exclude)`, which excludes the WHOLE tree, so `-x "$UNSET_VAR"`
turned the checker off and printed the all-clear (now exit 2); and `cd
"$(git rev-parse --show-toplevel)"` outside a repo degraded to `cd ""`, whose
`set -e` exit of 1 is indistinguishable from "hits found" -- fatal under the
DoD's own `! script <needle>` idiom (now `root=$(...) || exit 2`). Also:
options are no longer silently reinterpreted as needles after the first
operand (`--` escapes a real dash-prefixed needle), and the untracked-files
blind spot is now a stated caveat rather than a design note. Round 2 added
one more: an `-x` value of `--` sat in the same argument slot the
end-of-options detector reads, silently re-enabling the mis-ordered-option
hole; `-x` now rejects it.

**Evidence.** All re-run at the final tree state. Positive control:
`AGENTS.md` -> exit 1, 42 hit lines across 13 files, none under `tasks/`.
Negative: nonsense needle -> exit 0. Tool failure: outside a repo -> exit 2;
a `git` shim exiting 128 on `grep` -> the `>=2` branch prints "cannot verify"
and exits 128. Rejections: no args, `-x ''`, `AGENTS.md -x web`, bare `--` ->
exit 2; `-- -x` searches the literal needle `-x` -> exit 1. `-x` verified by
excluding seven paths and watching the hit list collapse to one line.
Historical case: worktree at `809057c^`, script copied in,
`docs/wasm-web-builds.md` -> exit 1 with all 18 `web/games/*/index.html`
references the original one-liner proof missed. `shellcheck` clean, `bash -n`
clean, `grep -nP '[^\x00-\x7F]'` over the script and AGENTS.md clean
(`check-ascii.sh` does NOT cover `scripts/`, so it is not evidence here),
`tatr check --ledger` exit 0.

**Reflection.** The exit-code `case` copied from `check-ascii.sh` is the
load-bearing part: `git grep` exits 1 for clean and >=2 for its own failure,
so the naive `if git grep ...` reports a broken pathspec as clean. Probing a
checker "both ways" is really three -- match, no-match, and tool error -- and
review showed a fourth axis the lesson does not name: probe the checker's own
ARGUMENT handling, because every silent-clean bug found here entered through
an argument (`-x ''`, a mis-ordered `-x`, a missing repo) rather than through
the matcher.
