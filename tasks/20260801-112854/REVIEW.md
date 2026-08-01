# Review: Release 0.19.6

- TASK: 20260801-112854
- BRANCH: chore/release-0.19.6
- WORKTREE: /home/alex/.cache/sprouts/bevy-common-systems/chore/release-0.19.6
- BASE: master

## Round 1

- REVIEWER: out-of-context
- VERDICT: REQUEST_CHANGES

Out-of-context general-purpose subagent (`a1fb1094a71e456d5`), read-only,
against `git diff master...HEAD` at 084062a. Fixes applied below; re-verdict
APPROVE.

Independently re-derived and CONFIRMED the load-bearing claim: no public item
added, removed or re-signed since `v0.19.5`. The `pub` grep returns 7 lines,
5 of them `pub(super)` inside the two new private modules and 2 comment text,
with no `-pub` line at all; the removed side of `mesh/builder.rs` and
`integrity/plugin.rs` is verbatim-moved private code. Also confirmed clean:
every CHANGELOG bullet against the real `v0.19.5..b3dc1e6` diff, all 7 link
refs and their compare ranges against `origin`, version consistency across
both manifests and both `Cargo.lock` entries (`git grep '0\.19\.5'` outside
`tasks/` / `CHANGELOG.md` / `LESSONS.md` returns nothing), ASCII over all 5
changed files, and `tatr check` exit 0.

| # | Sev | Finding | Fix |
|-|-|-|-|
| R1.1 | MAJOR | The "tag exists and both are pushed" DoD item was ticked `[x]` on a branch that cannot satisfy it -- no `v0.19.6` tag locally or on `origin`, branch unpushed. The Close-out's Evidence never claimed it either, so the record would have permanently misstated the release. | Unticked, with an explicit "ticked at LAND time, not on the branch" note and the consequence spelled out: the `[0.19.6]` compare link is dead until the tag lands. |
| R1.2 | MINOR | Close-out enumerated the 46-line non-comment diff as containing "two assertion messages". Only one is in that set (`screenshot.rs`, "a plain toggle value"); the other three are inside the split files the same sentence excludes. The enumeration also omitted the two `mod` declarations and counted an end-of-line comment as an assertion message. | Enumeration rewritten to the actual 46: two `mod` lines, the `manual_contains` fix, the `axis` shadowing plus its comment, `simulate_seconds`, one new test, one assertion message, one more end-of-line comment. |
| R1.3 | NIT | The CHANGELOG RAM bullet credited only `nix develop`, but the same work added `[profile.dev.package."*"] debug = false` to `Cargo.toml` -- the "cap the size of one binary" half that the flake comment itself points at. A contributor reading the changelog would not know the dev profile changed. | Bullet rewritten as the two halves that hold together: the dev-profile change (with the "first-party frames keep file and line" caveat) and the derived job caps. |

R1.2 is an instance of the ledger's own `record-numbers-from-a-rerun`: the
enumeration was written from the reading pass rather than re-derived from the
46 lines it claims to describe.

## Round 2

- REVIEWER: primary
- VERDICT: APPROVE

All three round-1 findings fixed in 21ad5c9. No new finding: the fixes touch
`CHANGELOG.md` prose and the task record only, and the round-1 verification of
the no-API-change claim, the link refs and version consistency still stands
(nothing under `src/` or in either manifest changed between rounds).
