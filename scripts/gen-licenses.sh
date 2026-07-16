#!/usr/bin/env bash
# Regenerate the shipped third-party dependency-license manifest.
#
# Walks the dependency graph with cargo-about (config in about.toml, rendered
# through about.hbs) and writes credits/THIRD-PARTY-LICENSES.md, which ships
# alongside the web showcase build. Generation FAILS if any crate carries a
# license not in about.toml's `accepted` list, so a new copyleft/unknown-license
# dependency is surfaced instead of silently shipped.
#
# Run from the repo root:  ./scripts/gen-licenses.sh
# Requires cargo-about:     cargo install cargo-about
set -euo pipefail

cd "$(dirname "$0")/.."

if ! cargo about --version >/dev/null 2>&1; then
    echo "error: cargo-about not found. Install it with: cargo install cargo-about" >&2
    exit 1
fi

mkdir -p credits
cargo about generate about.hbs -o credits/THIRD-PARTY-LICENSES.md
echo "wrote credits/THIRD-PARTY-LICENSES.md"
