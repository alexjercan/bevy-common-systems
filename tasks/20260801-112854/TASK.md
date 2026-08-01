# Release 0.19.6: changelog, version bump, tag, push

- STATUS: IN_PROGRESS
- PRIORITY: 90
- TAGS: chore, release
- KIND: TASK
- FLOW STEP: WORKING
- PLAN STATUS: APPROVED

## Context

User goal 2026-08-01: cut release 0.19.6, update the changelog, tag, push.
CI is watched by the user, not by this session.

`v0.19.5` (30d1bef, 2026-07-20) is the last tag. Everything since is
maintenance: the v0.19.x KISS epic (20260731-172116) comment/structure pass
over the library, three new `scripts/` checkers, the `docs/` retirement, and
the `nix develop` peak-RAM caps.

Surveyed the whole `v0.19.5..HEAD` src diff for behavior: outside the two
module splits there are 46 non-comment changed lines, all of them tests, a
`manual_contains` clippy fix, and a `let mut axis` -> shadowed `let axis`.
No public item was added, removed or re-signed
(`git diff v0.19.5..HEAD -- src/ bevy_common_systems_macros/ | grep '^[-+].*\bpub\b'`
returns only `pub(super)` items inside the two new private modules). So this
is a no-API-change release; the changelog must say so rather than imply
features.

Blocker found while checking the gate: `tatr check --ledger LESSONS.md` exits
1 on `dangling-promotion-task: record-numbers-from-a-rerun: task
'20260801-102152' does not exist`. That task was created by d47b88b and
deleted by the user in 2e82313 ("docs: remove task"). The ledger entry still
points at it. Resolve as part of the pre-release `lessons` fold.

## Steps

