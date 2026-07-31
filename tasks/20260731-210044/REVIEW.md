# Review: Re-size the cargo test peak-RAM fix for 15 examples / 60 doctests

- TASK: 20260731-210044
- BRANCH: fix/test-peak-ram

## Round 1

- REVIEWER: out-of-context
- VERDICT: REQUEST_CHANGES

Two open MAJORs, both evidence gaps against explicit Definition of Done lines
rather than defects in the code. The implementation verifies clean and the
design record is unusually candid about what it did not measure; it simply
stops short of measurements the task promised.

- [ ] R1.1 (MAJOR) tasks/20260731-210044/NOTES.md - the doctest phase was never
  measured separately, but DoD line 64 requires it
  (`./scripts/sample-peak-rss.sh -- nix develop --command cargo test --doc`).
  NOTES records exactly one run (full `cargo test`, 13.1 GB) and no `--doc`
  figure. This is the phase the entire `RUST_TEST_THREADS` half of the change
  exists to fix, so its effect is unverified. Compounding it, the seeded
  follow-up `tasks/20260731-210413/TASK.md:44` tells the next person to compare
  against "the recorded 2026-07-31 baseline" -- which does not exist, making
  that task's DoD unsatisfiable as written. Run the sampler on `--doc` and
  record the peak plus the `RUST_TEST_THREADS` value in force.
- [ ] R1.2 (MAJOR) tasks/20260731-210044/NOTES.md - no evidence for `cargo test
  --features debug` or `cargo test --examples`, both required by DoD line 67.
  `--features debug` is the HEAVIER configuration (adds bevy-inspector-egui and
  egui to every linked binary), so the "13.1 GB, fits comfortably" conclusion is
  demonstrated only for the lighter one. Record both.
- [ ] R1.3 (MINOR) flake.nix:83 and tasks/20260731-210044/NOTES.md:107 - the
  claim that the cap is "4 on a 4-core/16 GB runner -- a no-op where one is not
  [needed]" is arithmetically false; it is 3 there. `awk '/MemTotal/ {printf
  "%d", $2 / 1048576}'` truncates: a 16 GB runner reports ~16374624 kB = 15.6
  GiB -> 15; 15 / 4 = 3; min(4, 3) = 3. `pages.yml` builds via `nix develop`,
  so it takes a ~25% parallelism cut on the machine the record calls untouched.
  Either state the real value or round instead of truncate, and say which.
- [ ] R1.4 (MINOR) flake.nix:88 - `RUST_TEST_THREADS` is presented as capping
  doctest link concurrency, but being exported shell-wide it also caps test
  EXECUTION parallelism for every libtest harness in the devshell. Cheap here
  (the tests are pure math), but it is an unstated side effect of an overloaded
  knob. Acknowledge it in the flake comment and the NOTES table.
- [ ] R1.5 (MINOR) flake.nix:96 - `CARGO_BUILD_JOBS` is global to the devshell,
  not scoped to linking, so every `cargo build`/`run`/trunk invocation drops
  24 -> 7 jobs, including the ~400-crate cold dependency compile where rustc,
  not rust-lld, is the memory profile and the cap buys nothing. Cargo has no
  separate link-jobs knob, so the tradeoff is forced -- but record the cost and
  the `CARGO_BUILD_JOBS=24` escape hatch where a developer will see it.
- [ ] R1.6 (MINOR) scripts/sample-peak-rss.sh:82 - killing the sampler can
  truncate the peak file and silently report `0.0 GB`. `echo ... > "$peak_file"`
  truncates then writes, and `kill "$sampler_pid"` can land between the two; the
  subsequent `read` then yields empty strings and awk prints `0.0 GB` while
  still exiting with the command's true status -- a wrong number that looks
  like a real measurement. Write atomically (`> "$peak_file.tmp" && mv -f`) or
  refuse to print on a non-numeric read.
- [ ] R1.7 (NIT) scripts/sample-peak-rss.sh:88 - `-m` silently assumes systemd.
  Without `systemd-run` the wrapped command never runs and the output reads
  `command exit 127`, indistinguishable from a real build failure. Add a
  `command -v systemd-run` preflight.
- [ ] R1.8 (NIT) tasks/20260731-210044/TASK.md:5 - still `FLOW STEP: PLANNED` /
  `STATUS: OPEN` on an implemented branch. `tatr check` passes, so bookkeeping
  only.

### Verified clean (no finding)

- `shellcheck scripts/sample-peak-rss.sh` exit 0. Exit-code preservation
  (`status=$?` after the if/else) and quoting are correct.
- The `set -e` concern about `[ "$cap" -lt 1 ] && cap=1` as a function's last
  command is a non-issue: bash exempts a failing left operand of `&&`, and it
  was reproduced under `set -e` inside a command substitution.
- `shellHook` is a valid `mkShell` attribute in that position, and the derived
  value verifies: `nix develop --command bash -c 'echo $CARGO_BUILD_JOBS'` -> 7,
  matching `min(24, floor(32670224 kB / 1048576) / 4)` = 7.
- `[profile.dev.package."*"] debug = false` scope verified empirically, not from
  docs: all 1024 `.dwo` files in `target/debug/deps` belong to
  `bevy_common_systems`; zero dependency `.dwo` files. Dependencies emit no
  debuginfo, first-party still does.
- CI impact: `ci.yml` runs bare cargo with no Nix, so it is genuinely untouched
  and only benefits from the profile change; the `.cargo/config.toml:1` NOTE
  explaining why the cap is NOT committed there is accurate. `pages.yml` is the
  one affected path -- see R1.3.
- The reversal of the 2026-07-03 "rejected: capping jobs" decision is recorded
  in the old NOTES rather than silently contradicted. The deviation from Step 1
  (cap in `.cargo/config.toml`) is deliberate, explained, and better than the
  spec'd version.
- `cargo fmt --check`, `./scripts/check-ascii.sh`, `tatr check --ledger
  LESSONS.md` all exit 0.

### In-session verification by the primary

Re-derived independently rather than accepted:

- **R1.3 arithmetic.** Recomputed the cap for both machine profiles from
  `/proc/meminfo` semantics: this box gives `cores=24 mem_gb=31 cap=7`
  (matching the record); a 4-core/16 GB runner gives `mem_gb=15 cap=3`, NOT the
  4 the record claims. The finding stands.
- **Clippy is not implicated in the RAM problem at all.** Clippy is
  `cargo check`-shaped and produces no linked binaries, so rust-lld is never
  involved; its cost is cold-graph rustc parallelism. Measured under the cap on
  master: `--all-targets` 2s and `--features debug` 18s, both exit 0. This
  retires the session-scoped clippy prohibition, which was aimed at the wrong
  cause.

### Pending user checks

- The 13.1 GB peak, largest-rust-lld 2.8 GB, "box otherwise idle" and "no swap
  thrash" claims: re-running `cargo test` was prohibited for the reviewer.
- The first-party-backtrace probe (DoD line 66) was appended to `src/lib.rs` and
  reverted, so nothing in the tree reproduces it. The reviewer corroborated the
  mechanism via `.dwo` ownership, which is strong but not the same evidence.
- `PLAN STATUS: APPROVED` on line 8 was set by the implementing agent on its own
  plan. That gate is the user's, not the implementer's.
