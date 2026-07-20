# Adopt flow v2: root LESSONS.md, clean tatr check, AGENTS.md flow section

- STATUS: CLOSED
- PRIORITY: 90
- TAGS: chore, process

## Story

As a repo in the flow ecosystem, I want the v2 /flow conventions in place -
root LESSONS.md ledger, clean tatr check, AGENTS.md pointing at /flow - so
development here compounds the same way as everywhere else. Part of the
six-repo adoption goal (umbrella: nix.dotfiles tasks/20260720-171807).

## Steps

- [x] Ledger at the root: move docs/LESSONS.md to LESSONS.md (git mv) - or
      create it from the lessons-skill format if the repo has none - then
      run the doc-surface sweep for every reference to the old path
      (AGENTS.md, README, scripts, CI guards, wiki pages) and update them.
      Bring the ledger to format: bare counts until promotion, a
      "## Pending promotions (3+ occurrences, user decides)" section;
      move unpromoted (x3)+ entries there; keep existing PROMOTED/absorbed
      annotations as they are.
- [x] Fix tatr check findings best-effort, assuming recorded work was done
      properly where the record supports it:
      - closed-unchecked: tick Steps boxes whose close-out notes or landed
        commits evidence the work shipped; genuinely unshipped steps stay
        unticked and go on the residue list;
      - closed-not-approved: normalize nonstandard-but-approving verdict
        lines (e.g. "Verdict: APPROVE", "**APPROVE**") to
        "- VERDICT: APPROVE"; a review that really ended unapproved goes on
        the residue list untouched;
      - bad-severity: map to the nearest of BLOCKER/MAJOR/MINOR/NIT
        (LOW -> MINOR, NOTE/INFO/OBSERVATION -> NIT, FIXED -> the severity
        it had, keeping any "fixed in-review" note in the text), recording
        the mapping in the close-out.
- [x] AGENTS.md: add or refresh a "Development flow" section stating: /flow
      drives development here (plan/work/review/compound via tatr tasks,
      sprout worktrees, out-of-context round-1 reviews, DoD proofs with
      test:/cmd:/manual: notation); LESSONS.md at the repo root is the
      lessons ledger, read before starting any task; `tatr check` (plus
      `--ledger LESSONS.md`) is the conformance gate. Keep the section
      short; do not restructure the rest of the file.
- [x] Verify: tatr check exit 0 (or residue listed in the close-out),
      tatr check --ledger LESSONS.md exit 0, and the repo's own check
      suite still green.

## Definition of Done

- LESSONS.md at the repo root, old docs/ path gone, no stale references
  (cmd: test -f LESSONS.md && test ! -f docs/LESSONS.md && ! grep -rn "docs/LESSONS" --include="*.md" --include="*.sh" .)
