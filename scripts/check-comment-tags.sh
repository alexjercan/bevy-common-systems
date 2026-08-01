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
# The rustdoc exemption has one hole, which the SECOND rule below closes. A
# `///` on a test fn inside `#[cfg(test)]` is the right home for what the test
# PROVES, but rustdoc never renders it and rule 1 exempts it -- so it is the
# natural place to dump a value guard that the tag rule would otherwise have
# forced into a `NOTE:`. Three consecutive authors did exactly that (tasks
# 20260731-172224, -172232, -172233); see AGENTS.md.
#
# Rule 2 flags the signature of that migration: a numeric literal that appears
# BOTH in a test fn's `///` block AND in its body. If the doc is spelling out
# a number the body uses, the doc is guarding a value.
#
# Deliberate narrowing, measured on this tree before it shipped (see
# tasks/20260801-093730/NOTES.md) -- the naive "any untagged literal in a
# documented test fn" reads 183 hits in src/ alone and is unshippable:
# - a fn whose body holds ANY tagged block is exempt: the author demonstrably
#   applied the split, and which literal each block covers is a review call;
# - a literal on a line carrying an end-of-line comment is exempt, matching
#   rule 1's own exemption;
# - bare `0`, `1` and `2` never count. They collide with prose ("1%", "test 2")
#   far more often than they name a magic value.
#
# Known and accepted gap: a doc that justifies a value WITHOUT writing its
# digits ("the sliver value") is invisible here. All three recorded occurrences
# wrote the digits.
#
# Usage: scripts/check-comment-tags.sh <path>...
# Paths are files or directories; directories are scanned for *.rs.
# Both rules always run; the script exits 1 if either fires, 2 on a usage
# error. Rule 1 prints file:line:text, rule 2 prints file:line: literal -- text.

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

