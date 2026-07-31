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

Every run below forced a relink with `touch src/lib.rs` against a warm target,
so all 16 binaries and all 60 doctests actually re-link while the ~400
dependency crates stay cached. A cached-binary run measures nothing. Each was
serialized against the other session on this box with
`flock /home/alex/.claude/shared/heavy-build.lock`, and `--no-fail-fast` keeps
the run alive through the whole link storm.

### The divisor: 4 was wrong, 6 is the measured value

The first cut of this fix used `min(nproc, MemTotalGB / 4)` = 7 here. Measured
at that cap, three of the four configurations bust the 16 GB target:

| Run | cap 7 (`/4`) | cap 5 (`/6`) | largest rust-lld at cap 5 |
| --- | --- | --- | --- |
| `cargo test` | -- | **11.6 GB** | 2.7 GB |
| `cargo test --doc` | 16.4 GB | **10.6 GB** | 2.2 GB |
| `cargo test --features debug` | 18.4 GB | **13.5 GB** | 3.0 GB |
| `cargo test --examples` | 17.2 GB | **9.9 GB** | 2.2 GB |

All four at cap 5 exit 0: 147 lib tests (154 with `debug`), 59 passed + 1
ignored doctests, 117 example tests.

Two things the cap-7 column corrects in the earlier record:

- The original headline was a single `cargo test` run reported as 13.1 GB with
  the cap at 7. That figure cannot be right as a bound on the suite: `--doc`
  alone, a strict subset of `cargo test`, costs 16.4 GB at the same cap. It has
  been dropped rather than reconciled -- it was one unrepeated sample and the
  runs above supersede it.
- The divisor was sized against the DEFAULT feature set. `--features debug`
  links `bevy-inspector-egui` and egui into every binary and is the heaviest
  configuration; at 18.4 GB it was the one over the line, and it was the one
  never measured.

`MemTotalGB / 6` budgets ~6 GB per concurrent link against a measured ~2.7 GB
per job (whole-run peak / cap) and a 3.0 GB largest single `rust-lld`. The ~2x
margin is deliberate: overshooting the peak costs swap, while over-reserving
only costs parallelism.

Linked binary sizes under the new profile, for comparison with the 2026-07-03
figures (~300 MB lib test binary at 6 examples):

| Artifact | Size |
| --- | --- |
| lib test binary | 231 MB (was ~355 MB before `package."*" debug = false`) |
| each example binary | 483 MB (examples also link clap + bevy_asset_loader) |

The `.dwo` count in `target/debug/deps` (512 files) confirms
`split-debuginfo = "unpacked"` is still doing its job -- the DWARF is beside the
objects, not inside the linked images.

### What the caps cost, beyond linking

Both knobs are overloaded; neither is scoped to linking, because cargo has no
link-jobs setting.

| Knob | Intended effect | Also does | Cost here |
| --- | --- | --- | --- |
| `RUST_TEST_THREADS` | caps concurrent doctest LINKS, the only lever that reaches rustdoc's harness | caps test EXECUTION for every libtest harness in the devshell | negligible; these tests are pure math and finish in seconds |
| `CARGO_BUILD_JOBS` | caps concurrent lib/example links | caps the cold ~400-crate dependency compile, where rustc rather than rust-lld is the memory profile | a slower cold build; `CARGO_BUILD_JOBS=24 cargo build` takes the cores back |

### Not measured

The uncapped baseline (`CARGO_BUILD_JOBS=24 RUST_TEST_THREADS=24`) was NOT
measured. Reproducing it is exactly the failure being fixed, so it thrashes the
box it runs on -- and this box is shared with another Claude session whose own
builds would pollute a system-wide sample.

The cap-7 column above serves the purpose a true baseline would have: it is a
measured before/after on the same commit, same method, same idle box, and it is
what actually justifies the divisor. If someone does want the uncapped number,
run it under `-m 24G` so the bad case is OOM-killed inside its own systemd
scope instead of taking the desktop with it.

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
place. The probe was reverted rather than kept, so nothing in the tree
reproduces it on demand -- a deliberately panicking test in a copy-pastable
utility crate is worse than the recipe. To re-run it, append to `src/lib.rs`:

    #[cfg(test)]
    mod backtrace_probe {
        #[test]
        fn deliberate_panic_for_backtrace_check() {
            panic!("probe");
        }
    }

then `RUST_BACKTRACE=1 cargo test backtrace_probe`, read the frame, and revert.
A first-party frame carrying `at ./src/...` is the pass condition. Note that checking the linked binary's `.debug_line` section instead
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

## Post-landing: sampled per-link peaks are lower bounds

Added 2026-07-31 after the task closed, from the nova-protocol session's
independent measurement on the same box. It does not change the shipped cap,
but it changes how much confidence the per-link number deserves.

`sample-peak-rss.sh` polls once a second. That is fine for a whole-run bound and
misleading for a single link: rust-lld's RSS ramps and peaks narrowly near the
end of the link, so a 1-second grid usually samples the ramp. Measured there on
one link, same class as ours:

| Method | Largest single rust-lld |
| --- | --- |
| 1s sampler, whole-suite run | 2.14 GiB |
| `time -v` on an isolated `-j1` link | 2.93 GiB |

`time -v` reads `wait4`'s `ru_maxrss` and structurally cannot miss a peak, so
the sampler under-reported by ~27% on a link lasting several seconds -- not a
case of missing the process, just of missing its spike.

Consequences for the numbers above:

- The 3.0 GB largest `rust-lld` on `--features debug` is a floor; the true
  figure is plausibly ~3.8 GB. Do not shave the divisor against it.
- The whole-run totals (11.6 / 10.6 / 13.5 / 9.9 GB) are much less affected:
  with N links in flight the sum smooths any individual miss. The cap is sized
  off 13.5 GB, so the conclusion stands.
- Tightening the interval is the wrong fix -- `ps -eo rss` at 100ms across a
  24-core storm perturbs what it measures. Use the sampler for whole-run bounds
  and `time -v` at `-j1` for per-link numbers.

Related: the withdrawn 13.1 GB figure was taken under
`systemd-run -p MemoryMax=24G`. A memory ceiling changes reclaim and page-cache
behaviour, so a link that would peak at 3 GB unconstrained can complete under
the cap by reclaiming instead. Any `-m` run is a different experiment, not a
comparable one -- which is a second, independent reason that figure could not be
reconciled with the others.
