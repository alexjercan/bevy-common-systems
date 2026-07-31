#!/usr/bin/env bash
#
# Measure the peak memory of a Rust build/test command.
#
# `cargo test` links one full Bevy+avian binary per target (lib unittest, every
# example, every doctest), and rust-lld holds each one in memory. The peak
# therefore scales with the number of targets times the link parallelism, and it
# has outgrown its fix once already (see tasks/20260703-000003 and
# tasks/20260731-210044). That first fix was measured ad hoc, so nobody noticed
# the workload growing past it. This script makes the measurement re-runnable:
# after adding examples or doctests, run it and compare against the numbers in
# tasks/20260731-210044/NOTES.md.
#
# Usage:
#   ./scripts/sample-peak-rss.sh -- nix develop --command cargo test
#   ./scripts/sample-peak-rss.sh -m 20G -- nix develop --command cargo test --doc
#
# Options:
#   -m LIMIT   Run the command inside a memory-capped systemd user scope
#              (systemd-run -p MemoryMax=LIMIT). Use this whenever measuring a
#              configuration expected to blow up: without it, reproducing the
#              bug thrashes swap and takes the desktop down with it. The kernel
#              OOM-kills inside the scope instead, and the reported exit code
#              tells you it hit the wall.
#   -i SECS    Sampling interval. Default 1.
#
# Prints the peak summed RSS of all cargo/rustc/rustdoc/rust-lld processes, the
# largest single rust-lld, and exits with the wrapped command's own status.
#
# NOTE: the sample is system-wide, so a build running in another checkout or
# worktree is counted too. Check for competing cargo/rust-lld processes before
# trusting a number, and say in the record whether the box was otherwise idle.

set -uo pipefail

interval=1
memmax=""

while getopts "m:i:" opt; do
    case "$opt" in
        m) memmax="$OPTARG" ;;
        i) interval="$OPTARG" ;;
        *) echo "usage: $0 [-m LIMIT] [-i SECS] -- CMD..." >&2; exit 2 ;;
    esac
done
shift $((OPTIND - 1))

# Tolerate the `--` separator that documents where the command begins.
[ "${1:-}" = "--" ] && shift

if [ "$#" -eq 0 ]; then
    echo "usage: $0 [-m LIMIT] [-i SECS] -- CMD..." >&2
    exit 2
fi

peak_file=$(mktemp)
trap 'rm -f "$peak_file"' EXIT
echo "0 0" > "$peak_file"

# NOTE: match on the process name, not a command-line pattern. A `pgrep -f`
# style match would also match this script's own argv (it names `cargo`) and,
# in the kill case, take out the shell running it.
sample() {
    local total=0 largest=0
    while true; do
        read -r total largest < <(
            ps -eo rss=,comm= | awk '
                $2 == "cargo" || $2 == "rustc" || $2 == "rustdoc" || $2 == "rust-lld" { s += $1 }
                $2 == "rust-lld" && $1 > m { m = $1 }
                END { print s + 0, m + 0 }'
        )
        read -r peak_total peak_lld < "$peak_file"
        [ "$total" -gt "$peak_total" ] && peak_total=$total
        [ "$largest" -gt "$peak_lld" ] && peak_lld=$largest
        # NOTE: write atomically. A plain `>` truncates before it writes, so the
        # kill below can land in that window and leave the file empty -- which
        # would print a confident "0.0 GB" instead of the measurement.
        echo "$peak_total $peak_lld" > "$peak_file.tmp" && mv -f "$peak_file.tmp" "$peak_file"
        sleep "$interval"
    done
}

sample &
sampler_pid=$!
# NOTE: kill by recorded PID. `pkill -f` on a pattern drawn from our own argv
# would match this very shell.
trap 'kill "$sampler_pid" 2>/dev/null; rm -f "$peak_file"' EXIT

if [ -n "$memmax" ]; then
    # NOTE: preflight. Without this, a missing systemd-run means the wrapped
    # command never runs and the output reads `command exit 127`, which is
    # indistinguishable from the build itself failing.
    if ! command -v systemd-run > /dev/null 2>&1; then
        echo "$0: -m needs systemd-run, which is not on PATH" >&2
        exit 2
    fi
    systemd-run --user --scope -q -p "MemoryMax=$memmax" -- "$@"
else
    "$@"
fi
status=$?

kill "$sampler_pid" 2>/dev/null
wait "$sampler_pid" 2>/dev/null

read -r peak_total peak_lld < "$peak_file"

# NOTE: refuse to report a number we cannot vouch for. Silence beats a
# plausible-looking 0.0 GB when the peak file did not survive.
case "$peak_total$peak_lld" in
    *[!0-9]* | "") echo "peak-rss: measurement lost, no peak recorded (command exit $status)" >&2
                   exit "$status" ;;
esac

awk -v t="$peak_total" -v l="$peak_lld" -v s="$status" -v cap="${memmax:-none}" 'BEGIN {
    printf "peak-rss: toolchain total %.1f GB, largest rust-lld %.1f GB (MemoryMax=%s, command exit %d)\n", \
        t / 1048576, l / 1048576, cap, s
}'

exit "$status"
