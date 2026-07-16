# Taming `cargo test` peak memory

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

| Config                                          | Peak toolchain RAM | Linked binary |
| ----------------------------------------------- | ------------------ | ------------- |
| baseline (`debug = true`, embedded)             | ~38.3 GB (swaps)   | 1.5 GB        |
| `split-debuginfo = "unpacked"`                  | ~19.7 GB           | 347 MB        |
| `+ debug = "line-tables-only"` (committed)      | ~16.5 GB           | 300 MB        |

The committed config keeps the peak comfortably under half of physical RAM.

## Alternatives considered

- `debug = false` / `debug = 0`: smaller still, but loses backtraces too. Not
  worth the extra few GB when line-tables-only keeps failure diagnostics.
- Capping cargo's job count during linking: would lower the peak but slows every
  build and does not address the real cost (embedded DWARF). Rejected.
- `cargo test --lib` (skip examples): sidesteps the heavy example binaries, but
  the examples are the crate's integration tests and CI builds them, so the
  profile fix (which helps examples, tests and `cargo run` alike) is better.

## To restore full local-variable debugging

Set `debug = true` in `[profile.dev]` (keep `split-debuginfo = "unpacked"`).
That lands around ~19.7 GB peak -- still under physical RAM, no swap.
