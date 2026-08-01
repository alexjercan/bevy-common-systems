#!/usr/bin/env bash
#
# Fail loud on stale references left behind by a file move, rename or
# deletion. Give it the path (or any literal string) that no longer exists:
#
#     ./scripts/check-stale-refs.sh old/dir/renamed.md
#     ./scripts/check-stale-refs.sh -x web/dist old/dir/renamed.md gone.rs
#
# Options come first; a `-` prefixed operand after the first needle is an
# error rather than a silently-mistaken needle. Use `--` for a needle that
# really does start with a dash.
#
# NOTE: the example paths above are fictional on purpose. A real deleted path
# written here would make this file a permanent hit for the very check the
# caller is running.
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
# Only TRACKED files are scanned. Run this after `git add`, or a brand new
# file carrying the stale reference is invisible to it.
#
# Probed every way (task 20260801-100108) -- a checker that has only ever been
# run clean is untested, and "both ways" is really three: match, no-match, and
# tool failure:
#
#   * positive control: `./scripts/check-stale-refs.sh AGENTS.md` exits 1 and
#     lists the many real references to AGENTS.md in the tree.
#   * negative control: a nonsense needle that appears nowhere in the repo
#     exits 0 with "no stale references".
#   * tool failure: run from outside a git repo and it exits 2 -- never 0
#     ("clean") and never 1 ("hits"), which matters because the natural way to
#     assert the positive control is `! check-stale-refs.sh <needle>`, and that
#     idiom would turn a crash into a pass. The `>=2` branch below covers the
#     same thing for a git grep that fails mid-run; it is not reachable through
#     `-x`, since every `-x` value is nested inside `:(exclude)` and a bad
#     pathspec magic there parses as an inert literal path rather than an
#     error. It was probed with a `git` shim that exits 128 on `grep`.
#   * empty exclude: `-x ''` is rejected with exit 2, because the bare pathspec
#     `:(exclude)` excludes the WHOLE tree -- `-x "$UNSET_VAR"` would otherwise
#     silently pass green.
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
        x)
            # NOTE: a bare `:(exclude)` with no path excludes the entire tree,
            # so an empty -x (e.g. an unset variable) would turn the checker
            # off and still report clean. Refuse it.
            if [ -z "$OPTARG" ]; then
                echo "error: -x needs a non-empty pathspec" >&2
                usage
                exit 2
            fi
            excludes+=(":(exclude)$OPTARG")
            ;;
        h) usage; exit 0 ;;
        *) usage; exit 2 ;;
    esac
done
# NOTE: getopts consumes an explicit `--` itself, so ask whether the last
# argument it looked at was one before shifting it away.
explicit_end=false
if [ "${*:$((OPTIND - 1)):1}" = "--" ]; then
    explicit_end=true
fi
shift $((OPTIND - 1))

if [ "$#" -eq 0 ]; then
    usage
    exit 2
fi

# NOTE: getopts also stops at the first non-option, so `... needle -x web`
# would silently treat `-x` and `web` as needles and exclude nothing. Reject a
# dash-prefixed operand; `--` escapes a needle that really starts with one.
if [ "$explicit_end" = false ]; then
    for needle in "$@"; do
        case "$needle" in
            -*)
                echo "error: options must precede needles; use -- for a needle starting with '-'" >&2
                usage
                exit 2
                ;;
        esac
    done
fi

root=$(git rev-parse --show-toplevel) || exit 2
cd "$root"

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
