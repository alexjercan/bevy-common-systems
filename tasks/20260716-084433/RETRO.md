# Retro: docs records-next-to-task restructure

- TASK: 20260716-084433
- SCOPE: docs reorg only, no code changes.

See TASK.md and NOTES.md for what changed and why; this retro is about how the
cycle went.

## What went well

- Studying the sister repo's actual cleanup commit (nova-protocol 514fb577)
  first gave an exact target: the archive-stub `TASK.md` format, the
  `docs/` -> task-folder mapping, and the AGENTS.md convention wording were all
  lifted rather than reinvented.
- Deriving the rename map from `git mv` (git's own rename detection) meant the
  reference-rewrite pass had a guaranteed-correct old->new table; no hand-typed
  paths to get wrong.
- Doing the mechanical moves as small scripted passes (retros, then spikes,
  then devlogs) with per-pass counts caught the one no-id retro and the 3
  task-less devlogs cleanly instead of silently misfiling them.

## What went wrong / difficulties

- The top-level devlogs used `2026-07-03` dates while tatr ids use
  `20260703-HHMMSS`, and several devlogs carried no task-id header. Mapping
  them needed reading bodies and matching slugs to retros - the only
  judgement-heavy part.
- First instinct was that the top-level devlogs duplicated the retros; reading
  a matched pair showed they are distinct (design record vs cycle retro), which
  is exactly the NOTES.md vs RETRO.md split. Good that this was checked before
  deleting anything.

## What to do differently

- The reference rewrite had to be literal and longest-path-first to avoid
  partial-path collisions; worth remembering that regex path rewrites over a
  docs tree are a foot-gun. See LESSONS ledger.
