# KISS pass: feedback/ tween/ ui/ transform/ + small modules

- PRIORITY: 50
- TAGS: chore, kiss, ui, tween
- KIND: STORY
- ACTIVITY: COMPOUNDING
- GATES: PLAN REVIEW RETRO
- RESOLUTION: DONE
- PARENT: 20260731-172116
- DEPENDS ON: 20260731-172208

Scope: everything not claimed by the other children -- `src/feedback/`, `src/tween/`, `src/ui/`, `src/transform/`, `src/audio/`, `src/health/`, `src/helpers/`, `src/input/`, `src/scoring/`, `src/time/`. Fifth and last child of epic 20260731-172116.

Expect mostly deletion, few or no splits -- say so in `NOTES.md` rather than
manufacturing structure.

`ui/status.rs` documents that its closures run in an exclusive system every
frame; that is a performance contract, not fluff.

## Baseline (measured 2026-08-01, this branch point)

`./scripts/check-comment-tags.sh` over the ten scope dirs: **exit 1, 106
untagged non-doc comment blocks** across 21 files. Top: `feedback/flash.rs` 24,
`feedback/screen_flash.rs` 11, `ui/touchpad.rs` 10, `tween/mod.rs` 8,
`health/mod.rs` 8, `transform/point_rotation.rs` 7. 14 files carry 4 or fewer;
14 more files in scope carry zero.

(Blocks, not lines. The Story text's "28 / 24 / 13 / 10" counted comment LINES
-- a different unit. Recording both, per `record-numbers-from-a-rerun`.)

Bare tatr HUIDs in non-doc comments in scope: **0**.

Code before the test module (`^#\[cfg\((all\()?test`, the form that caught
`persist/mod.rs` out last cluster): `ui/status.rs` 323, `feedback/flash.rs`
291, `tween/mod.rs` 261, `helpers/wasd.rs` 234, `ui/touchpad.rs` 232,
`feedback/screen_flash.rs` 207; every other file under 200. All below the 404
of `modding/events.rs`, which the previous cluster measured and KEPT.

## Steps

- [x] Re-run the two baseline measurements on the work branch and record them in `NOTES.md`; correct this section if they differ. (106 blocks and 0 HUIDs both reproduced exactly.)
- [x] Read every file in scope end to end; table every one of the 106 blocks in `NOTES.md` with a keep / compact / drop / promote call and a one-line reason.
- [x] Drop code-restating and task-narration comments.
- [x] Compact each kept comment to one tagged line (`NOTE:` / `FIXME:` / `BUG:` / `TODO:`), HUID only when it points at a live task record. (37 kept and tagged after review round 1 added `src/material.rs`; 0 carry a HUID.)
- [x] Inside `#[cfg(test)]`: what the test PROVES becomes a `///` on the test fn; what guards a VALUE in the body stays a tagged `NOTE:`. Do not use `///` to move an untagged body comment out of the checker's reach (`tasks/20260731-172232` review round 1).
- [x] Audit rustdoc (`//!`, `///`) for stale claims against the code; fix what is wrong, leave style alone. Includes `ui/status.rs`'s exclusive-system performance contract -- verify, do not trim. (Contract verified and moved to the module doc; four rustdoc warnings fixed; one draft claim about `color_fn` refuted and corrected.)
- [x] Apply the inherited split test to `ui/status.rs`, `feedback/flash.rs`, `tween/mod.rs`: more than one concern AND disjoint dependency sets on both sides of the cut. Record the call, including every KEEP, in `NOTES.md`. (All KEEP; `ui/touchpad.rs` recorded as the closest call.)
- [x] Run the full verification suite below.

## Definition of Done

