#!/usr/bin/env bash
#
# Build each showcased example to a self-contained static wasm page with trunk.
#
# Output goes under web/dist/games/<name>/ so the webpack gallery can link to /
# iframe it. PUBLIC_PATH (default "/") makes the assets resolve under a GitHub
# Pages subpath; it must match the gallery's PUBLIC_PATH.
#
# Requires the nix devshell (trunk + the wasm32 target). Can be run from
# anywhere (e.g. `npm run build:games`, which invokes it from web/); it cd's to
# the repo root itself.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
public_path="${PUBLIC_PATH:-/}"

# trunk resolves its target / cargo project relative to the current directory
# and fails ("Unable to find any Trunk configuration") if run from a subdir like
# web/, so always drive it from the repo root.
cd "$repo_root"
# Games build into a staging dir; webpack's CopyPlugin copies it into dist/games
# (webpack owns dist/ and cleans it, so the games cannot live directly there).
out_root="${OUT_ROOT:-$repo_root/web/build/games}"

# One entry per showcased game: "<example-name> <html-dir>".
games=(
  "06_fruitninja web/games/06_fruitninja"
  "07_orbit web/games/07_orbit"
  "08_dropzone web/games/08_dropzone"
  "09_reactor web/games/09_reactor"
  "10_asteroids web/games/10_asteroids"
  "11_overload web/games/11_overload"
  "12_bastion web/games/12_bastion"
)

for entry in "${games[@]}"; do
  read -r example html_dir <<<"$entry"
  echo ">> building $example"
  # Games are served from "<public_path>games/<example>/".
  game_public_path="${public_path%/}/games/$example/"
  # Path is relative to repo_root (we cd'd there above).
  trunk build \
    --release \
    --example "$example" \
    --public-url "$game_public_path" \
    --dist "$out_root/$example" \
    "$html_dir/index.html"
done

echo ">> games built into $out_root"
