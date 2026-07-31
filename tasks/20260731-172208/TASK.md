# KISS pass: debug/ + lib.rs + completion.rs (sets the comment convention)

- STATUS: CLOSED
- PRIORITY: 90
- TAGS: chore, kiss, debug
- KIND: STORY
- FLOW STEP: DONE
- PLAN STATUS: APPROVED
- PARENT: 20260731-172116

Scope: `src/debug/` (`inspector.rs`, `wireframe.rs`, `mod.rs`,
`harness/{mod,autopilot,screenshot}.rs`), plus `src/lib.rs` and
`src/completion.rs`. Also ships the shared checker the four sibling clusters
reuse.

Measured on base (2026-07-31): 74 non-doc comment lines forming **37 comment
blocks**, none tagged. Per file: `inspector.rs` 29 lines / 14 blocks,
`wireframe.rs` 4 / 4, `autopilot.rs` 17 / 6, `screenshot.rs` 17 / 11,
`lib.rs` 4 / 1, `completion.rs` 3 / 1; `debug/mod.rs` and `harness/mod.rs` are
already clean.

Load-bearing comments that MUST survive (tagged, not deleted):
`inspector.rs:55` (primary-context retargeting rationale), `inspector.rs:142`
(the egui-cluster removal hazard -- removing the marker alone panics
bevy_egui), `lib.rs:7` (why `completion` is ungated), `completion.rs:110`
(per-registrant watcher is deliberate), `autopilot.rs:198` and `:250`,
`screenshot.rs:127` and `:187`. Pure restatement to delete:
`inspector.rs:35/38/42/63/70/98/101/106`, all four in `wireframe.rs`,
`screenshot.rs:206/278/280/286/297`.

Two non-doc comments carry bare HUIDs (`inspector.rs:60`, `inspector.rs:210`)
and currently fail the epic's HUID proof.

