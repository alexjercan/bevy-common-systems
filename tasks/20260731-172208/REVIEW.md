# Review: KISS pass -- debug/ + lib.rs + completion.rs

- TASK: 20260731-172208
- BRANCH: refactor/kiss-debug-cluster
- BASE: master @ 8500161
- WORKTREE: /home/alex/.cache/sprouts/bevy-common-systems/refactor/kiss-debug-cluster

## Round 1

- REVIEWER: primary session (in-context; exception recorded below)
- VERDICT: REQUEST_CHANGES

Reviewer note: primary session, in-context. The skill's out-of-context default was
NOT met: subagent dispatch is disabled for this session, and the diff is not
trivial, so this is a recorded exception rather than a satisfied requirement.
Compensating measure: every claim below was re-derived from the tree by
running the command shown, not from the implementation narrative.

### Independent re-derivations

- **"No behavior change."** Confirmed, and by a stronger method than the
  task's own `pub`-line grep: stripped all `//`-prefixed and blank lines from
  both `master` and `HEAD` versions of all eight in-scope files and diffed.
  The ONLY surviving difference across the whole cluster is
  `screenshot.rs:170`, the added `assert_eq!` message. Nothing else in the
  cluster changed.
- **Proofs rerun in the worktree**: `check-comment-tags.sh` exit 0; scoped
  HUID grep empty; `cargo doc --no-deps --features debug` 0 in-scope warnings
  and 10 total (baseline 11); `cargo fmt --check` clean; `cargo clippy
  --all-targets` and `--features debug` exit 0 (one pre-existing warn-level
  `manual_contains`, see the round-2 note); `cargo test` 59 pass,
  `--features debug` 66 pass, `--examples` 1 pass; `check-ascii.sh` clean;
  `tatr check --ledger LESSONS.md` clean.
- **Checker behavior probed directly** on a synthetic file rather than trusted
  from its source; that probe produced both findings below.

### Findings

#### MAJOR-1 -- the convention's stated scope exceeds what the checker enforces

`AGENTS.md:137`, `scripts/check-comment-tags.sh:32`

The new bullet reads "Inline (non-doc) comments only guard a value ... and
open with `NOTE:` / `FIXME:` / `BUG:` / `TODO:` on the block's first line ...
Enforced by `./scripts/check-comment-tags.sh`". The awk pattern is anchored
at `^[ \t]*//`, so END-OF-LINE comments are invisible to it. Probe:

```
let x = 1; // untagged trailing comment, pure restatement
```

is not reported; the script exits 0.

This is not hypothetical for the epic. `src/` already carries 12 trailing
comments, and every one of them sits in a SIBLING cluster's files:

| File | Example |
| --- | --- |
| `integrity/plugin.rs:354,355` | `// 1 neighbor`, `// 2 neighbors` |
| `physics/pd_controller.rs:189,191` | `// no clamp` |
| `mesh/builder.rs:563` | `// parallel to edge AB` |
| `modding/events.rs:466` | `// maintain_handler_index picks up ...` |
| `persist/mod.rs:161,183`, `scoring/streak.rs:207`, `tween/mod.rs:290`, `ui/menu.rs:141`, `transform/smooth_look_rotation.rs:41` | value labels |

This task exists to SET the convention the other four follow, so the
ambiguity propagates by design: a sibling reads the bullet, runs the script,
gets exit 0, and either ships untagged trailing comments (rule silently
unenforced) or tags them, turning `// 0..=1` into `// NOTE: 0..=1`, which the
KISS pass is supposed to prevent.

Change: decide the scope and make doc and script agree. Recommended -- narrow
the `AGENTS.md` bullet to own-line comments and state the trailing-comment
exemption explicitly (those 12 are legitimate value labels, not narration),
and say the same in the script header next to the existing rustdoc-exemption
line. Extending the awk to trailing comments is the alternative, but it
forces tags onto labels that read worse with them.

#### MINOR-1 -- a correctly tagged comment can be reported as untagged

`scripts/check-comment-tags.sh:33`

The regex `/\/\/ (NOTE|FIXME|BUG|TODO):/` demands exactly one space. Probe
output:

```
p.rs:3:    //NOTE: no space after slashes
p.rs:5:    //  NOTE: two spaces
```

Both are flagged, and the error text tells the author to "tag with
NOTE:/FIXME:/BUG:/TODO:" -- advice they already followed. rustfmt does not
normalize comment interiors by default, so neither spelling gets corrected
for them.

