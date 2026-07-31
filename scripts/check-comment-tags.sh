#!/usr/bin/env bash
#
# Enforce the own-line comment convention (see AGENTS.md): a non-doc comment
# that survives the KISS pass guards a value, explains a non-obvious setting,
# or records a hazard, and says so with a tag. Every such comment BLOCK must
# therefore open with NOTE:, FIXME:, BUG: or TODO: on its first line.
#
# Two things are deliberately NOT checked, matching the AGENTS.md bullet:
# - rustdoc (`///`, `//!`), which is the public API surface;
# - end-of-line comments (`let x = 1; // 1 neighbor`), which label a value in
#   place and read worse with a tag. They still have to earn their keep, but
#   that is a review call, not a grep.
#
# Usage: scripts/check-comment-tags.sh <path>...
# Paths are files or directories; directories are scanned for *.rs.
# Exits non-zero and prints file:line:text for each untagged block.

set -euo pipefail

if [ "$#" -eq 0 ]; then
    echo "usage: $0 <path>..." >&2
    exit 2
fi

# NUL-delimited so paths containing whitespace survive the hand-off to awk.
files=()
while IFS= read -r -d '' file; do
    files+=("$file")
done < <(find "$@" -type f -name '*.rs' -print0 | sort -z)

if [ "${#files[@]}" -eq 0 ]; then
    echo "check-comment-tags: no .rs files under $*" >&2
    exit 2
fi

# A block is a run of consecutive comment lines; only its first line is
# checked, so a kept hazard may wrap without repeating the tag on every line.
# `//` followed by `/` or `!` is rustdoc and resets the run like any code
# line would, so a doc comment cannot mask the comment that follows it.
# Any spacing after the slashes counts as tagged - rustfmt does not normalize
# comment interiors, so `//NOTE:` must not be reported as untagged.
untagged=$(awk '
    /^[ \t]*\/\/([^\/!]|$)/ {
        if (!in_block) {
            in_block = 1
            if ($0 !~ /\/\/[ \t]*(NOTE|FIXME|BUG|TODO):/) {
                print FILENAME ":" FNR ":" $0
            }
        }
        next
    }
    { in_block = 0 }
' "${files[@]}")

if [ -n "$untagged" ]; then
    count=$(printf '%s\n' "$untagged" | wc -l)
    echo "error: $count untagged non-doc comment block(s); tag with NOTE:/FIXME:/BUG:/TODO: or delete (see AGENTS.md):" >&2
    printf '%s\n' "$untagged" >&2
    exit 1
fi

echo "check-comment-tags: every non-doc comment block in $* is tagged"