- Every kept non-doc comment in scope is a tagged block; base has 106 untagged BLOCKS across 21 files (cmd: `./scripts/check-comment-tags.sh src/feedback src/tween src/ui src/transform src/audio src/health src/helpers src/input src/scoring src/time src/material.rs` exits 0, and `./scripts/check-comment-tags.sh src bevy_common_systems_macros/src` -- the epic-wide gate -- exits 0 too).
- No non-doc comment in scope carries a bare tatr HUID; base already has zero, so this is a no-regression gate (cmd: `grep -rnE '^\s*//([^/!]|$)' src/feedback src/tween src/ui src/transform src/audio src/health src/helpers src/input src/scoring src/time src/material.rs | grep -E '20[0-9]{6}-[0-9]{6}' | grep -vE 'NOTE:|FIXME:|BUG:|TODO:'` prints nothing).
- The four pre-existing in-scope rustdoc warnings are gone: `pointer` module-vs-primitive ambiguity at `helpers/mod.rs:5` and `input/mod.rs:4`, unresolved link to `UnifiedPointerPlugin` at `helpers/pointer.rs:17`, redundant explicit link target at `helpers/pointer.rs:16` (cmd: `nix develop --command cargo doc --no-deps --features debug 2>&1 | grep -c '^warning: .*lib doc'` -> 0).
- `ui/status.rs`'s exclusive-system performance contract is verified against the code, not trimmed on sight (manual: read `tasks/20260731-172233/NOTES.md`).
- `NOTES.md` records a keep/compact/drop/promote call for all 106 comment blocks plus per-file code-before-tests numbers behind each split-or-keep decision, including every file over 200 code lines (manual: read `tasks/20260731-172233/NOTES.md`).
- Task artifacts lint clean (cmd: `tatr check`). The ledger form (`tatr check --ledger LESSONS.md`) is clean through review and reports exactly one `promotion-awaiting-decision` after the retro, because `state-what-the-checker-cannot-see` reached x3 and its disposition is the user's call in `/lessons`, not this task's.
- Formatting clean (cmd: `nix develop --command cargo fmt --check`).
- Lints clean in both feature configurations (cmd: `nix develop --command cargo clippy --all-targets` and `nix develop --command cargo clippy --all-targets --features debug`).
- Tests pass in both feature configurations and for examples (cmd: `nix develop --command cargo test`, `... cargo test --features debug`, `... cargo test --examples`).
- Plain-ASCII rule holds (cmd: `./scripts/check-ascii.sh`).
- Public API unchanged: no item renamed, removed, or moved out of its prelude (manual: `git diff master -- src/feedback src/tween src/ui src/transform src/audio src/health src/helpers src/input src/scoring src/time` shows no `pub` signature or prelude re-export line changed).

## Close-out

**What / why.** Comment hygiene and a structure check over the ten module
directories no other epic child claimed. 106 untagged non-doc comment blocks in
21 files -> 0, plus the one block in `src/material.rs` that review round 1
showed no child had claimed: 37 kept and compacted to tagged `NOTE:` blocks,
the rest dropped as code restatement or promoted to rustdoc. Every
call is tabled in `NOTES.md`. Four pre-existing rustdoc warnings in
`helpers/`+`input/` (flagged as this cluster's by 20260731-172232) are fixed.
No public item renamed, removed or moved: `git diff -U0 -- src` changes not one
line containing `pub` or `prelude`.

**Alternatives.** All eight files over 180 code lines were measured against the
inherited split test (more than one concern AND disjoint dependency sets) and
all were KEPT -- the largest, `ui/status.rs` at 328, is below every keep
precedent in the epic (`modding/events.rs` 404, `camera/shake.rs` 313) and far
below the one split (`mesh/builder.rs` 521). `ui/touchpad.rs` is recorded in
`NOTES.md` as the closest call: it does hold two dependency-disjoint concerns,
and is kept on size alone with the cut located for later. Also considered and
rejected: promoting the `tween/mod.rs` P100 provenance to rustdoc (it is task
history, and the `///` below it already states the race), and simplifying
`helpers/temp.rs`'s no-op `(x,).chain()` (out of remit for a comment pass).

**Difficulties / diagnosis.** One real catch: the first draft of the new
`ui/status.rs` module doc said both `value_fn` and `color_fn` take `&World`.
Grepping the actual signatures refuted it -- `color_fn` is
`Fn(Box<&dyn Any>) -> Option<Color>` and runs in the ordinary parallel
`update_status_bar_item_ui`; only `value_fn` is exclusive. Corrected before
commit. The task's stated performance contract itself held (verified:
`update_status_bar_item_values(world: &mut World)`, added to `Update`), but was
documented only in a loose comment copied from Bevy's own docs and was absent
from the module doc where a caller would read it -- so it moved there.

**Evidence.** `./scripts/check-comment-tags.sh <ten dirs> src/material.rs` exit
0 (base: exit 1, 106 blocks), and the epic-wide
`./scripts/check-comment-tags.sh src bevy_common_systems_macros/src` exit 0
(base: exit 1, the `material.rs` block). `tatr check --ledger LESSONS.md` 0.
HUID grep silent. `cargo fmt --check` 0. `cargo clippy
--all-targets` 0 and `--features debug` 0. `cargo test` 0, `cargo test
--features debug` 0, `cargo test --examples` 0 (0 failures in all three).
`cargo doc --no-deps --features debug` 0 with no lib-doc warning line.
`./scripts/check-ascii.sh` 0. `tatr check` 0 (the `--ledger` form reports one
expected `promotion-awaiting-decision`, resolved by `/lessons`).

**Reflection.** The `record-numbers-from-a-rerun` guard paid twice: the
baseline was re-run on the branch before anything was written down, and the
unit (blocks, not lines) was named so the Story's differing numbers did not
have to be reconciled by guessing. The near-miss was in the opposite direction
from last cluster's -- not a measurement, but a plausible-sounding sentence
about code I had just read, written into a module doc where it would have
mislead every future caller. Reading the signature took ten seconds.
