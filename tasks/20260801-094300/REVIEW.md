# Review: Retire docs/: fold reference docs into web/README, examples/README and task NOTES

- TASK: 20260801-094300
- BRANCH: chore/retire-docs-folder
- WORKTREE: /home/alex/.cache/sprouts/bevy-common-systems/chore/retire-docs-folder
- BASE: master

## Round 1

- REVIEWER: out-of-context
- VERDICT: REQUEST_CHANGES

Out-of-context general-purpose subagent (`a039f7feb090c1e35`), given
only the task ID, branch/worktree, review dimensions and record format. The
primary re-ran the stale-reference sweep and re-derived the `05_explode` input
claim from `examples/05_explode.rs:126` before accepting the findings.

### Findings

- [x] R1.1 (MAJOR) `web/games/*/index.html` -- 18 live references to the
  deleted `docs/wasm-web-builds.md` across 9 game pages (the audio-unlock
  comment ~line 19 and the assets comment ~line 33-37 of each), missed because
  the DoD's sweep `--include` list covered `.md .sh .js .yml .rs .ts .toml` but
  not `.html`. The proof passed as written; what it was written to scan was too
  narrow -- the same fail-open shape the close-out's Reflection names, fixed on
  the `grep -v` half but not the `--include` half.
  **Change:** repoint all 18 to `web/README.md`; widen the DoD proof with
  `--include='*.html' --include='*.json'`.
  **Response:** done. `sed -i 's#docs/wasm-web-builds\.md#web/README.md#g'
  web/games/*/index.html`; DoD proof widened. Re-run of the widened sweep now
  exits 1 (no matches) across the whole worktree outside `tasks/`.

- [x] R1.2 (MINOR) `AGENTS.md:212` -- the `05_explode` row says "Space slices a
  mesh", but `examples/05_explode.rs:126` gates on
  `input.just_pressed(MouseButton::Left)`, and the new `examples/README.md:29`
  (correctly) says Left Mouse Button. Two adjacent tables contradicting each
  other, with AGENTS.md the wrong one.
  **Change:** replace "Space" with "Left Mouse Button".
  **Response:** done.

- [x] R1.3 (NIT) `examples/README.md:41` -- "see the example table in AGENTS.md
  for the task IDs" is unfulfillable for `01`-`05`, which have no task IDs in
  that table.
  **Change:** soften.
  **Response:** done -- "carries the task IDs for the larger games".

- [x] R1.4 (NIT) `examples/README.md:16-19` -- "Most of them follow the
  06_fruitninja shape ... and a wasm build" overstates: 9 of 15 have a wasm
  build (`web/scripts/build-games.sh`), and `01`-`05` have no states shape.
  **Change:** scope the claim.
  **Response:** done -- "The games from `06_fruitninja` on ... and (for most of
  them) a wasm build".

### Verified clean

- **No content loss.** `master:docs/wasm-web-builds.md` lines 6-203 are
  byte-identical to `web/README.md:84-275` except: the one intentionally
  dropped paragraph ("Adding a game later is a small change...", whose three
  facts all survive in the `## Adding a game` list above it), the
  "trunk must run from the repo root" subsection moved earlier verbatim, and
  `## Notes` -> `## Wasm notes`. The close-out's one-paragraph claim is exact.
  `docs/dev-harness.md` splits losslessly: rationale sections verbatim into
  `tasks/20260704-175421/NOTES.md`, "How to use it" (both code blocks intact)
  into `examples/README.md`. `docs/README.md` duplicated AGENTS.md "Where
  records go"; `docs/plans/README.md` was an empty stub and
  `tasks/20260731-172116` is `KIND: EPIC`, so the AGENTS.md rewording is
  factual.
- **Links.** Every new relative link resolves; both of `web/README.md`'s
  newly-internalized "see ... below" refs now sit above their targets.
- **Harness accuracy.** Every API claim in the new `examples/README.md` checks
  out against `src/debug/harness/`: `BCS_AUTOPILOT` / `BCS_SHOT` env names,
  `.hold` / `.input` / `.settle_frames`, the default `screenshot.png`, and the
  `PreUpdate ... .after(InputSystems)` ordering.
- **AGENTS.md consistency.** No line names a path that no longer exists; the
  two retargeted gotchas point at files that carry the content.
- **DoD proofs** re-run from the worktree root: all pass (`cargo build
  --examples` not re-run -- reported exit 0 pre-review, and the only Rust edits
  are five comment lines).

One open MAJOR at the end of round 1 (R1.1).

## Round 2

- REVIEWER: primary (fix-verification round; no new code paths, all four
  findings are one-line reference or prose edits)
- VERDICT: APPROVE

All four findings addressed in the same worktree; the primary re-ran the
widened sweep (exit 1, no matches), `./scripts/check-ascii.sh` (clean),
`nix run nixpkgs#actionlint -- .github/workflows/pages.yml` (exit 0) and
`tatr check --ledger LESSONS.md` (exit 0). The R1.1 fix is comment-only inside
`web/games/*/index.html` (`<!-- -->` blocks, no trunk directives touched), so it
cannot affect the wasm build; the R1.2-R1.4 fixes are markdown prose.

No pending `manual:` items.
