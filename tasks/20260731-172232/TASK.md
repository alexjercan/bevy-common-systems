# KISS pass: modding/ + persist/ + macros subcrate

- STATUS: IN_PROGRESS
- PRIORITY: 60
- TAGS: chore, kiss, modding, persist
- KIND: STORY
- FLOW STEP: REVIEWING
- PLAN STATUS: APPROVED
- PARENT: 20260731-172116
- DEPENDS ON: 20260731-172208

Scope: `src/modding/` (`events.rs`, `registry.rs`, `mod.rs`), `src/persist/` (`mod.rs`, `backend.rs`), and the `bevy_common_systems_macros/` subcrate.

`modding/events.rs` (404 lines before tests) and `registry.rs` (320) are the
two biggest non-mesh code bodies, 34 non-doc comments between them. `persist/`
adds 14 more, largely platform narration around the native/wasm backend split.

The macros subcrate is small but its rustdoc is the documented workaround for
the `EventKind` default-`Info`-path footgun (see AGENTS.md gotchas and
`tasks/20260703-095509`). Verify that guidance is still accurate rather than
trimming it.

## Steps

- [x] Read every file in scope end to end; list each non-doc comment with a keep/compact/drop call in `NOTES.md`.
- [x] Drop code-restating and task-narration comments.
- [x] Compact each kept comment to one tagged line (`NOTE:` / `FIXME:` / `BUG:` / `TODO:`), HUID only when it points at a live task record.
- [x] Audit rustdoc (`//!`, `///`) for stale claims; fix what is wrong, leave style alone.
- [x] Measure code-before-tests per file; split only where the file carries more than one concern, and record the decision (split or keep) in `NOTES.md`.
- [x] Run the full verification suite.

## Definition of Done

