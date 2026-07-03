#!/usr/bin/env bash
#
# Enforce the repo-wide plain-ASCII writing rule (see AGENTS.md): no em
# dashes, smart quotes, ellipsis characters, arrows or other non-ASCII
# typographic characters in code, comments or docs.
#
# Exits non-zero and prints the offending file:line:text if any byte outside
# the 0x00-0x7F range is found under the source trees. Run from the repo root.

set -euo pipefail

# Directories that must stay plain ASCII.
roots=(src bevy_common_systems_macros/src examples)

# grep -P '[^\x00-\x7F]' matches any non-ASCII byte. -r recurse, -n line
# numbers. Branch on the exact exit code: 0 = matches found (violation),
# 1 = no matches (clean), >=2 = grep itself failed (e.g. a scanned
# directory is missing). We must fail loudly on >=2 rather than treat it as
# clean, otherwise a future refactor that renames a root would silently
# disable the guard while CI stays green.
set +e
matches=$(grep -rnP '[^\x00-\x7F]' "${roots[@]}")
status=$?
set -e

case "$status" in
    0)
        echo "error: non-ASCII characters found (plain ASCII is required, see AGENTS.md):" >&2
        echo "$matches" >&2
        exit 1
        ;;
    1)
        echo "check-ascii: no non-ASCII characters found in ${roots[*]}"
        ;;
    *)
        echo "check-ascii: grep failed (exit $status); cannot verify ${roots[*]}" >&2
        exit "$status"
        ;;
esac
