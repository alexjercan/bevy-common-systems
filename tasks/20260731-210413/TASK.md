# Migrate to edition 2024 so rustdoc merges the 60 doctests into one binary

- STATUS: OPEN
- PRIORITY: 1
- TAGS: build,memory,toolchain
- KIND: TASK
- FLOW STEP: BACKLOG
- PLAN STATUS: DRAFT
- DEPENDS ON: 20260731-210044

## Problem

`cargo test`'s doctest phase links 60 separate full Bevy+avian binaries, one per
doctest. This crate is `edition = "2021"`, and rustdoc's merged doctests -- which
compile and link a crate's doctests as a SINGLE binary -- are enabled only for
edition 2024. Task 20260731-210044 capped the concurrency of those 60 links
(`RUST_TEST_THREADS`) because it is cheap and reversible; migrating the edition
would delete the term outright instead of throttling it, but is a repo-wide
change and was deliberately deferred.

Second, unrelated-looking but same root: `[profile.dev]` does not reach doctests
at all. `cargo test --doc -v` shows cargo passes rustdoc no `-C debuginfo` and
no `-C split-debuginfo`. Merged doctests would make that irrelevant for the peak
(one binary, not 60) without needing cargo to change.

## Scope

Migrate the crate (and `bevy_common_systems_macros`, and the 15 examples) to
edition 2024, then confirm rustdoc actually merges the doctests rather than
assuming the edition bump is sufficient.

## Steps

1. `cargo fix --edition` across the workspace; review every rewrite by hand.
2. Bump `edition` in `Cargo.toml` and `bevy_common_systems_macros/Cargo.toml`.
3. Confirm merging actually happened -- do not infer it from the edition field.
   A merged run reports the doctests as one binary; `./scripts/sample-peak-rss.sh`
   on `cargo test --doc` should drop sharply against the number recorded in
   `tasks/20260731-210044/NOTES.md`.
4. If merging holds, revisit whether `RUST_TEST_THREADS` in `flake.nix` is still
   needed for the doctest phase, or now only for the example link phase.

## Definition of Done

- Crate and macros subcrate are edition 2024 (cmd: `grep -n '^edition' Cargo.toml bevy_common_systems_macros/Cargo.toml`).
- Doctests are actually merged, shown rather than assumed: the `cargo test --doc` peak drops materially against the recorded 2026-07-31 baseline (cmd: `./scripts/sample-peak-rss.sh -- nix develop --command cargo test --doc`, compared against `tasks/20260731-210044/NOTES.md`).
- No doctest is lost in the migration (cmd: `nix develop --command cargo test --doc -- --list | tail -1` -> `60 tests, 0 benchmarks`, or the count as of the migration, stated in NOTES.md).
- Tests pass in both feature configurations and for examples (cmd: `nix develop --command cargo test`, `... cargo test --features debug`, `... cargo test --examples`).
- Formatting, ASCII and task lint clean (cmd: `nix develop --command cargo fmt --check`, `./scripts/check-ascii.sh`, `tatr check --ledger LESSONS.md`).