- tatr check clean or residue documented (cmd: /home/alex/personal/tatr/tatr check;
  manual: user reviews the residue list at the goal's Finish)
- ledger lints clean (cmd: /home/alex/personal/tatr/tatr check --ledger LESSONS.md)
- AGENTS.md names /flow and LESSONS.md (cmd: grep -n "flow\|LESSONS.md" AGENTS.md)

## Notes

- Use the tatr binary at /home/alex/personal/tatr/tatr (the installed one
  may predate the check subcommand).
- Preserve history honestly: normalizations keep meaning; ticks record
  verifiably shipped work only (linter-adoption cleanup, per the precedent
  in tatr's own 20260720-152503).

## Close-out (2026-07-20)

What changed:

- Ledger: `git mv docs/LESSONS.md LESSONS.md`; header rewritten to the
  lessons-skill format (bare counts, lifecycle annotations, promotion at
  x3). All 24 unpromoted (x3)+ entries moved verbatim into a new
  "## Pending promotions (3+ occurrences, user decides)" section (14
  process + 10 technical); the 4 sub-3 entries (reuse-the-kit x2,
  try-entity-commands-for-fire-and-forget x2,
  wasm-getrandom-and-build-profile x2, release-gates x1) stay under
  Technical lessons. No entry had a PROMOTED/absorbed/RETIRED annotation,
  so none needed preserving. Reference sweep: AGENTS.md (layout, "Where
  records go", Workflow), README.md "More" list, docs/README.md (link now
  `../LESSONS.md`); no hits in scripts/, .github/, web/, src/. pages.yml
  needs no change: `**/*.md` in paths-ignore already covers a root
  LESSONS.md.
- AGENTS.md: new short "## Development flow" section (before Workflow):
  /flow (plan/work/review/compound), sprout worktrees, out-of-context
  round-1 reviews, DoD proofs in test:/cmd:/manual: notation, root
  LESSONS.md read before any task, tatr check (+ --ledger) as the
  conformance gate. Rest of the file untouched except the three
  docs/LESSONS.md path fixes.
- tatr check fixes, 16 findings -> 13 fixed + 3 residue:
  - bad-severity (11/11 fixed), mapping recorded: "OBSERVATION, not a
    defect" -> NIT (text keeps "Observation, not a defect") in
    20260703-132214; "NIT, informational" -> NIT (text keeps
    "Informational:") in 20260703-165400; "verified" x7, "verified, good
    call", "verified, not a finding" -> NIT with the verified note kept as
    a "Verified:..." text prefix in 20260704-220719 (4), 20260704-223846
    (1), 20260705-090557 (4). No LOW/NOTE/INFO/FIXED severities existed,
    so the LOW->MINOR / FIXED->original branches were not needed.
  - closed-unchecked (2/5 tasks fully fixed, 1 partial): 20260704-134600
    all 6 steps ticked (src/feedback/flash.rs with flash_mix,
    NOTES.md-documented pure + 3 ECS tests, lib.rs prelude wiring,
    10_asteroids demo, follow-up task 20260704-155505 exists, verify suite
    per NOTES/review + CI on the landed merge). 20260708-112713 all 6
    steps ticked (src/integrity/{components,blast,plugin}.rs with 14 tests
    + test_support, physics/rigid_body.rs 3 tests, ui/health_display 4 +
    ui/objectives 1 tests, examples/15_integrity.rs, NOTES.md "full nova
    test suites came across intact", shipped via reviewed PRs #2/#3 with
    CI green - close commit b92690f). 20260705-140043 steps 1, 2, 4 ticked
    (landed commit ace3138 + current AGENTS.md state; check-ascii gated by
    CI on the merge); step 3 left unticked (see residue).

Residue (unchecked steps left honestly unticked; user reviews at the
goal's Finish):

- 20260704-102342 (5 steps): task is SUPERSEDED - split per user direction
  into 20260704-103544 / 20260704-103553 / 20260704-103517, where the work
  actually shipped. Nothing was implemented under this id, so no step can
  honestly be ticked.
- 20260705-140043 (1 step): "Re-read the Examples section for any other
  drift while in there" - no close-out notes/RETRO exist and the landed
  commit ace3138 touched only the Module Map and versions, so the re-read
  is unevidenced.
- 20260711-094942 (3 steps): steps annotated "(dropped, premise
  falsified)" / "(dropped, no behavior change shipped)" in the task body,
  with the drop documented in its Outcome section - deliberate drops, not
  unshipped work; ticking them would misrecord.

Check results:

- /home/alex/personal/tatr/tatr check: exit 1 with exactly the 3 residue
  findings above (closed-unchecked 5/1/3); zero bad-severity findings.
- /home/alex/personal/tatr/tatr check --ledger LESSONS.md: zero ledger
  findings (promotion-stalled all resolved); exit is 1 only because the
  same 3 task residue findings fire. Scoped run
  `tatr check 20260720-171843 --ledger LESSONS.md` exits 0.
- DoD cmd note: the literal `! grep -rn "docs/LESSONS" ...` matches two
  task records only - this task's own body (quotes the old path in its
  step text and in the DoD cmd itself, so the literal check can never
  pass) and CLOSED 20260716-084433's historical goal text. Both are
  history, not stale live references; with `--exclude-dir=tasks` the grep
  is clean (exit 0). Live surfaces (AGENTS.md, README.md, docs/, scripts/,
  web/, .github/) have zero references.
- Repo suite: scripts/check-ascii.sh clean; cargo fmt --check clean;
  cargo test 147 unit + 59 doctests passed, 0 failed (1 ignored);
  cargo test --examples all green (incl. 12_bastion's include_str catalog
  fixture tests - the only include_str! in the tree, untouched). Diff is
  markdown-only (13 files), so the clippy compile gates are unaffected;
  no pre-existing failures encountered.
