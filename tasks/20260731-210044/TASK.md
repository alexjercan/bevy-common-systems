# Re-size the cargo test peak-RAM fix for 15 examples / 60 doctests

- PRIORITY: 2
- TAGS: build, memory, toolchain
- KIND: TASK
- ACTIVITY: COMPOUNDING
- GATES: PLAN REVIEW RETRO
- RESOLUTION: DONE

## Problem

`cargo test` exhausts RAM on the 31 GB / 24-core dev box; `rust-lld` is the
process holding it. The 2026-07-03 fix (`tasks/20260703-000003`) is still in
place and still correct -- but it was measured at 6 examples / 12 doctests and
the workload has since grown to 15 examples / 60 doctests.

Two additive terms, both scaling with `nproc = 24`:

- Build phase: 1 lib-test binary + 15 example binaries, each statically linking
  Bevy 0.19 + avian3d. Current linked size ~355 MB each
  (`target/debug/deps/bevy_common_systems-*`). Cargo links up to 24 at once.
- Doctest phase, NOT covered by the existing fix: `cargo test --doc -- --list`
  reports 60 tests. Edition is 2021, so rustdoc's merged doctests do not apply
  and each doctest links its own full-Bevy binary. `cargo test --doc -v` shows
  cargo passes rustdoc no `-C debuginfo` and no `-C split-debuginfo`, so
  `[profile.dev]` never reaches them.

Linker is rust-lld by nightly default (`rustc --print link-args` -> `-fuse-ld=lld`);
nothing in the repo configures it. Not a misconfiguration.

Corroboration: nova-protocol `tasks/20260720-000609/TASK.md:91` already records
"full bcs suite NOT run - it OOMs the box, see bcs-no-full-test-suite" on
2026-07-20. That lesson slug exists in neither repo's LESSONS.md.

## Scope

Cap the concurrency multiplier and cut the remaining per-binary debuginfo, then
refresh the stale record. Deliberately NOT in scope: migrating to edition 2024
for merged doctests (the biggest structural win -- deletes the doctest term
outright -- but a repo-wide change; seeded as its own task).

## Steps

0. `scripts/sample-peak-rss.sh`: wrap a command, sample summed RSS of
   cargo/rustc/rustdoc/rust-lld once a second, report the peak and the command's
   real exit code. The 2026-07-03 fix was measured ad hoc, which is why nobody
   noticed the workload outgrowing it; make the measurement re-runnable.
   Establish the BEFORE numbers with it on a clean target.
1. `.cargo/config.toml`: set `[build] jobs` to cap parallel link jobs.
2. `flake.nix`: set `RUST_TEST_THREADS` in the devshell to cap rustdoc's
   doctest harness, which `[build] jobs` does not reach.
3. `Cargo.toml`: add `[profile.dev.package."*"] debug = false` -- dependency
   line tables are the bulk of what survives `line-tables-only`, and Bevy/avian
   backtraces are never read here. First-party line tables stay.
4. Refresh the stale numbers: the comment block in `Cargo.toml` and
   `tasks/20260703-000003/NOTES.md` both present ~16.5 GB as current. Annotate
   with the measured 2026-07-31 peak and the workload it was sized against.
5. Seed a follow-up task for the edition-2024 merged-doctests migration.
6. Close the dangling `bcs-no-full-test-suite` reference (ledger entry).

## Definition of Done

