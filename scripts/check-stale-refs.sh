#!/usr/bin/env bash
#
# Fail loud on stale references left behind by a file move, rename or
# deletion. Give it the path (or any literal string) that no longer exists:
#
#     ./scripts/check-stale-refs.sh docs/wasm-web-builds.md
#     ./scripts/check-stale-refs.sh -x web/dist old/path.md another/path.md
#
# Two deliberate fail-loud choices (see AGENTS.md, and the ledger lesson
# probe-a-new-checker-both-ways):
#
#   * git grep over the tracked tree, from the repo root, so paths in the
#     output are repo-root relative and match how docs spell them. A plain
#     `grep -rn ... .` emits `./`-prefixed paths, which then silently defeat
#     any `grep -v '^tasks/'` style filter.
#   * exclusions, never an --include allowlist. An allowlist of file types
#     fails SILENT the day a new file type enters the repo; that is exactly
#     how 18 stale references in web/games/*/index.html survived a green
#     one-liner proof in task 20260801-094300.
#
# tasks/ is excluded by convention: task records are historical documents and
# are supposed to name files that no longer exist. Anything else the caller
# wants skipped goes through a repeated `-x <pathspec>`.
#
# Probed both ways (task 20260801-100108) -- a checker that has only ever been
# run clean is untested:
#
#   * positive control: `./scripts/check-stale-refs.sh AGENTS.md` exits 1 and
#     lists the many real references to AGENTS.md in the tree.
#   * negative control: a nonsense needle that appears nowhere in the repo
#     exits 0 with "no stale references".
#
# Run from anywhere inside the repo.

set -euo pipefail

usage() {
    echo "usage: $(basename "$0") [-x <pathspec>]... <needle>..." >&2
    echo "  -x  additional git pathspec to exclude (repeatable)" >&2
}

excludes=()
while getopts ":x:h" opt; do
    case "$opt" in
        x) excludes+=(":(exclude)$OPTARG") ;;
        h) usage; exit 0 ;;
        *) usage; exit 2 ;;
    esac
done
shift $((OPTIND - 1))

if [ "$#" -eq 0 ]; then
    usage
    exit 2
fi

cd "$(git rev-parse --show-toplevel)"

# NOTE: task records are historical by convention -- they are meant to name
# files that no longer exist, so they can never be a stale reference.
pathspecs=(":(exclude)tasks/" "${excludes[@]+"${excludes[@]}"}")

failed=0
for needle in "$@"; do
    # Branch on git grep's exact exit code: 0 = hits (violation), 1 = clean,
    # >=2 = git grep itself failed (bad pathspec, not a repo, ...). A tool
    # failure must NOT be reported as clean, otherwise the guard silently
    # disables itself while CI stays green.
    set +e
    matches=$(git grep -n --fixed-strings -e "$needle" -- "${pathspecs[@]}")
    status=$?
    set -e

    case "$status" in
        0)
            echo "error: stale references to '$needle':" >&2
            echo "$matches" >&2
            failed=1
            ;;
        1)
            echo "check-stale-refs: no stale references to '$needle'"
            ;;
        *)
            echo "check-stale-refs: git grep failed (exit $status); cannot verify '$needle'" >&2
            exit "$status"
            ;;
    esac
done

exit "$failed"