One in-scope stale rustdoc link: `autopilot.rs:51` -> `loop_while_pending`
(unresolved; one of 11 baseline `cargo doc` warnings, the other 10 sit in
other clusters' files).

Note the `debug` feature gate: nothing under `src/debug/` compiles without
`--features debug`, so every check runs in both configurations. `cargo` is
only on PATH inside `nix develop`.

See `DECISION.md` for the tag shape (D1), the shared script (D2), and the
no-split call (D3).

## Steps

- [x] Read every in-scope file end to end; record each of the 37 comment
      blocks in `NOTES.md` with a keep / compact / drop call and one-line
      reason.
- [x] Add `scripts/check-comment-tags.sh`: takes paths, fails listing every
      non-doc comment block whose first line lacks `NOTE:` / `FIXME:` /
      `BUG:` / `TODO:`. Confirm it reports 37 on the unmodified tree before
      editing any source (that is the proof it works).
- [x] Drop the code-restating and test-narration comments listed above.
- [x] Compact each kept comment to one tagged block (first line tagged, per
      DECISION.md D1); move the surrounding prose into `NOTES.md`.
- [x] Re-home the two HUID comments (`inspector.rs:60`, `:210`) as tagged
      `NOTE:` lines that name a live task record, or drop the HUID.
- [x] Fix the `autopilot.rs:51` rustdoc link; scan the rest of the in-scope
      `//!` / `///` for factually stale claims (fix what is wrong, leave style
      alone).
- [x] Record final code-before-tests per file in `NOTES.md` and confirm the
      no-split call (D3) still holds against the measured numbers.
- [x] Add one `AGENTS.md` Conventions bullet: kept inline comments are tagged
      blocks, checked by `scripts/check-comment-tags.sh`.
- [x] Run the full verification suite (all `cmd:` proofs below) under
      `nix develop --command`.

## Definition of Done

- Every kept non-doc comment in scope is a tagged block; base has 37 untagged (cmd: `./scripts/check-comment-tags.sh src/debug src/lib.rs src/completion.rs` exits 0).
- No non-doc comment in scope carries a bare tatr HUID; base has 2 in `inspector.rs` (cmd: `grep -rnE '^\s*//([^/!]|$)' src/debug src/lib.rs src/completion.rs | grep -E '20[0-9]{6}-[0-9]{6}' | grep -vE 'NOTE:|FIXME:|BUG:|TODO:'` prints nothing).
- Rustdoc in scope is warning-free and the crate total does not exceed the 11-warning baseline; base has 1 in-scope at `autopilot.rs:51` (cmd: `nix develop --command cargo doc --no-deps --features debug 2>&1 | grep -cE '^\s+--> src/(debug|lib\.rs|completion\.rs)'` -> 0).
- Formatting clean (cmd: `nix develop --command cargo fmt --check`).
- Lints clean in both feature configurations (cmd: `nix develop --command cargo clippy --all-targets` and `nix develop --command cargo clippy --all-targets --features debug`).
- Tests pass in both feature configurations and for examples (cmd: `nix develop --command cargo test`, `... cargo test --features debug`, `... cargo test --examples`).
- Plain-ASCII rule holds, including the new script (cmd: `./scripts/check-ascii.sh`).
- Task artifacts and ledger lint clean (cmd: `tatr check --ledger LESSONS.md`).
- `NOTES.md` records a keep/compact/drop call for all 37 comment blocks plus the per-file code-before-tests numbers behind the split decision (manual: read `tasks/20260731-172208/NOTES.md`).
- Public API unchanged: no item renamed, removed, or moved out of its prelude (manual: `git diff master -- src/lib.rs` and each in-scope `pub mod prelude` shows no re-export line changed).

## Close-out

**What / why.** 37 untagged non-doc comment blocks (74 lines) across the eight
in-scope files became 0 untagged / 35 lines: 13 kept and tagged, 21 dropped,
3 promoted into rustdoc. Shipped `scripts/check-comment-tags.sh` plus one
`AGENTS.md` Conventions bullet, so the rule the four sibling clusters follow
is runnable rather than remembered. Fixed the one stale in-scope rustdoc link
(`autopilot.rs:51`). No behavior, signature, or public-path change.

**Alternatives.** (1) One-line-only kept comments -- rejected in DECISION.md
D1; the egui-cluster hazard at `inspector.rs:142` cannot compress that far
without losing the mechanism it guards. (2) Prose-only convention -- rejected
D2; four downstream tasks would each re-derive it. (3) Splitting
`autopilot.rs` / `screenshot.rs` -- rejected D3 and re-confirmed against final
measurements in `NOTES.md`; both are single-concern and under the epic's
outlier threshold.

**Difficulties.** The two HUIDs in `inspector.rs` name another repo's tasks,
so the "live task record" carve-out did not apply; moving both into rustdoc
preserved the provenance and cleared the proof. The checker's first cut let a
`///` line continue a `//` run, which would mask an untagged comment beneath a
doc comment; fixed so any non-comment line, rustdoc included, resets the
block. A `///` written after `#[test]` compiles but is not an item doc --
caught by re-reading, not by a tool result.

**Evidence.** Checker 37 -> 0; scoped HUID grep 2 -> 0; in-scope rustdoc
warnings 1 -> 0 and crate total 11 -> 10; fmt/clippy (both feature configs)
clean; tests 59 / 66 / 1 pass, unchanged in both configs, as a comment-only
pass should be; ascii clean. `git diff master` shows no `pub`/`fn`/`impl`/
`const` line added or removed in scope, and the `lib.rs` diff is
comment-only.

**Reflection.** Building the checker before the first edit turned "did I miss
one" from judgement into a number. The most reliable DROP signal was
proximity to rustdoc -- five dropped comments were restating a `///` block
within ten lines; the siblings should read the item's own rustdoc before
calling a comment load-bearing. Promoting provenance to rustdoc instead of
deleting it is the general escape from the HUID rule.

## Notes

- Assumption: the epic's grep proof stays authoritative for HUIDs; the new
  script is the stricter superset the siblings inherit.
- Non-doc comments inside `#[cfg(test)]` modules count -- the convention is
  uniform. Test narration an assertion message already states gets dropped.
- `debug/mod.rs` and `harness/mod.rs` need no comment work; still audit their
  rustdoc.