- A full clean-target `cargo test` completes without exhausting RAM, and the sampled peak summed RSS of all cargo/rustc/rustdoc/rust-lld processes is under 16 GB on this 31 GB box (cmd: `./scripts/sample-peak-rss.sh -- nix develop --command cargo test`, redirected to a file, exit 0 checked).
- The doctest phase is measured separately, since `[build] jobs` does not reach rustdoc's harness (cmd: `./scripts/sample-peak-rss.sh -- nix develop --command cargo test --doc`).
- No test is lost to the concurrency caps: the doctest count is unchanged at 60 (cmd: `nix develop --command cargo test --doc -- --list | tail -1` -> `60 tests, 0 benchmarks`).
- First-party panic backtraces still carry file/line after `[profile.dev.package."*"] debug = false`, i.e. the dependency-only scope of the override is shown, not assumed (manual: a deliberately panicking unit test's backtrace names a `src/` file and line; recorded in `NOTES.md`).
- Tests pass in both feature configurations and for examples (cmd: `nix develop --command cargo test`, `... cargo test --features debug`, `... cargo test --examples`).
- Formatting clean (cmd: `nix develop --command cargo fmt --check`).
- Plain-ASCII rule holds (cmd: `./scripts/check-ascii.sh`).
- Task artifacts and ledger lint clean (cmd: `tatr check --ledger LESSONS.md`).
- No stale peak-RAM figure survives: every occurrence of the 2026-07-03 numbers carries the workload it was measured at (cmd: `grep -rn '16\.5 GB\|19\.7 GB\|38\.3 GB' Cargo.toml tasks/ docs/` -- each hit sits in a block naming its example/doctest counts).
- `NOTES.md` records before/after peaks and the sampling method, so the next growth spurt is re-measured the same way (manual: read `tasks/20260731-210044/NOTES.md`).
- The edition-2024 merged-doctests follow-up exists as its own task, and the dangling `bcs-no-full-test-suite` slug referenced by nova-protocol resolves to a real ledger entry (manual: `tatr ls -f` shows the new task; `grep -n 'bcs-no-full-test-suite' LESSONS.md` hits).

## Close-out

**What and why.** The 2026-07-03 peak-RAM fix was sized against 6 examples /
12 doctests and the workload had grown to 15 / 60. Two settings were added --
`[profile.dev.package."*"] debug = false` to shrink each linked binary, and a
derived `CARGO_BUILD_JOBS` / `RUST_TEST_THREADS` cap in `flake.nix` to bound how
many link at once -- plus `scripts/sample-peak-rss.sh` so the measurement is
re-runnable instead of ad hoc. `RUST_TEST_THREADS` is not redundant with
`CARGO_BUILD_JOBS`: cargo passes no job limit to rustdoc's harness, so it is the
only lever on the 60 doctest links, which are the single largest term.

**The cap divisor was wrong and review caught it.** Round 1 flagged that three
of the four DoD configurations were never measured. Measuring them falsified the
fix rather than merely documenting it: at the shipped cap of 7,
`cargo test --doc` alone peaked at 16.4 GB and `--features debug` at 18.4 GB,
both over the 16 GB target. The divisor moved `MemTotalGB / 4` -> `/ 6` (cap
7 -> 5) and all four configurations now measure 11.6 / 10.6 / 13.5 / 9.9 GB.

**Alternatives.** `.cargo/config.toml` for the cap was the planned Step 1 and
was rejected -- it would also hit `ci.yml`, which runs bare cargo on a small
runner that needs no cap. `mold` was rejected (a link-speed tool; peak RSS is
not reliably lower and it does not touch concurrency). Global `debug = 0` was
rejected for `package."*"`, which keeps first-party backtraces. Edition 2024
merged doctests would delete the doctest term outright and is the largest
available win, deferred as its own task 20260731-210413.

**Difficulties.** The headline 13.1 GB figure from the first implementation pass
could not be reproduced and was withdrawn: it cannot bound the suite when a
strict subset costs 16.4 GB at the same cap. Cached binaries measure nothing, so
every run forces a relink with `touch src/lib.rs` against a warm target. The
sampler is system-wide and this box is shared with another Claude session, so
runs take `flock /home/alex/.claude/shared/heavy-build.lock`.

**Evidence.** Four sampled configurations in `NOTES.md`, all exit 0; doctest
count unchanged at 60; `cargo fmt --check`, `./scripts/check-ascii.sh`,
`tatr check --ledger LESSONS.md` all exit 0; `nix develop` derives jobs=5
threads=5 as designed.

**Reflection.** A cap sized against the default feature set is not sized. The
heaviest configuration (`--features debug`, which links egui into every binary)
is the one that had to be measured, and it was the one skipped. Measure the
worst case or the number is decoration.
