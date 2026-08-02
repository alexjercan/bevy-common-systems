# Retire docs/: fold reference docs into web/README, examples/README and task NOTES

- PRIORITY: 60
- TAGS: chore, docs
- KIND: TASK
- ACTIVITY: COMPOUNDING
- GATES: PLAN REVIEW RETRO
- RESOLUTION: DONE

## Context

`docs/` holds four files and nothing else: two reference docs
(`dev-harness.md`, `wasm-web-builds.md`), a `README.md` whose "Where records
go" section is verbatim duplicated in `AGENTS.md`, and `plans/README.md`, an
empty index superseded by tatr EPIC tasks (`20260731-172116` is one). Splitting
orientation across `AGENTS.md` + `docs/README.md` gives two homes for one rule.

Each remaining doc has a better home next to what it documents:

| Source | Destination | Why |
| --- | --- | --- |
| `docs/wasm-web-builds.md` | `web/README.md` | it is entirely about `web/`: trunk, `build-games.sh`, per-game `index.html`, audio unlock, Pages |
| `docs/dev-harness.md` usage half | new `examples/README.md` | how to run/verify the examples, next to them |
| `docs/dev-harness.md` rationale half | `tasks/20260704-175421/NOTES.md` | the task that built the harness; folder has TASK+REVIEW, no NOTES |
| `docs/README.md` | dropped | duplicates AGENTS.md "Where records go" |
| `docs/plans/README.md` | dropped | empty index; multi-task plans are tatr EPICs |

`examples/README.md` also absorbs the example table from the root `README.md`
so the list has one home; the root keeps the pitch and links out.

## Steps

1. [x] `git mv docs/wasm-web-builds.md` content into `web/README.md`: keep
   web/README's current structure (showcase parts, adding a docs page, build,
   Pages, adding a game) and fold the wasm doc in as sections after "Build" --
   getrandom gotcha, trunk from repo root, assets/copy-dir, audio + autoplay
   (incl. the iOS ringer-channel note), notes (canvas, `04_status_item`
   exclusion, release size). Resolve the two now-internal links (lines 76, 78)
   to in-page section refs. No content dropped: every heading of the old file
   appears.
2. Write `examples/README.md`: how to run an example (`cargo run --example
   NN_name`, `--features debug`), the example table moved from `README.md`
   (name, one-liner, headline modules), and the headless-verification section
   lifted from `docs/dev-harness.md` "How to use it" (`BCS_AUTOPILOT`,
   `BCS_SHOT`, the `main()` snippet, the autopilot/screenshot mutual
   exclusion). Plain ASCII (`scripts/check-ascii.sh` scans `examples/`).
3. Write `tasks/20260704-175421/NOTES.md` from the rest of
   `docs/dev-harness.md`: "What this is", "Why this shape", "Key API
   decisions", "Proof", "Alternatives considered". Header lines (DATE/TASK/
   SPIKE) become the NOTES preamble.
4. `git rm -r docs/`.
5. Fix references:
   - `README.md`: "More" list drops the `docs/` bullet, points `web/README.md`
     for wasm and `examples/README.md` for the games; example section becomes a
     short pointer to `examples/README.md`.
   - `AGENTS.md`: line 17 (plans -> EPIC tasks, not `docs/plans/`), line 19
     (domain docs -> `web/README.md`, `examples/README.md`), line 32 (layout
     table row for `docs/` removed; add `examples/` README mention), line 38
     ("never loose per-task `.md` in `docs/`" -> reword), line 242
     (`docs/dev-harness.md` -> `examples/README.md`), line 255
     (`docs/wasm-web-builds.md` -> `web/README.md`).
   - `web/games/_shared/audio-unlock.js:26` comment -> `web/README.md`.
   - `.github/workflows/pages.yml`: drop the now-dead `'docs/**'` paths-ignore
     entry (`'**/*.md'` already covers it).
6. Verify: no live reference to `docs/` outside `tasks/` (historical records
   stay untouched); `actionlint` on `pages.yml`; `check-ascii.sh`;
   `cargo build` (unchanged sources, cheap sanity); `tatr check`.

## Definition of Done