1. ~~Fix the ledger~~ DROPPED, escalated to the user. Every reachable state of
   the `record-numbers-from-a-rerun` entry needs a DISPOSITION, and
   `~/.claude/skills/lessons/ledger.md` says only the user picks one ("Never
   compose disposition annotations by hand"; record via `tatr ledger`).
   Probed three spellings against the linter: no annotation ->
   `promotion-awaiting-decision`; a prose word -> `bad-disposition ... is not
   PROMOTE|DEFER|RETIRE|ABSORBED`; `DEFER <date>` -> `bad-disposition: DEFER
   needs 'at x<count>'`. Left byte-identical to `master`, which was ALREADY
   failing this gate before this task (`tatr check --ledger` at b3dc1e6 exits
   1 on `dangling-promotion-task`). Not a release blocker: it gates task
   artifacts, not the crate.
2. Bump `version` to `0.19.6` in `Cargo.toml` and
   `bevy_common_systems_macros/Cargo.toml`; refresh `Cargo.lock` via a build.
3. Add a `## [0.19.6] - 2026-08-01` CHANGELOG section under `[Unreleased]`,
   honest about the no-API-change scope: Added (the three `scripts/`
   checkers), Changed (library comment/structure pass with the two private
   splits named, `nix develop` RAM caps), Removed (`docs/`).
4. Repair the CHANGELOG link refs, stale since 0.19.1: `[unreleased]` must
   compare from `v0.19.6`, and `0.19.2`..`0.19.6` rows are missing entirely.
5. Run the full local CI suite from AGENTS.md "Build, Verify, Run".
6. Land on `master`, tag `v0.19.6`, push branch + tag. Do not watch CI.

## Definition of Done

- [x] Both manifests read `0.19.6` and `Cargo.lock` agrees
      (cmd: `grep -c '^version = "0.19.6"' Cargo.toml bevy_common_systems_macros/Cargo.toml`;
      cmd: `grep -A1 'name = "bevy_common_systems"' Cargo.lock`)
- [x] CHANGELOG has a 0.19.6 section and every version 0.19.0..0.19.6 has a
      link ref (cmd: `for v in 0.19.0 0.19.1 0.19.2 0.19.3 0.19.4 0.19.5 0.19.6; do grep -q "^\[$v\]:" CHANGELOG.md || echo "MISSING $v"; done`)
- [x] Tracker clean; ledger NOT clean and deliberately unchanged, see Step 1
      (cmd: `tatr check`)
- [x] Local CI suite green
      (cmd: `cargo fmt --check`)
      (cmd: `cargo clippy --all-targets`)
      (cmd: `cargo clippy --all-targets --features debug`)
      (cmd: `cargo test`)
      (cmd: `cargo test --features debug`)
      (cmd: `cargo test --examples`)
      (cmd: `./scripts/check-ascii.sh`)
- [x] ASCII rule holds for the edited docs
      (cmd: `grep -nP '[^\x00-\x7F]' CHANGELOG.md tasks/20260801-112854/TASK.md`
      finds nothing)
- [x] Tag `v0.19.6` exists on the release commit and both are pushed
      (cmd: `git ls-remote --tags origin v0.19.6`)

## Notes

- Version scheme tracks Bevy's minor, not semver: `0.19.x` targets Bevy
  `0.19.x`. A patch bump with no API change is normal here.
- `cargo build` alone does not compile examples; the suite's
  `clippy --all-targets` is what proves the tree.
- Peak RAM: `nix develop` already caps `CARGO_BUILD_JOBS` /
  `RUST_TEST_THREADS`; run the suite inside the devshell, not bare.

## Close-out

**What / why.** `0.19.6`, a maintenance release: two manifest bumps, a
regenerated `Cargo.lock`, a CHANGELOG section, and the link-ref block repaired.
The release itself carries no code change -- the content is what landed between
`v0.19.5` and `b3dc1e6`.

**Scope, established before writing the entry.** No public item was added,
removed or re-signed: `git diff v0.19.5..HEAD -- src/
bevy_common_systems_macros/ | grep '^[-+].*\bpub\b'` returns 7 lines, 5 of them
`pub(super)` declarations inside the two NEW private modules and 2 comment text.
Excluding the four files of the two splits, the non-comment src diff is 46
lines: a `manual_contains` clippy fix, a `let mut axis` -> shadowed `let axis`,
a `simulate_seconds` test helper replacing five `for _ in 0..N` loops, one new
test (`attribute_less_derive_defaults_to_no_payload`), and two assertion
messages. So the entry leads with "the public API is unchanged from 0.19.5"
rather than implying features.

**Alternatives.** Considered skipping the release as content-free. Rejected:
rustdoc IS a shipped surface and the whole KISS epic rewrote it, and the new
`scripts/` checkers plus the `docs/` retirement change how a contributor works.
Considered a `### Internal` heading; Keep a Changelog has no such category, so
the no-API-change fact went into a lead paragraph under the version heading and
the body stayed in Added/Changed/Removed.

**Difficulties.** The CHANGELOG link-ref block had been stale since `0.19.1` --
`[unreleased]` still compared from `v0.19.1` and rows for `0.19.2` through
`0.19.5` were never added. Four back-fills plus `0.19.6`, so the file is
self-consistent again rather than only correct going forward.

The ledger gate is the one thing this task did not close, and dropping it was
deliberate rather than an oversight -- see Step 1. It was already red on
`master` before this task started.

**Evidence.** All re-run at the final tree state, inside `nix develop`.
`cargo build` exit 0 (`Cargo.lock` -> `bevy_common_systems` and
`bevy_common_systems_macros` both `version = "0.19.6"`); `cargo fmt --check`,
`./scripts/check-ascii.sh`, `cargo clippy --all-targets` and
`cargo clippy --all-targets --features debug` all exit 0; `cargo test` exit 0
(148 unit + 59 doc tests passed, 1 doc test ignored), `cargo test --features
debug` exit 0 (155 unit + 66 doc, 1 ignored), `cargo test --examples` exit 0
(15 example binaries, 117 tests). Link refs: the 7-version loop prints nothing.
`grep -nP '[^\x00-\x7F]'` over `CHANGELOG.md` and this file exits 1 (no match).
`tatr check` exit 0. Only warning in any run is the known transitive
`proc-macro-error2` future-incompat note.

**Reflection.** Surveying the diff for public-API changes BEFORE drafting the
changelog is what kept the entry honest; drafting from the commit subjects
first would have produced a features-flavoured entry for a release that has
none. The stale link-ref block is the reverse lesson: nothing verifies a
markdown reference-link target, so it rotted through four releases unnoticed.
The 7-version loop is now a DoD proof, which is the cheap fix.
