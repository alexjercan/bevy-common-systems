# Re-sizing the `cargo test` peak-RAM fix

Design/fix record for task 20260731-210044. Supersedes the peak figures in
`tasks/20260703-000003/NOTES.md`, which stay valid only for the repo as it stood
on 2026-07-03.

## What was actually wrong

Not a misconfiguration, and not a regression in the 2026-07-03 fix. That fix is
still in the tree and still doing its job. It was **outgrown**.

`cargo test` links one full Bevy 0.19 + avian3d binary per target, and rust-lld
holds each image in memory while linking. Peak RAM is therefore roughly

    (targets linked concurrently) x (size of one linked binary)

The 2026-07-03 work attacked the right-hand factor and got it from ~1.5 GB to
~300 MB. It explicitly rejected capping the left-hand factor:

> Capping cargo's job count during linking: would lower the peak but slows every
> build and does not address the real cost (embedded DWARF). Rejected.

That was correct **at 6 examples and 12 doctests**. Since then:

| Term | 2026-07-03 | 2026-07-31 |
| --- | --- | --- |
| examples | 6 | 15 |
| doctests | 12 | 60 |
| link jobs in flight | up to 24 (`nproc`) | up to 24 (`nproc`) |

A 2.5x growth in example targets and 5x in doctests is not something a
per-binary saving can absorb. Concurrency became the dominant factor, so the
rejected knob is now the necessary one. Recorded as a reversal in the old
NOTES.md rather than silently contradicted.

## The part that was never covered at all

The doctest phase is a second, separate blow-up, and `[profile.dev]` does not
reach it.

Evidence, from `cargo test --doc -v`: the rustdoc invocation cargo builds
carries `--crate-name`, `--extern`, `-L`, `--check-cfg` and friends, and
**no `-C debuginfo` and no `-C split-debuginfo`**. Cargo does not forward
profile debuginfo settings to rustdoc, so neither `split-debuginfo = "unpacked"`
nor `debug = "line-tables-only"` applies to a single one of the 60 doctest
binaries.

They are 60 separate binaries because the crate is `edition = "2021"`. Rustdoc's
merged doctests -- all of a crate's doctests compiled and linked as one binary
-- are an edition 2024 feature. Confirmed count:

    $ nix develop --command cargo test --doc -- --list | tail -1
    60 tests, 0 benchmarks

Nor does `[build] jobs` reach them: cargo passes rustdoc no job limit, so
rustdoc's harness runs its own compile-and-link at one per core. Capping that
phase needs `RUST_TEST_THREADS`, which is why the fix touches `flake.nix` and
not only the two Cargo files.

At 12 doctests this term was noise. At 60 it is the newly dominant one.

## Not the linker's fault

rust-lld is the nightly default for `x86_64-unknown-linux-gnu`; nothing in this
repo selects it.

    $ rustc -vV | head -1
    rustc 1.98.0-nightly (bc2112ed5 2026-06-18)
    $ rustc --print link-args ... | tr ' ' '\n' | grep -i lld
    "-fuse-ld=lld"

So seeing `rust-lld` at the top of btop is expected, not a smoking gun.

## What changed

Three files, three factors, all of which must hold together:

| File | Setting | Factor it caps |
| --- | --- | --- |
| `flake.nix` | `CARGO_BUILD_JOBS` (derived) | lib/example link concurrency |
| `flake.nix` | `RUST_TEST_THREADS` (derived) | doctest link concurrency |
| `Cargo.toml` | `[profile.dev.package."*"] debug = false` | per-binary size |

Plus `scripts/sample-peak-rss.sh`, so the next person measures instead of
guessing -- see below.

### Why the caps are derived rather than committed constants

The obvious home for a job cap is `[build] jobs` in `.cargo/config.toml`, and
that is where this fix first put it. It is the wrong home, for two reasons that
only show up when you look at who else builds this repo:

- `.github/workflows/ci.yml` runs bare `cargo` on `ubuntu-latest` -- 4 cores,
  16 GB. That box needs no cap at all; its default is already 4. Committing
  `jobs = 6` to `.cargo/config.toml` would have RAISED parallelism there, on
  the machine with the least RAM per core. A memory fix that makes the smaller
  machine worse is not a fix.
- `.github/workflows/pages.yml` builds inside `nix develop`, so the devshell's
  environment reaches CI too. A hardcoded constant in `flake.nix` has the same
  problem as one in `.cargo/config.toml`, just for one workflow instead of the
  other.

So the cap is computed per machine in the devshell's `shellHook`:

    min(nproc, MemTotalGB / 4)

which is 7 on this 24-core/31 GB desktop and 4 on a 4-core/16 GB runner -- a
real cap where one is needed, a no-op where one is not. The divisor is the
measured per-link headroom, not a guess: see the peak-per-job figures below.
Both variables respect a value the caller already set, so a one-off
`CARGO_BUILD_JOBS=24 cargo test` still works for measuring the uncapped case.
The block is skipped where `/proc/meminfo` is absent, leaving the darwin
systems this flake declares alone.

