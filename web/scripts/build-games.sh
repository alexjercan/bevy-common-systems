#!/usr/bin/env bash
#
# Build each showcased example to a self-contained static wasm page with trunk.
#
# Output goes under web/dist/games/<name>/ so the webpack gallery can link to /
# iframe it. PUBLIC_PATH (default "/") makes the assets resolve under a GitHub
# Pages subpath; it must match the gallery's PUBLIC_PATH.
#
# Requires the nix devshell (trunk + the wasm32 target). Run from the repo root
# or anywhere; paths are resolved relative to this script.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
public_path="${PUBLIC_PATH:-/}"
# Games build into a staging dir; webpack's CopyPlugin copies it into dist/games
# (webpack owns dist/ and cleans it, so the games cannot live directly there).
out_root="${OUT_ROOT:-$repo_root/web/build/games}"

# One entry per showcased game: "<example-name> <html-dir>".
games=(
  "06_fruitninja web/games/06_fruitninja"
)

for entry in "${games[@]}"; do
  read -r example html_dir <<<"$entry"
  echo ">> building $example"
  # Games are served from "<public_path>games/<example>/".
  game_public_path="${public_path%/}/games/$example/"
  trunk build \
    --release \
    --example "$example" \
    --public-url "$game_public_path" \
    --dist "$out_root/$example" \
    "$repo_root/$html_dir/index.html"
done

echo ">> games built into $out_root"