- [x] `docs/` gone (cmd: `test ! -e docs`)
- [x] No live `docs/` reference outside `tasks/`
      (cmd: `! grep -rn "docs/" --include='*.md' --include='*.sh' --include='*.js' --include='*.yml' --include='*.rs' --include='*.ts' --include='*.toml' --include='*.html' --include='*.json' . | grep -v '^tasks/' | grep -v node_modules | grep -v '^web/dist'`)
- [x] `web/README.md` covers every heading of the old wasm doc
      (cmd: `for h in getrandom trunk copy-dir autoplay ringer 04_status_item; do grep -q "$h" web/README.md || exit 1; done`)
- [x] `examples/README.md` exists and documents both harness env vars
      (cmd: `grep -q BCS_AUTOPILOT examples/README.md && grep -q BCS_SHOT examples/README.md`)
- [x] `tasks/20260704-175421/NOTES.md` exists with the design rationale
      (cmd: `grep -q "Alternatives considered" tasks/20260704-175421/NOTES.md`)
- [x] ASCII rule holds (cmd: `./scripts/check-ascii.sh`)
- [x] Workflow still valid (cmd: `nix run nixpkgs#actionlint -- .github/workflows/pages.yml`)
- [x] Tracker clean (cmd: `tatr check --ledger LESSONS.md`)

## Notes

- Docs-only change; the only Rust edits are five example comments repointing
  `docs/dev-harness.md` -> `examples/README.md`, so verification is
  `cargo build --examples` rather than the full suite.
- `tasks/` records referencing `docs/...` are historical and stay as written.
- `docs/plans/` has never held a file (`_No multi-task plans yet._`), so
  nothing is lost by dropping it.
- Assumption: the user wants content preserved, not trimmed -- this is a move,
  not an edit pass. Anything genuinely stale gets flagged, not silently cut.

## Close-out

**What / why.** `docs/` is gone. `wasm-web-builds.md` folded into
`web/README.md` (one file now covers the site and the wasm build it ships);
`dev-harness.md` split into `examples/README.md` (how to run and headlessly
verify an example) and `tasks/20260704-175421/NOTES.md` (why the harness has
the shape it does -- the task that built it had TASK+REVIEW but no NOTES);
`docs/README.md` dropped as a duplicate of AGENTS.md "Where records go";
`docs/plans/` dropped, never having held a file, with multi-task planning now
expressed as tatr EPIC + STORY tasks (`20260731-172116` is the precedent).
Each doc now sits next to the thing it documents, and orientation has one home.

`examples/README.md` also absorbed the example list from the root `README.md`,
which had drifted (it said "fourteen" and stopped at `14_breach`, missing
`15_integrity`). The root README now points at it and keeps the pitch.

**Alternatives.** (a) Keep `docs/` and only delete the duplicate README --
rejected, the goal is retiring the folder. (b) Put the harness rationale in
`examples/README.md` too -- rejected, "why the API is shaped this way,
alternatives rejected" is a task record by this repo's own convention, and the
usage half is what an example author needs. (c) Trim stale content while
moving -- rejected, a move plus an edit pass is two reviews in one diff; the
content moved verbatim except for one paragraph in the trunk section that
duplicated the "Adding a game" list already above it in `web/README.md`.

**Difficulties.** The DoD's no-stale-reference grep was written with a `^./`
filter but `grep -rn ... .` emits paths without the `./` prefix, so the first
run looked clean while five example files, `assets/sounds/README.md` and the
`pages.yml` header comment still pointed at `docs/`. Fixed the proof, then the
references. Also dropped the now-dead `'docs/**'` entry from `pages.yml`
paths-ignore (`'**/*.md'` already covered it).

**Evidence.** All eight DoD proofs pass: `docs/` gone, no live `docs/`
reference outside `tasks/` (historical records left as written), every heading
of the old wasm doc present in `web/README.md`, both harness env vars
documented, NOTES.md carries the rationale, `check-ascii.sh` clean,
`actionlint` clean on `pages.yml`, `tatr check --ledger` exit 0.
`cargo build --examples` exit 0 (3m01s, only the known `proc-macro-error2`
future-incompat note).

**Reflection.** A `grep -v` path filter is itself a proof that can be wrong in
the silent direction -- it fails open. Worth writing such a proof so the
expected-nonzero hits are named, or running it once with the filter removed to
see what it is actually suppressing.
