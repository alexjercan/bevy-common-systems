# Lessons ledger

One or two lines per lesson: slug, one sentence, an occurrence count, and a
task id or two (an id resolves to `tasks/<id>/RETRO.md`). /compound and
/lessons append new lessons or bump counts; two lines is the cap. Keep entries
SHORT - when a new occurrence adds a variant, sharpen the one sentence instead
of appending a paragraph. At three occurrences a lesson moves to Pending
promotions for the user to fold into a guideline, template, or tool. Counts
stay bare - (xN) - until a lifecycle event annotates them:
(xN, PROMOTED <date> -> <target>), (xN, absorbed by <tool or template>,
<date>), or (xN, RETIRED <date>: <reason>). Seeded 2026-07-16 from ~100
retros.

## Process lessons

Promoted 2026-07-20 (task 20260720-220050) into the global `~/AGENTS.md` and the
flow skills, which run in every repo - kept here (annotated) as the paid record.

- `evidence-before-claim` (x9, PROMOTED 2026-07-20 -> global ~/AGENTS.md: an edit is a hypothesis until the artifact shows it): Never write a behavioral claim before the run or assertion that backs it is in hand. 20260704-170738, 20260704-220736.
- `full-command-output` (x7, PROMOTED 2026-07-20 -> global ~/AGENTS.md: never let a pipe/echo eat the exit code): redirect to a file, read the section in full, check `$?` of the real command. A `tail -N` that hides a warning also poisons the RECORD, even when the exit code is honestly 0. 20260703-094842, 20260731-172208.
- `pkill-by-pid` (x4, PROMOTED 2026-07-20 -> global ~/AGENTS.md: kill by recorded PID, never `pkill -f`): anchor `xdotool search --name` to `^title$`. 20260705-163112, 20260704-173937.
- `regression-test-must-fail-without-fix` (x4, PROMOTED 2026-07-20 -> review skill: "would it fail with the fix deleted?"): flip the code buggy and confirm the new test reds. 20260705-155230, 20260711-091519.
- `verify-api-in-source` (x8, PROMOTED 2026-07-20 -> plan skill: read the dependency's source/probe before designing around it): grep the exact symbol, do not trust memory or AGENTS.md. 20260705-134942, 20260704-103517.
- `sprout-first` (x4, PROMOTED 2026-07-20 -> work skill: sprout is the first action; commit the task before branching): created from local HEAD when it builds on unpushed commits. 20260704-161526, 20260705-151821.
- `grep-whole-tree-before-rename` (x5, PROMOTED 2026-07-20 -> work skill doc-surface sweep): grep src AND examples AND docs for every reference in one pass before a rename/move/version bump. 20260703-152619, 20260703-121414.
- `one-tatr-new-per-call` (x3, PROMOTED 2026-07-20 -> tatr skill: one `tatr new` per command): second-resolution IDs collide and (pre-0.2.0) silently overwrote. 20260703-150200, 20260703-173128.
- `no-concurrent-git-same-tree` (x3, PROMOTED 2026-07-20 -> flow land protocol: `git merge-base --is-ancestor master <branch>` right before squash-land): do not merge into master while a human commits there. 20260703-212303, 20260704-175422.
- `negative-result-is-a-deliverable` (x4, PROMOTED 2026-07-20 -> spike/compound skills: a "we decided not to" is recorded, not silent): still leave a module doc / testable snippet / pointer and the reasoning. 20260704-161522, 20260704-161526.
- `convention-and-checker-are-one-artifact` (x1): when the deliverable is a RULE, diff the prose's scope against the tool's matcher and name each gap an exemption or a bug -- a checker narrower than its wording is worse than none, since exit 0 reads as compliance. 20260731-172208.
- `fmt-runs-last` (x2): rustfmt reflows what you just wrote - an end-of-line comment after `{` or inside a multi-line call becomes an untagged BLOCK, and a narrowed `fn` signature collapses back to one line. Run fmt, THEN the checkers, after every edit round including review fixes; to label a magic number inside a construct, extract a named helper instead. 20260731-172223, 20260731-172224.
- `split-along-the-test-seam` (x1): when deciding whether to split a file, look at its test modules first - two that share no helper are a seam the author already found. Measure code-before-tests, not total lines: a 564-line file with 150 of code is one concern plus a rig. 20260731-172223.
- `record-numbers-from-a-rerun` (x2): every count that lands in a record comes from re-running its command at write time, with the UNIT named - a file measured as 33 comment lines and 15 comment blocks is one file and two numbers, and writing either from memory of the reading pass mixes them. Cheapest guard: paste the per-file breakdown, so the total is reconstructible instead of asserted. The guard is easiest to apply to the number you expect to be challenged and easiest to skip on a downstream one that feels like bookkeeping - a file measured at 326 lines before a later edit shipped as 326 when it was 328. 20260731-172224, 20260731-172233.
- `state-what-the-checker-cannot-see` (x3, PENDING PROMOTION): when a pass takes a route its own project checker is blind to (test intent promoted to `///` inside `#[cfg(test)]`, which rustdoc never renders and the tag checker exempts), record the decision in the same commit and draw the line in the convention doc - an unstated legitimate route is indistinguishable from gaming the check. Then check your OWN diff against that line: on its second outing the same author put a value guard behind the `///`, which is the misuse the rule was written to stop. Authoring a convention does not confer compliance with it, and INHERITING one does not either - third outing, third author-side slip, one file where three magic inputs' justification went behind the `///` with the intent. 20260731-172224, 20260731-172232, 20260731-172233.
- `a-gotcha-needs-a-witness` (x1): a documented gotcha with no test exercising the path it warns about decays silently and keeps steering callers for the life of the fix. The `EventKind` derive default was documented as not resolving long after it was fixed, invisibly, because the repo's one derive site always passed the attribute. Treat every "always do X because Y is broken" note as a claim about current behaviour: find its executing witness or write one, and prefer a compile-time bound when the original defect was a compile failure. 20260731-172232.
- `probe-a-new-checker-both-ways` (x2): before trusting a check script OR a measuring grep whose output lands in a record, probe it on a case it must match and one it must not; a clean run on real sources cannot reveal a matcher that silently covers less than intended, and correct neighbouring rows are not evidence about the row you have not probed. A `#[cfg(test)]` grep missed `#[cfg(all(test, not(target_arch = "wasm32")))]` and reported a 148-line file as 200 lines of code. Complements `regression-test-must-fail-without-fix` (red on a violation) with coverage. 20260731-172208, 20260731-172232.
- `compute-a-complement-scope` (x1): a scope stated as a complement ("everything not claimed by the other children", "whatever the previous pass missed") is a QUERY - run it against the tree once and let its output be the enumeration, rather than hand-listing what you believe the complement contains. A KISS cluster planned and gated against its own ten-directory list; `src/material.rs` was claimed by no sibling, sat in nobody's list, and as the LAST task there was no downstream catcher - the epic-wide checker would have exited 1 after the epic closed. Corollary: when the project ships a checker, the final task in a multi-task cleanup runs it UNSCOPED at least once, since per-task path scoping is exactly what lets a gap hide from every task while staying visible to the whole. 20260731-172233.

## Technical lessons

- `reuse-the-kit` (x2): Reuse the crate kit (TempEntity, glowing_material, ExplodeMesh, CameraShakeInput) end to end with zero new deps, and use HDR `glowing_material` so effects bloom -- unlit kills bloom. 20260705-101224.
- `try-entity-commands-for-fire-and-forget` (x2): For any "command applied to a stale entity" panic reach for `try_*` entity commands; to test cross-system ordering deterministically disable `auto_insert_apply_deferred`. 20260705-155230.
- `wasm-getrandom-and-build-profile` (x2): `rand` on wasm needs both `getrandom = { features = ["wasm_js"] }` and `--cfg getrandom_backend="wasm_js"`; `split-debuginfo = "unpacked"` + `debug = "line-tables-only"` halves test-build peak RAM. 20260703-000001, 20260703-000003.
- `size-limits-against-the-heaviest-config` (x1): a resource limit (job cap, memory budget, timeout) must be measured against the heaviest build configuration, not the default one. Sizing the cargo job cap on default features gave 7; `--features debug` links egui into every binary and busted the target at 18.4 GB, forcing 5. The default-feature number is informational, the worst case is the one that decides. 20260731-210044.
- `falsify-a-bound-with-a-subset` (x1): when a measurement claims to bound a whole, measure one part of it too -- a subset that exceeds the claimed total refutes it for free. A `cargo test` peak reported as 13.1 GB stood for a day; `cargo test --doc`, a strict subset, measured 16.4 GB at the same cap and withdrew it. The cheapest possible check on any single-run headline number. 20260731-210044.
- `bcs-no-full-test-suite` (x1): a bare `cargo test` here links ~76 full Bevy+avian binaries (1 lib + 15 examples + 60 doctests) and peak RAM is `(links in flight) x (binary size)`, so it exhausts a 31 GB box at cargo's default job count. Run it only under the `flake.nix` caps (`CARGO_BUILD_JOBS`/`RUST_TEST_THREADS`), and measure with `./scripts/sample-peak-rss.sh` rather than assuming. `RUST_TEST_THREADS` is the only lever on the doctest phase -- `[build] jobs` and `[profile.dev]` never reach rustdoc's harness. 20260703-000003, 20260731-210044.
- `release-gates` (x1): Version bevy_common_systems to Bevy's minor (0.19.x -> Bevy 0.19.x), and run cargo-about with an accepted-permissive set so generation fails on copyleft/unknown-license deps. 20260716-000016.

Promoted 2026-07-20 (task 20260720-220050) into this repo's AGENTS.md - kept here
(annotated) as the paid record. The five marked "Conventions/Promoted-lessons"
were freshly folded into AGENTS.md's Conventions section; the rest were already
documented in the AGENTS.md section named.

- `clippy-all-targets-gate` (x4, PROMOTED 2026-07-20 -> AGENTS.md Build/Verify/Run): `cargo clippy --all-targets` is the compile gate; plain `cargo build` skips examples and cfg(test) dead code. 20260704-165400, 20260705-092619.
- `run-the-example` (x7, PROMOTED 2026-07-20 -> AGENTS.md Build/Verify/Run + Gotchas): run an interactive example through its real entry point (build then run), guarding `$DISPLAY`/timeout. 20260703-170744, 20260705-101224.
- `verify-observable-effect` (x6, PROMOTED 2026-07-20 -> AGENTS.md Build/Verify/Run): verify each advertised control's effect by diffing state frame to frame, not a proxy log. 20260705-090640, 20260704-220736.
- `order-against-a-real-edge` (x6, PROMOTED 2026-07-20 -> AGENTS.md Conventions: ordering caveat): `A.before(B)` against a concrete member, never both against a maybe-empty third set. 20260704-134500, 20260704-190405.
- `extract-pure-and-test-that` (x6, PROMOTED 2026-07-20 -> AGENTS.md Build/Verify/Run): extract decision/per-dt functions as pure fns and unit-test those; MinimalPlugins TimePlugin clobbers manual `Time::advance_by`. 20260705-132238, 20260703-132214.
- `copy-bevy-019-ui-verbatim` (x5, PROMOTED 2026-07-20 -> AGENTS.md Gotchas): copy Bevy 0.19 UI idioms verbatim (FontSize::Px, BorderRadius/Node fields, TextLayout); percentage widths for responsive grids. 20260703-150200, 20260704-143000.
- `headless-test-plugin-set` (x4, PROMOTED 2026-07-20 -> AGENTS.md Build/Verify/Run): headless apps add the plugins the code touches -- `init_asset::<T>()`, MinimalPlugins + Transform/Asset/Mesh + PhysicsPlugins + `app.finish()` for avian. 20260705-094950, 20260711-091519.
- `doctest-needs-state-or-hidden-fn` (x3, PROMOTED 2026-07-20 -> AGENTS.md Build/Verify/Run): App-build doctests need StatesPlugin or a hidden `# fn wire_up(){}`; `cargo test --doc` catches the panic. 20260704-175425, 20260705-000005.
- `oracle-from-defining-library` (x3, PROMOTED 2026-07-20 -> AGENTS.md): build convention-sensitive test oracles with the defining library (`surface_frame` beats `from_rotation_arc`). 20260711-091519, 20260705-154507.
- `reset-shared-state-in-same-commit` (x7, PROMOTED 2026-07-20 -> AGENTS.md Conventions/Promoted-lessons): grep the reset/`start_run` path and add an accumulator/timer's reset in the same commit; extract a helper past three. 20260703-132207, 20260704-103544.
- `input-driver-every-state` (x4, PROMOTED 2026-07-20 -> AGENTS.md Conventions/Promoted-lessons): a system writing an Input a plugin integrates every frame runs in every state, or zero the Input on state exit. 20260703-165432, 20260704-220736.
- `check-plugin-self-added-deps` (x3, PROMOTED 2026-07-20 -> AGENTS.md Conventions/Promoted-lessons): read a plugin's `build()` for self-added deps (PopupPlugin -> TweenPlugin) to avoid duplicate-plugin panics. 20260705-132200, 20260704-201801.
- `harvest-by-reading-evidence` (x5, PROMOTED 2026-07-20 -> AGENTS.md Conventions/Promoted-lessons): survey precedent/homes, reproduce bodies byte-for-byte, refactor call sites as the test, delete dead markers. 20260705-090557, 20260704-161513.
- `split-verifiable-from-manual` (x3, PROMOTED 2026-07-20 -> AGENTS.md Conventions/Promoted-lessons): split headless-verifiable from manual-play-test, and name which real paths a shortcut harness does NOT exercise. 20260703-163328, 20260705-103238.

- `state-what-the-checker-cannot-see` (x3, PROMOTED 2026-08-01 -> task 20260801-093730): a `///` on a test fn inside `#[cfg(test)]` is the right home for what the test PROVES, but rustdoc never renders it and the tag checker exempts it -- so it gets misused to move a VALUE guard out of the body. Promoted to the checker, not prose: teach `check-comment-tags.sh` to flag a `///` test fn whose body holds an unexplained numeric literal. 20260731-172224, 20260731-172232, 20260731-172233.

## Pending promotions (3+ occurrences, user decides)

None.

Earlier: the 24 x3+ lessons were resolved and promoted on 2026-07-20
(task 20260720-220050): process lessons into the global `~/AGENTS.md` and the
flow skills, domain lessons into this repo's AGENTS.md (see the annotations above).