# Rule 2. Regions are tracked by brace depth rather than by "the rest of the
# file", so a `///` on a public item AFTER the test module is not read as test
# doc. String and char literals are blanked before braces are counted, so a
# `"{rate}"` in an assert message cannot shift the depth.
guarding=$(awk '
    # Blank out everything on a line that is not code, so no brace inside it
    # can shift the brace depth. Four sources, every one found by probing
    # rather than by reading (review round 1):
    # - comment text. A brace named in prose is the easiest one to write.
    # - string literals.
    # - raw strings. `r#"..."#` holds the repo`s JSON test fixtures, which are
    #   nothing but braces.
    # - char literals, but ONLY those: a tick opens one just when a closing
    #   tick follows within the next two characters. A lone tick is a LIFETIME,
    #   and treating it as a quote swallowed the rest of its line -- including
    #   the brace on a struct or impl header.
    # All four desync the depth the same way: the enclosing test region reads
    # as closed early and every test fn below it goes unexamined. The symptom
    # is always a SILENT pass, never a false report, which is why each one
    # needs its own fixture line rather than trust.
    # `q` and `rawterm` are deliberately NOT locals: a string literal may span
    # lines, and this repo`s JSON fixtures are multi-line `r#"..."#` blocks
    # full of braces. Carrying the open-literal state across lines is what
    # keeps their contents out of the depth count.
    function blank(s,   out, c, i, prev, term, p) {
        out = ""
        prev = " "
        i = 1
        if (rawterm != "") {
            p = index(s, rawterm)
            if (p == 0) return ""
            i = p + length(rawterm)
            rawterm = ""
            prev = "\""
        }
        for (; i <= length(s); i++) {
            c = substr(s, i, 1)
            if (q != "") {
                if (c == "\\") { i++; continue }
                if (c == q) q = ""
                continue
            }
            if (c == "/" && substr(s, i + 1, 1) == "/") break
            if (c == "r" && prev !~ /[A-Za-z0-9_]/ && match(substr(s, i), /^r#*"/)) {
                term = "\"" substr(s, i + 1, RLENGTH - 2)
                p = index(substr(s, i + RLENGTH), term)
                # No terminator on this line: the raw string runs on, and
                # `rawterm` carries what ends it to the next line.
                if (p == 0) { rawterm = term; break }
                i = i + RLENGTH + p + length(term) - 2
                prev = "\""
                continue
            }
            if (c == "\"") { q = c; prev = c; continue }
            if (c == "'"'"'" && substr(s, i) ~ /^'"'"'(\\.|[^'"'"'\\])'"'"'/) {
                q = c
                prev = c
                continue
            }
            out = out c
            prev = c
        }
        return out
    }
    # Collect every numeric literal in `s` into `arr`. A digit run preceded by
    # an identifier or `.` character is part of a name or a longer number, not
    # a literal of its own.
    function nums(s, arr,   rest, tok, pre) {
        rest = s
        while (match(rest, /[0-9]+(\.[0-9]+)?/)) {
            tok = substr(rest, RSTART, RLENGTH)
            pre = (RSTART > 1) ? substr(rest, RSTART - 1, 1) : " "
            if (pre !~ /[A-Za-z0-9_.]/ && tok != "0" && tok != "1" && tok != "2") {
                arr[tok] = 1
            }
            rest = substr(rest, RSTART + RLENGTH)
        }
    }
    function endfn(   i) {
        if (in_fn && !suppressed) for (i = 1; i <= n_hit; i++) print hit[i]
        in_fn = 0; started = 0; n_hit = 0; suppressed = 0
    }
    # A .rs file always balances to depth 0. If it does not, this pass lost
    # track of the braces and every conclusion it drew about the file is void
    # -- and the visible symptom would otherwise be a clean run. Say so
    # instead. Four separate desync causes were found by probing during review
    # round 1; this is the guard that stops the fifth from being silent.
    function checkbalance() {
        if (curfile == "") return
        if (depth != 0) print "DESYNC:" curfile ": brace depth ended at " depth ", not 0"
        else if (rawterm != "" || q != "") print "DESYNC:" curfile ": unterminated string literal"
    }
    FNR == 1 {
        endfn(); checkbalance(); curfile = FILENAME
        depth = 0; in_test = 0; test_depth = -1; pending = 0; in_doc = 0
        q = ""; rawterm = ""
    }
    {
        code = blank($0)
        opens = gsub(/\{/, "{", code)
        closes = gsub(/\}/, "}", code)
        next_depth = depth + opens - closes

        if (!in_test) {
            # `pending` bridges `#[cfg(test)]` to the brace of the item it
            # applies to, over further attributes and a wrapped item header.
            # A statement end (`;`) or a closing brace means the item was
            # BRACE-LESS and is already over, so the latch drops. Without that,
            # `#[cfg(test)] use ...;` latched until the next brace anywhere
            # below, and production code was read as a test region -- reporting
            # public rustdoc, which this rule must never touch. Round 1.
            if ($0 ~ /^[ \t]*#\[cfg\((all\()?[ \t]*test[,)]/) pending = 1
            else if (pending && code ~ /;[ \t]*$/) pending = 0

            if (pending && opens > 0) { in_test = 1; test_depth = depth; pending = 0 }
            else if (pending && closes > 0) pending = 0
        } else if (in_fn) {
            if ($0 ~ /\/\/[ \t]*(NOTE|FIXME|BUG|TODO):/) {
                suppressed = 1
            } else if ($0 !~ /\/\//) {
                split("", body_nums)
                nums(code, body_nums)
                for (tok in body_nums) {
                    if (tok in doc_nums) hit[++n_hit] = FILENAME ":" FNR ": " tok " -- " $0
                }
            }
            if (next_depth > fn_depth) started = 1
            else if (started) endfn()
        } else if ($0 ~ /^[ \t]*\/\/\//) {
            if (!in_doc) { split("", doc_nums); in_doc = 1 }
            nums($0, doc_nums)
        } else if (in_doc && $0 ~ /^[ \t]*#\[/) {
            # An attribute between the doc and the item it documents keeps the
            # run alive; `#[test]` is the common case.
        } else if (in_doc && $0 ~ /^[ \t]*(pub[ \t]+)?(async[ \t]+)?fn[ \t]/) {
            in_fn = 1; started = 0; n_hit = 0; suppressed = 0; fn_depth = depth
            in_doc = 0
        } else {
            in_doc = 0
        }

        if (in_test && next_depth <= test_depth) {
            endfn(); in_test = 0; test_depth = -1; in_doc = 0
        }
        depth = next_depth
    }
    END { endfn(); checkbalance() }
' "${files[@]}")

# A desync means rule 2 lost the braces and its verdict on that file is
# meaningless. That is a TOOL failure, not a finding, so it exits 2: a caller
# writing `! check-comment-tags.sh <path>` must not read it as "clean".
desync=$(printf '%s\n' "$guarding" | grep '^DESYNC:' || true)
if [ -n "$desync" ]; then
    echo "error: the test-region parser lost brace depth; rule 2's verdict on these files is void:" >&2
    printf '%s\n' "$desync" | sed 's/^DESYNC://' >&2
    exit 2
fi

status=0

if [ -n "$untagged" ]; then
    count=$(printf '%s\n' "$untagged" | wc -l)
    echo "error: $count untagged non-doc comment block(s); tag with NOTE:/FIXME:/BUG:/TODO: or delete (see AGENTS.md):" >&2
    printf '%s\n' "$untagged" >&2
    status=1
fi

if [ -n "$guarding" ]; then
    count=$(printf '%s\n' "$guarding" | wc -l)
    echo "error: $count literal(s) guarded from a test fn's /// instead of a body NOTE:; move the value guard into the body (see AGENTS.md):" >&2
    printf '%s\n' "$guarding" >&2
    status=1
fi

if [ "$status" -ne 0 ]; then
    exit 1
fi

echo "check-comment-tags: every non-doc comment block in $* is tagged, and no test fn's /// guards a literal"
