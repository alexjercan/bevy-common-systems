# Versioned releases and a third-party license gate

Date: 2026-07-16

## What changed

- Bumped `bevy_common_systems` (and the `bevy_common_systems_macros` sub-crate)
  from `0.0.1` to `0.19.0`. The version now tracks Bevy's minor: bcs 0.19.x
  targets Bevy 0.19.x, so a consumer can read the compatible Bevy line straight
  off the bcs version.
- Marked both crates `publish = false` and gave them explicit `license = "MIT"`
  (plus `homepage`/`repository` on the root). bcs is distributed as a git
  dependency and is never pushed to crates.io; `publish = false` also makes both
  crates "first-party" for cargo-about, so they stay out of the third-party
  license manifest.
- Added the cargo-about license setup, mirroring nova-protocol:
  - `about.toml` - the accepted permissive-license set and the ship targets
    (native desktop + wasm). Generation FAILS on any license outside the set, so
    a copyleft/unknown-license dependency is surfaced instead of silently
    shipped.
  - `about.hbs` - the manifest template.
  - `scripts/gen-licenses.sh` - regenerates `credits/THIRD-PARTY-LICENSES.md`
    locally.
  - A `licenses` job in `.github/workflows/ci.yml` that runs the generation and
    lets it fail as the gate (no project build needed - cargo-about only reads
    `cargo metadata` and crate license files).
- Added `.github/workflows/release-flow.yaml`, triggered on `v*` tags (and
  `workflow_dispatch`). bcs has no native binary to ship - the one compiled
  artifact it distributes is the WebAssembly web showcase - so the workflow
  builds `web/` (via the nix devshell, like the Pages deploy), drops the
  generated `THIRD-PARTY-LICENSES.md` next to it, zips `web/dist`, and attaches
  the zip to the matching GitHub Release.

## Why

nova-protocol already gates its shipped binaries this way; bcs is a source
dependency of nova, and its web showcase ships compiled wasm that statically
links the same dependency graph, so the same attribution obligation and the same
copyleft gate apply. Keeping the two repos' license tooling identical means one
mental model for both.

The `0.19.0` version was chosen deliberately to match Bevy's minor rather than
starting a fresh `0.1.0` semver line - Bevy compatibility is the single most
load-bearing fact about a bcs release, so encoding it in the version is worth
more than a conventional "first real release" number.

## How releases work now

1. Bump the version in both `Cargo.toml` files (and the two `Cargo.lock`
   entries) to the new Bevy-matched minor.
2. Commit, then tag `vX.Y.Z` and push the tag.
3. `release-flow.yaml` builds the showcase and publishes the GitHub Release.
4. Consumers (e.g. nova-protocol) pin the git dependency to that tag:
   `bevy_common_systems = { git = "...", tag = "vX.Y.Z" }`.

## Verification

`cargo about generate about.hbs -o ...` was run locally against the full graph:
exit 0, 461 MIT + a handful of other permissive crates, no rejections, and the
first-party `bevy_common_systems*` crates correctly excluded from the manifest.
The `mkdir -p credits` in `gen-licenses.sh` (and the committed `credits/.gitkeep`)
ensures the output directory exists on a fresh checkout.
