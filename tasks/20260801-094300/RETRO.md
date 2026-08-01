# Retro: Retire docs/: fold reference docs into web/README, examples/README and task NOTES

- TASK: 20260801-094300
- BRANCH: chore/retire-docs-folder
- REVIEW ROUNDS: 2 (R1 REQUEST_CHANGES, R2 APPROVE)

## What went well

- Planning the move as a destination TABLE (source file -> destination -> why)
  before touching anything made the diff mechanical and made "did anything get
  lost?" a checkable question rather than a judgement call. The reviewer could
  diff old-vs-new paragraph by paragraph against that table.
- Deciding up front that this was a MOVE and not an edit pass. The one
  deliberate deletion (a paragraph in the trunk section duplicating the
  "Adding a game" list already above it in the same file) was named in the
  close-out, so the reviewer verified exactly one claim instead of auditing
  every dropped line. Trimming while moving would have made the diff
  unreviewable for content loss.
- Splitting `dev-harness.md` by AUDIENCE rather than by size: what an example
  author needs to run the harness went to `examples/README.md`, while "why the
  API is shaped this way / alternatives rejected" went to the NOTES of the task
  that built it (20260704-175421, which had TASK+REVIEW but no NOTES). That is
  the repo's own records convention applied retroactively, and it filled a real
  gap rather than just relocating bytes.
- The out-of-context reviewer paid for itself: it caught the MAJOR by
  questioning the PROOF rather than the prose, which is exactly what an
  implementer who just wrote the proof is worst positioned to do.

## What went wrong

- The DoD's stale-reference sweep used an `--include` allowlist
  (`.md .sh .js .yml .rs .ts .toml`) that omitted `.html`, so it exited clean
  while 18 references to the deleted `docs/wasm-web-builds.md` survived in nine
  `web/games/*/index.html` files. Root cause: the allowlist was written from
  the file types the ORIGINAL survey had found hits in, not from the file types
  the repo contains. The original survey used the same too-narrow list, so it
  never had a chance to see the `.html` hits -- the proof inherited the survey's
  blind spot and then certified it.
- Compounding this: the proof ALSO had a wrong `grep -v '^./tasks/'` prefix
  filter (`grep -rn ... .` emits paths without `./`), which I did catch and fix
  mid-implementation. Fixing one fail-open half of a proof and shipping the
  other is the tell -- once a proof has been shown to fail open, the whole
  expression deserves re-derivation, not a patch to the clause that happened to
  be visible.
- The failure mode was already in the ledger twice
  (`grep-whole-tree-before-rename`, `probe-a-new-checker-both-ways`) and in the
  close-out's own Reflection paragraph, written BEFORE the review found the
  instance it describes. Naming a hazard in prose is not the same as running
  the check it implies.

## What to improve next time

- A negative proof (`! grep ...`) is only as good as its positive control. The
  cheap guard: run the sweep once WITHOUT the filters and read what they
  suppress. Here that would have printed 18 `.html` lines in one second.
- Prefer an exclude list to an include list for whole-tree sweeps: `grep -rn
  --exclude-dir={node_modules,dist,.git,tasks}` fails LOUD (a new file type
  shows up as noise to triage) where `--include=` fails silent (a new file type
  is invisible). Same for the deny half: anchor the path filter to what the
  tool actually emits, or use `git grep`, whose paths are repo-root relative
  and stable.

## Diagnose

- **Breadth.** 17 files, ~470 lines, but inherently one change: a folder cannot
  be retired in pieces without leaving dangling references between commits, and
  every touched file outside the three destinations is a one-line pointer fix.
  No missed split.
- **Churn.** One review round of rework, all of it R1.1 and all of it the same
  root cause. The plan-time question that would have prevented it: the plan
  wrote the DoD proof as a command WITHOUT running it against a known-positive
  case first. `plan` says to run each `cmd:` proof on the base branch and
  confirm it is red for the intended missing change -- I ran it and it WAS red,
  but red for the 100+ references it could see, which masked the ones it could
  not. Redness is not evidence of coverage; a proof needs a case it must catch
  AND a case it must not.
- **Context.** No context pressure. No compaction warning, no handoff, one
  delegation (the round-1 reviewer, ~82k subagent tokens), which was the right
  call for a diff whose main risk is content loss across a 200-line move.

## Action items

- None requiring a follow-up task; the proof is fixed in this task's DoD and
  the lesson is bumped in the ledger.
- `probe-a-new-checker-both-ways` reaches x3 and moves to Pending promotions
  for `/lessons` to dispose of.