- Every kept non-doc comment in scope is a tagged block; base has 24 untagged BLOCKS -- `events.rs` 8, `registry.rs` 8, `persist/mod.rs` 5, `persist/backend.rs` 2, macros `lib.rs` 1 (the Story's "34" and "14" count comment LINES, a different unit) (cmd: `./scripts/check-comment-tags.sh src/modding src/persist bevy_common_systems_macros/src` exits 0).
- No non-doc comment in scope carries a bare tatr HUID (cmd: `grep -rnE '^\s*//([^/!]|$)' src/modding src/persist bevy_common_systems_macros/src | grep -E '20[0-9]{6}-[0-9]{6}' | grep -vE 'NOTE:|FIXME:|BUG:|TODO:'` prints nothing).
- The two pre-existing rustdoc warnings in scope are gone: `EventHandlerIndex`'s doc links to the private `queue_system` and `maintain_handler_index` (`modding/events.rs:302,318`). The other four warnings `cargo doc` emits belong to `helpers/` and `input/`, which are cluster 20260731-172233's, not this task's (cmd: `nix develop --command cargo doc --no-deps --features debug 2>&1 | grep -cE '^\s+--> src/(modding|persist)/'` -> 0).
- The macros subcrate's `EventKind` guidance is verified against the code, not trimmed on sight: either it still describes real behaviour or the record says how it drifted (manual: read `tasks/20260731-172232/NOTES.md`).
- `NOTES.md` records a keep/compact/drop call for all 24 comment blocks plus per-file code-before-tests numbers behind each split-or-keep decision (manual: read `tasks/20260731-172232/NOTES.md`).
- Task artifacts and ledger lint clean (cmd: `tatr check --ledger LESSONS.md`).
- Formatting clean (cmd: `nix develop --command cargo fmt --check`).
- Lints clean in both feature configurations (cmd: `nix develop --command cargo clippy --all-targets` and `nix develop --command cargo clippy --all-targets --features debug`).
- Tests pass in both feature configurations and for examples (cmd: `nix develop --command cargo test`, `... cargo test --features debug`, `... cargo test --examples`).
- Plain-ASCII rule holds (cmd: `./scripts/check-ascii.sh`).
- Public API unchanged: no item renamed, removed, or moved out of its prelude (manual: `git diff master -- src/modding src/persist bevy_common_systems_macros/src` shows no `pub` signature or prelude re-export line changed).

## Close-out

**What.** 24 untagged non-doc comment blocks across `src/modding/`,
`src/persist/` and `bevy_common_systems_macros/src/` -> 0; bare tatr HUIDs were
already 0 and stay 0. No file split -- every candidate was measured and KEPT.
Two in-scope rustdoc warnings fixed (`EventHandlerIndex` linked the private
`queue_system` and `maintain_handler_index`; both named in prose instead, as
`integrity/plugin.rs` already does). One live documentation defect found and
corrected: AGENTS.md's `EventKind` gotcha, plus a new regression test guarding
the behaviour it described.

**Why no split.** `modding/events.rs` at 404 code lines is the crate's largest
remaining body and was the obvious candidate. The test that carried the two
previous clusters' splits -- do the parts have DISJOINT dependency sets? --
fails here. Its four layers (traits; the `EventHandler` component; `GameEvent` /
`Commands::fire`; the plugin's queue + index + dispatcher) are not separable:
layer 4 needs all of 1-3, since `EventHandlerIndex` stores `EventHandler<W>`
clones generic over `W: EventWorld` and the dispatcher reads `GameEvent` while
calling both trait objects. No cut reduces what any resulting file imports --
unlike `mesh/slice.rs`, whose kernel imported only `bevy::prelude` after the
move. `registry.rs` (320) is one concern end to end (parse `HandlerSpec`,
resolve names, build handlers), its bulk past 320 being the rig that exercises
exactly that. `persist/` is already split along its one seam.

**The guidance drift.** The task said to VERIFY the macros `EventKind`
guidance rather than trim it, and it turned out to be wrong. AGENTS.md carried
*"default `Info` path does not resolve ... always pass `#[event_info(...)]`"*.
True when written; the derive was fixed since and the default is now
`quote! { () }`, which satisfies the `Info` bounds and needs no import at the
derive site. The in-code comment at `macros/src/lib.rs:12` recorded the fix, but
the repo-level guidance never caught up, so it kept steering callers away from a
working path. Nothing in the repo derives without `#[event_info(...)]`
(`examples/03_modding.rs` is the only derive site), so the default was live but
UNEXERCISED -- which is why the stale warning stayed invisible. Added
`modding::events::tests::attribute_less_derive_defaults_to_no_payload`.

**Alternatives.** Splitting `events.rs` along its four layers (rejected on the
dependency-set evidence above -- it would move code between files while leaving
`use super::*` behind, which is churn, not structure). Deleting the macros
guidance as noise (rejected: the task called for verification, and the "do not
name a concrete type here" half is still the live hazard, so it stayed as the
in-code `NOTE:`). Asserting the derive default with a runtime `let` binding
(rejected after clippy flagged `let_unit_value`; replaced by a
`fn requires_unit_payload<E: EventKind<Info = ()>>()` call, which is a stronger
guard anyway -- the original defect was a COMPILE failure, so a compile-time
bound is what actually reproduces it).

**Difficulties.** The DoD initially miscounted the `cargo doc` warnings as four
in scope; one of them ("redundant explicit link target") lives in
`src/helpers/pointer.rs`, which belongs to cluster 20260731-172233. Corrected
the DoD mid-task to say two in scope rather than quietly meeting a wrong number.
The first version of the new test also introduced a clippy warning of its own --
caught by the run, not by review.

**Evidence.** `check-comment-tags.sh src/modding src/persist
bevy_common_systems_macros/src` exit 0 (24 -> 0); bare-HUID grep 0 matches;
`cargo doc --no-deps --features debug` in-scope warnings 0 (down from 2; the 4
remaining are pre-existing in `helpers/`/`input/`, cluster 20260731-172233's);
`check-ascii.sh` exit 0; `cargo fmt --check` exit 0; `cargo clippy
--all-targets` exit 0 and `--features debug` exit 0 (only the expected
`proc-macro-error2` future-incompat note); `cargo test` 148 + 59 (master
baseline 147 + 59, +1 the new derive test), `--features debug` 155 + 66
(baseline 154 + 66), `--examples` all 15 ok; `tatr check --ledger LESSONS.md`
exit 0; `git diff master -- src/modding src/persist
bevy_common_systems_macros/src` changes no `pub` signature or prelude line
(the only matches are two comment lines that mention `pub(super)`).

**Reflection.** A gotcha with no test behind it decays silently. The
`EventKind` default was documented as broken for the life of the fix because
the repo had exactly one derive site and it always passed the attribute, so
nothing ever exercised the path the warning was about. Every entry in the
AGENTS.md Gotchas list is a claim about current behaviour; the ones with no
executing witness are the ones to distrust.