Change: allow any run of spaces after the slashes, e.g.
`/\/\/ *(NOTE|FIXME|BUG|TODO):/`.

#### MINOR-2 -- filenames are word-split into awk

`scripts/check-comment-tags.sh:39`

`awk '...' $files` is deliberately unquoted to pass a list, so any path
containing whitespace splits into broken arguments. No such path exists in
this repo today, but the script is now a documented general tool with a
`<path>...` interface and an `AGENTS.md` reference.

Change: `find ... -print0` piped to `xargs -0 awk -f`, or accumulate into a
bash array.

### Non-findings (checked, no issue)

- Promoting the two nova HUIDs into rustdoc is sound: the epic proof scans
  non-doc comments only, the HUIDs name another repo's tasks so the "live task
  record" carve-out genuinely could not apply, and provenance is useful to a
  downstream reader.
- Every DROP re-checked against the item's own rustdoc.
  `screenshot.rs:159` (overlay) and `:216` (bounded wait) were the two claimed
  to be redundant with `hide_debug_overlay`'s doc and `MAX_WAIT_FRAMES`'s doc
  respectively -- both confirmed to say the same thing within a few lines.
- The `///` on `rejects_non_positive_dimensions` correctly precedes `#[test]`.
- Blank line between comment paragraphs resets the block, so each paragraph
  needs its own tag. Consistent with the documented "block" definition.
- No-split call (D3) matches the measured numbers in NOTES.md.

### Round 1 verdict detail

REQUEST_CHANGES -- MAJOR-1 open.

Pending `manual:` items (do not block):

- Public API unchanged. Re-derived mechanically above and stronger than the
  stated check; a user spot-check of `cargo doc` output remains available.
- `NOTES.md` covers all 37 blocks with per-file measurements: verified present
  and complete.

## Round 2

- REVIEWER: primary session (in-context; same exception as round 1)
- VERDICT: APPROVE

### Responses to round 1

- **MAJOR-1 -- fixed.** `AGENTS.md:137` now says "Own-line (non-doc) comments"
  and names the end-of-line exemption explicitly, with an example
  (`let x = 1; // 1 neighbor`) and the standing requirement that such comments
  still earn their keep. `scripts/check-comment-tags.sh` header carries the
  same two exemptions side by side, so the script and the bullet state one
  rule. Scope chosen per the reviewer's recommendation: the 12 existing
  trailing comments are value labels that a `NOTE:` tag would degrade.
- **MINOR-1 -- fixed.** Tag regex is now
  `/\/\/[ \t]*(NOTE|FIXME|BUG|TODO):/`. Probed: `//NOTE:` and `//  NOTE:` both
  pass, and an untagged own-line comment is still reported.
- **MINOR-2 -- fixed.** `find -print0 | sort -z` into a bash array, passed to
  awk as `"${files[@]}"`. Probed with a directory named `dir with space`: the
  file inside is genuinely scanned (an untagged comment placed there IS
  reported), so the fix is verified by observed detection, not just by a clean
  exit.

### Verification rerun

- `check-comment-tags.sh src/debug src/lib.rs src/completion.rs` exit 0.
- Scoped HUID grep empty.
- `cargo fmt --check` exit 0; `cargo clippy --all-targets` exit 0;
  `--features debug` exit 0.
- `cargo test` 59 pass, `--features debug` 66 pass, `--examples` all pass.
- `cargo doc --no-deps --features debug`: 0 in-scope warnings, 10 total
  (baseline 11).
- `check-ascii.sh` clean (covers the reworded AGENTS.md bullet and script
  header).

### Pre-existing issue surfaced (not a finding against this diff)

Not a finding against this diff. `cargo clippy --all-targets` emits
`clippy::manual_contains` at `src/completion.rs:88`. Confirmed PRE-EXISTING on
`master` @ 8500161, and outside this diff (the comment edited in this file is
at line 110). Per the review rule on pre-existing problems, filed as tatr task
**20260731-180747** rather than folded into a comment-hygiene pass. It is
warn-level, so no `cmd:` proof fails.

### Round 2 verdict detail

APPROVE. No open BLOCKER or MAJOR. All three round-1 findings fixed and
each fix independently probed. Pending `manual:` items unchanged from round 1;
neither blocks.
