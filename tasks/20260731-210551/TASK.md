# Cap link concurrency: cargo test OOMs on rust-lld

- STATUS: OPEN
- PRIORITY: 85
- TAGS: chore,build,memory
- KIND: TASK
- FLOW STEP: BACKLOG
- PLAN STATUS: DRAFT

`cargo test` exhausts system RAM. Observed in btop: `rust-lld` consuming the
box. Blocks the test proofs of every task, not one -- 20260731-172223 closed
its verification with `cargo check --all-targets` because test EXECUTION could
not run.

## Findings so far

Diagnosis from a session investigation, not yet re-measured:

- Nothing in the repo configures a linker. rust-lld is the **nightly default**
  for `x86_64-unknown-linux-gnu` (`rustc --print link-args` shows
  `-fuse-ld=lld`). Not a misconfiguration.
- `Cargo.toml` already carries the 2026-07-03 mitigation
  (`split-debuginfo = "unpacked"` + `debug = "line-tables-only"`,
  `tasks/20260703-000003/NOTES.md`), which took peak RAM ~38.3 GB -> ~16.5 GB.
  **That measurement was taken at 6 examples and 12 doctests.** The tree now
  has 15 examples and 60 doctests, on a 31 GB box with `nproc = 24`.
- Two additive terms:
  - build phase: 1 lib-test binary + 15 example binaries, each statically
    linking all of Bevy 0.19 + avian3d (~354 MB each on disk today), up to 24
    links in parallel;
  - doctest phase, **not covered by the mitigation at all**: `cargo test --doc -v`
    shows rustdoc receiving no `-C debuginfo` and no `-C split-debuginfo` --
    `[profile.dev]` does not reach doctests. Edition is 2021, so rustdoc's
    merged doctests do not apply and it links 60 separate full-Bevy binaries.
- Rejected as fixes: mold (peak RSS not reliably below lld's, and does not
  address concurrency); `-Wl,--no-keep-memory` / `--reduce-memory-overheads`
  (GNU bfd flags, lld does not implement them); `[profile.dev.package."*"]
  opt-level = 3` (runtime tuning, orthogonal); global `debug = 0` (loses
  first-party backtrace lines for little extra gain).

## Steps

- [ ] Re-measure current peak RSS for `cargo test` and `cargo test --doc`
      separately, so the fix is judged against a number this tree produced.
- [ ] Cap build link concurrency via `[build] jobs` in `.cargo/config.toml`.
- [ ] Bound the doctest harness (`[build] jobs` does NOT reach it): set
      `RUST_TEST_THREADS` in `flake.nix` next to `RUST_BACKTRACE`.
- [ ] Evaluate `[profile.dev.package."*"] debug = false` -- dependency debuginfo
      is the bulk of what survives `line-tables-only`, and Bevy/avian backtraces
      are never read here.
- [ ] Update the stale 16.5 GB figure in `Cargo.toml`'s comment and in
      `tasks/20260703-000003/NOTES.md`; both now mislead.
- [ ] Bump `wasm-getrandom-and-build-profile` in `LESSONS.md` with the
      measurement-ages variant (this is its third occurrence, so it moves to
      Pending promotions).
- [ ] Scope edition 2024 as a SEPARATE task if the throttles are not enough:
      merged doctests would collapse 60 links into 1, deleting the doctest term
      outright rather than throttling it.

## Definition of Done

- `cargo test` completes without exhausting RAM on a 31 GB box (cmd: `nix develop --command cargo test`).
- `cargo test --examples` completes without exhausting RAM (cmd: `nix develop --command cargo test --examples`).
- Peak RSS for a full `cargo test` is recorded in `NOTES.md` alongside the pre-fix number (manual: read `tasks/20260731-210551/NOTES.md`).
- No stale peak-RAM figure remains (cmd: `grep -rn '16.5 GB\|38.3 GB' Cargo.toml tasks/20260703-000003/` shows only figures labelled with their example/doctest count).
- Ledger lint clean after the bump (cmd: `tatr check --ledger LESSONS.md`).
