#!/usr/bin/env bash
#
# Probe `check-comment-tags.sh` against fixtures it must match and fixtures it
# must not, plus the argument surface. A checker that has only ever been run
# clean is untested, and "both ways" is really three: match, no-match, tool
# error (lessons `probe-a-new-checker-both-ways`, `probe-the-argument-surface-too`).
#
# The tool-error probe matters most: the natural way to assert the positive
# control is `! check-comment-tags.sh <fixture>`, and that idiom turns a crash
# into a pass. Exit 2 has to be distinguishable from exit 1.
#
# Usage: scripts/test-check-comment-tags.sh
# Exits 0 when every probe holds, 1 naming the first that does not.

set -uo pipefail

here=$(cd "$(dirname "$0")" && pwd)
checker="$here/check-comment-tags.sh"
fixtures="$here/fixtures/comment_tags"

failures=0

fail() {
    echo "FAIL: $*" >&2
    failures=$((failures + 1))
}

# Run the checker, capturing output and exit code without tripping `set -e`.
run() {
    out=$("$checker" "$@" 2>&1)
    code=$?
}

expect_code() {
    local want=$1 what=$2
    if [ "$code" -ne "$want" ]; then
        fail "$what: expected exit $want, got $code"
        printf '%s\n' "$out" >&2
        return 1
    fi
    return 0
}

expect_output() {
    local needle=$1 what=$2
    case $out in
    *"$needle"*) ;;
    *)
        fail "$what: output does not mention '$needle'"
        printf '%s\n' "$out" >&2
        ;;
    esac
}

# --- Rule 2, match: every documented test fn in the violating fixture hides a
# value guard behind its `///`, including the one under
# `#[cfg(all(test, not(target_arch = "wasm32")))]`.
run "$fixtures/violating.rs"
if expect_code 1 "violating fixture"; then
    expect_output "guarded from a test fn's ///" "violating fixture"
    for literal in 0.35 4.49 4.51 12; do
        expect_output ": $literal --" "violating fixture (literal $literal)"
    done
    # The violating fixture must isolate rule 2: if rule 1 also fires on it, an
    # exit of 1 stops being evidence about the rule under test.
    case $out in
    *"untagged non-doc comment block"*)
        fail "violating fixture also trips rule 1; it must isolate rule 2"
        ;;
    esac
fi

# --- Rule 2, no-match: near misses the rule must stay silent on.
run "$fixtures/compliant.rs"
expect_code 0 "compliant fixture"

# --- Rule 1 still fires, and both rules run in one pass rather than the first
# short-circuiting the second.
run "$fixtures"
if expect_code 1 "both fixtures together"; then
    expect_output "guarded from a test fn's ///" "both fixtures together"
fi

# --- Tool error: usage failures must be exit 2, never 0 ("clean") and never
# 1 ("hits"), or `! checker <path>` would read a crash as a pass.
run
expect_code 2 "no arguments"

run "$fixtures/no-such-file.rs"
expect_code 2 "nonexistent path"

empty=$(mktemp -d)
trap 'rm -rf "$empty"' EXIT
run "$empty"
expect_code 2 "a directory holding no .rs files is not silently clean"

# --- Tool error: a file whose braces do not balance means the parser lost
# track, so rule 2's verdict on it is void. Every desync found in review round
# 1 presented as a CLEAN run; this is the guard that makes the next one loud.
cat >"$empty/desync.rs" <<'RS'
#[cfg(test)]
mod t {
    /// A test fn documenting 3.75 that the parser will never reach.
    #[test]
    fn unreachable() {
        assert!(measure() < 3.75);
    }
RS
run "$empty/desync.rs"
if expect_code 2 "unbalanced file is a tool error, not a clean run"; then
    expect_output "brace depth ended at" "unbalanced file"
fi

if [ "$failures" -ne 0 ]; then
    echo "test-check-comment-tags: $failures probe(s) failed" >&2
    exit 1
fi

echo "test-check-comment-tags: all probes pass"
