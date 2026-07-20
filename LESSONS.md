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

## Technical lessons

- `reuse-the-kit` (x2): Reuse the crate kit (TempEntity, glowing_material, ExplodeMesh, CameraShakeInput) end to end with zero new deps, and use HDR `glowing_material` so effects bloom -- unlit kills bloom. 20260705-101224.
- `try-entity-commands-for-fire-and-forget` (x2): For any "command applied to a stale entity" panic reach for `try_*` entity commands; to test cross-system ordering deterministically disable `auto_insert_apply_deferred`. 20260705-155230.
- `wasm-getrandom-and-build-profile` (x2): `rand` on wasm needs both `getrandom = { features = ["wasm_js"] }` and `--cfg getrandom_backend="wasm_js"`; `split-debuginfo = "unpacked"` + `debug = "line-tables-only"` halves test-build peak RAM. 20260703-000001, 20260703-000003.
- `release-gates` (x1): Version bevy_common_systems to Bevy's minor (0.19.x -> Bevy 0.19.x), and run cargo-about with an accepted-permissive set so generation fails on copyleft/unknown-license deps. 20260716-000016.

## Pending promotions (3+ occurrences, user decides)

- `evidence-before-claim` (x9): Never write a behavioral claim (test comment, assertion, review response, docs sentence, task note) before the run or assertion that backs it is in hand. 20260704-170738, 20260704-220736.
- `regression-test-must-fail-without-fix` (x4): Flip the code back to its buggy state and confirm the new test actually fails; a green regression test can be green because it looks in the wrong place. 20260705-155230, 20260711-091519.
- `full-command-output` (x6): Never conclude "there is no X" or "build passed" from a piped/truncated `tail`; redirect to a file, read the relevant section in full, and check `$?` of the real command not a later `echo`. 20260703-094842, 20260703-144934.
- `verify-api-in-source` (x8): Verify any unfamiliar API signature, `From`/`new` impl, or symbol against the installed crate source before writing it; grep the exact symbol, do not trust memory or AGENTS.md. 20260705-134942, 20260704-103517.
- `run-the-example` (x7): Before marking an interactive example done, actually run it through its real entry point (build then run as separate `&&`-chained steps), guarding for `$DISPLAY` or a timeout in headless sessions. 20260703-170744, 20260705-101224.
- `verify-observable-effect` (x6): For each control an example advertises, verify its observable effect (view moved, entity selected) by diffing state frame to frame, not a proxy log line or "app stayed up". 20260705-090640, 20260704-220736.
- `clippy-all-targets-gate` (x4): Run `cargo clippy --all-targets` (or `--examples`) as the compile gate before the first "done"; a plain `cargo build` skips examples and cfg(test)-only dead code. 20260704-165400, 20260705-092619.
- `sprout-first` (x4): Make sprouting a fresh worktree the unskippable first action of a task, created from local HEAD when it must build on unpushed commits; create or commit the tatr task before branching so it exists on the branch. 20260704-161526, 20260705-151821.
- `grep-whole-tree-before-rename` (x5): When renaming a placeholder, bumping a version, or moving a feature between surfaces, grep the entire tree (src AND examples AND docs) for every reference in the same pass. 20260703-152619, 20260703-121414.
- `one-tatr-new-per-call` (x3): Space `tatr new` calls across separate tool invocations; second-resolution timestamp IDs collide and silently overwrite in the same shell line. 20260703-150200, 20260703-173128.
- `pkill-by-pid` (x4): Kill background runs by the `$!` PID captured at launch, never a `pkill -f` pattern that also matches the killing command's own args; anchor `xdotool search --name` to `^title$`. 20260705-163112, 20260704-173937.
- `no-concurrent-git-same-tree` (x3): On a shared checkout do not merge into master while a human commits there; re-fetch and check `git merge-base --is-ancestor master <branch>` right before any squash-merge. 20260703-212303, 20260704-175422.
- `negative-result-is-a-deliverable` (x4): A "we decided not to" outcome still leaves the crate better (module doc, testable snippet, pointer) and records the reasoning; document the negative result, do not leave it silent. 20260704-161522, 20260704-161526.
- `split-verifiable-from-manual` (x3): Split "assets load / logic runs" (headless-verifiable) from "asset is audible/visible / transition fires" (needs manual play-test), and say which real paths a shortcut harness does NOT exercise. 20260703-163328, 20260705-103238.
- `reset-shared-state-in-same-commit` (x7): When adding an accumulator, timer, or per-run resource, grep the reset/`start_run` path and add its reset in the same commit; extract a helper once the count exceeds three. 20260703-132207, 20260704-103544.
- `order-against-a-real-edge` (x6): Encode system-ordering correctness with a direct `A.before(B)` edge against a concrete member, never by ordering both against a third set that may be empty; reason about the >1-substep fixed-timestep case. 20260704-134500, 20260704-190405.
- `input-driver-every-state` (x4): A system writing an Input that a plugin integrates every frame must run in every state (or zero the Input on state exit); state-gating leaves a stale last value. 20260703-165432, 20260704-220736.
- `extract-pure-and-test-that` (x6): Extract decision functions and per-dt steps as pure functions and unit-test those; MinimalPlugins TimePlugin clobbers manual `Time::advance_by`, and screenshots at state entry are not gameplay proof. 20260705-132238, 20260703-132214.
- `copy-bevy-019-ui-verbatim` (x5): Copy Bevy 0.19 UI/visual idioms verbatim from an existing example (FontSize::Px, per-camera AmbientLight, TextLayout literal, BorderRadius/Node fields) and use percentage widths for responsive grids. 20260703-150200, 20260704-143000.
- `headless-test-plugin-set` (x4): Headless test apps must add the plugins the code touches -- `init_asset::<T>()` before building an asset-loading resource, and MinimalPlugins + Transform/Asset/Mesh + PhysicsPlugins + `app.finish()` for avian. 20260705-094950, 20260711-091519.
- `doctest-needs-state-or-hidden-fn` (x3): App-build doctests need StatesPlugin (not MinimalPlugins alone) or must live inside a hidden `# fn wire_up() {}` so they type-check without requiring headless graphics; `cargo test --doc` catches the runtime panic. 20260704-175425, 20260705-000005.
- `check-plugin-self-added-deps` (x3): When wiring a crate plugin into an example, read its `build()` for self-added dependency plugins (PopupPlugin -> TweenPlugin) to avoid runtime-only duplicate-plugin panics. 20260705-132200, 20260704-201801.
- `oracle-from-defining-library` (x3): Construct test oracles for convention-sensitive math (frame order, handedness, sign) with the library that defines the convention, never re-derived next to the code under test; `surface_frame` beats `from_rotation_arc` for stable upright-on-sphere. 20260711-091519, 20260705-154507.
- `harvest-by-reading-evidence` (x5): Let concrete reference code draw the harvest line -- survey precedent and homes first, reproduce bodies byte-for-byte, refactor call sites as the test, and delete the now-dead markers from the consumer. 20260705-090557, 20260704-161513.