## Measurements

Sampling method: `./scripts/sample-peak-rss.sh` samples the summed RSS of all
`cargo`/`rustc`/`rustdoc`/`rust-lld` processes once a second and reports the
peak plus the largest single `rust-lld`. It preserves the wrapped command's exit
code, and `-m LIMIT` runs the command in a memory-capped systemd user scope so a
measurement of a bad configuration cannot take the desktop down with it.

The sample is system-wide, so a build in another checkout would pollute it. All
runs below were taken with the box otherwise idle (zero foreign
cargo/rustc/rust-lld processes), on 31 GB RAM / 24 cores, from a clean target
directory in the sprout worktree.

Full `cargo test`, clean target, caps in force:

    $ ./scripts/sample-peak-rss.sh -m 24G -- nix develop --command cargo test
    test result: ok. 147 passed; 0 failed; 0 ignored
    test result: ok. 59 passed; 0 failed; 1 ignored
    peak-rss: toolchain total 13.1 GB, largest rust-lld 2.8 GB (MemoryMax=24G, command exit 0)

13.1 GB peak on a 31 GB box, no swap thrash, exit 0. Largest single rust-lld was
2.8 GB, which is where the `MemTotalGB / 4` divisor in the shellHook comes from:
roughly 4 GB of headroom per concurrent link, with margin.

Linked binary sizes under the new profile, for comparison with the 2026-07-03
figures (~300 MB lib test binary at 6 examples):

| Artifact | Size |
| --- | --- |
| lib test binary | 231 MB (was ~355 MB before `package."*" debug = false`) |
| each example binary | 483 MB (examples also link clap + bevy_asset_loader) |

The `.dwo` count in `target/debug/deps` (512 files) confirms
`split-debuginfo = "unpacked"` is still doing its job -- the DWARF is beside the
objects, not inside the linked images.

### Not measured

The uncapped baseline (`CARGO_BUILD_JOBS=24 RUST_TEST_THREADS=24`) was NOT
re-measured for this record. Two honest reasons: reproducing it is exactly the
failure being fixed, so it thrashes the box it runs on; and the machine was busy
with unrelated work for the whole window, which would have polluted a
system-wide sample anyway. The 13.1 GB figure above therefore stands on its own
as an absolute "fits comfortably", not as a measured ratio against a before.

If someone wants the ratio, the clean way is a warm target plus
`touch src/lib.rs` (relinks all 16 binaries and rebuilds the 60 doctests without
recompiling the ~400 dependency crates), run twice with the two settings, on an
idle box, under `-m` so the bad case gets OOM-killed in its own scope instead of
taking the desktop with it.

## Why `[profile.dev.package."*"] debug = false` is safe here

Cargo's `package."*"` override applies to dependencies, not to workspace
members, so `src/` keeps the `[profile.dev]` `line-tables-only` setting and
first-party panic backtraces keep their file and line. Bevy and avian frames
lose theirs, which is not a trade this crate ever notices -- it debugs at
runtime through bevy-inspector-egui.

Shown rather than assumed. A `#[test] fn ... { panic!() }` was appended to
`src/lib.rs`, run under `RUST_BACKTRACE=1`, and reverted:

    2: bevy_common_systems::backtrace_probe::deliberate_panic_for_backtrace_check
                 at ./src/lib.rs:50:9

File and line intact for first-party frames with the dependency override in
place. Note that checking the linked binary's `.debug_line` section instead
would have proved nothing either way -- `split-debuginfo = "unpacked"` keeps
DWARF in the `.dwo` files, so the section is empty by design and an absence
there is not an absence of line info. The runtime backtrace is the honest probe.

## Deferred, deliberately

Migrating to edition 2024 would let rustdoc merge the 60 doctests into one
binary, deleting the doctest term outright rather than throttling it. That is
the largest available win and it is seeded as task 20260731-210413. It was kept
out of this change because it rewrites the whole crate and all 15 examples,
while the three settings here are one line each and reversible.

Also considered and rejected:

- **mold.** Not in the devshell, and not an obvious win: mold is tuned for
  link speed via aggressive parallel mmap, and its peak RSS is not reliably
  below lld's. It would not touch the concurrency factor that is the actual
  cause. Would need benchmarking before it could be justified.
- **`-Wl,--no-keep-memory` / `--reduce-memory-overheads`.** GNU bfd ld flags.
  lld does not implement them. Not applicable.
- **`[profile.dev.package."*"] opt-level = 3`**, Bevy's usual recommendation.
  Runtime framerate tuning, orthogonal to link memory, and it would make builds
  slower. nova-protocol sets it because it is a game; this crate is a library.
- **`debug = 0` globally.** Buys a few GB at the cost of test backtraces
  pointing at nothing. The dependency-scoped override gets most of the saving
  and keeps them.
