# Taming `cargo test` peak memory

> SUPERSEDED IN PART, 2026-07-31. Everything below is correct **for the repo as
> it stood at 6 examples and 12 doctests**. The fix was never wrong, it was
> outgrown: at 15 examples and 60 doctests the same mechanism exhausts RAM
> again, and the doctest phase turned out never to have been covered by this
> profile at all. Treat the peak figures here as historical. Current analysis,
> numbers and the added concurrency caps: `tasks/20260731-210044/NOTES.md`.

## Symptom

Running the unit tests (`cargo test`) consumed almost the entire machine's RAM
and pushed it into swap. On a 32 GB box the toolchain peaked at ~38 GB, so the
system thrashed and became unresponsive during the test build.

## Investigation

The unit tests themselves are trivial: 20 pure-math/geometry tests (sphere
conversions, PD torque, point rotation, mesh explode) plus 12 doctests. None of
them allocate meaningfully at runtime, so the RAM spike is not the tests
running -- it is the *build and link* step that `cargo test` performs first.

Measured on this repo (peak = summed RSS of all rustc/cargo/rust-lld processes,
sampled once a second during a forced rebuild of the test binaries):

- `cargo test` builds one binary per target: the lib unittest binary plus one
  binary per example (`examples/NN_name.rs`, 6 of them) plus the doctests.
- Each of those binaries statically links the whole Bevy 0.19 + avian3d 0.7
  engine. With the default dev profile (`debug = true`, debuginfo embedded in
  the executable) each linked binary is ~1.5 GB, almost all of it DWARF.
- The host has 24 cores, so cargo runs up to 24 codegen/link jobs in parallel.
  Several `rust-lld` processes each hold a multi-GB binary in memory at once.

Result: the largest single process at peak was always `rust-lld`, and the summed
peak was ~38 GB -- larger than physical RAM, hence the swap thrash.

## Fix

Added a `[profile.dev]` section to `Cargo.toml`:

```toml
[profile.dev]
split-debuginfo = "unpacked"
debug = "line-tables-only"
```

Both knobs attack the same 1.4 GB of DWARF per binary:

- `split-debuginfo = "unpacked"` leaves the DWARF in the per-object `.o` files
  and has the executable reference them, instead of copying it all through the
  linker into the output binary. Full debug info is retained.
- `debug = "line-tables-only"` keeps file/line info (so panic backtraces in
  tests still point at source lines) but drops local-variable info. This crate
  debugs at runtime through `bevy-inspector-egui` (the `debug` feature), not
  through gdb/lldb, so losing DWARF locals costs nothing in practice here.

## Measured impact (same machine, same sampling method)

SUPERSEDED as a current figure by `tasks/20260731-210044/NOTES.md`. The numbers
below are still the correct record of what the profile knobs bought, but they
were measured at **6 examples / 12 doctests** and at cargo's default job count.
The workload is now 15 examples / 60 doctests, and concurrency -- not
per-binary size -- is the dominant term. Do not quote these as today's peak.

| Config                                          | Peak toolchain RAM | Linked binary |
| ----------------------------------------------- | ------------------ | ------------- |
| baseline (`debug = true`, embedded)             | ~38.3 GB (swaps)   | 1.5 GB        |
| `split-debuginfo = "unpacked"`                  | ~19.7 GB           | 347 MB        |
| `+ debug = "line-tables-only"` (committed)      | ~16.5 GB           | 300 MB        |

At 6 examples / 12 doctests the committed config kept the peak comfortably
under half of physical RAM. It no longer does at 15 / 60 without a job cap.

## Alternatives considered

- `debug = false` / `debug = 0`: smaller still, but loses backtraces too. Not
  worth the extra few GB when line-tables-only keeps failure diagnostics.
- Capping cargo's job count during linking: would lower the peak but slows every
  build and does not address the real cost (embedded DWARF). Rejected.
  **Reversed 2026-07-31 (task 20260731-210044).** The reasoning held only while
  per-binary size was the dominant term. Once the target count grew 2.5x and the
  doctest count 5x, concurrency became the multiplier that no per-binary saving
  could offset, and the cap went in as `[build] jobs` plus `RUST_TEST_THREADS`.
- `cargo test --lib` (skip examples): sidesteps the heavy example binaries, but
  the examples are the crate's integration tests and CI builds them, so the
  profile fix (which helps examples, tests and `cargo run` alike) is better.

## To restore full local-variable debugging

Set `debug = true` in `[profile.dev]` (keep `split-debuginfo = "unpacked"`).
That landed around ~19.7 GB peak at 6 examples / 12 doctests -- under physical
RAM, no swap. At today's 15 examples / 60 doctests it would not be; re-measure
with `./scripts/sample-peak-rss.sh` before trusting it.
