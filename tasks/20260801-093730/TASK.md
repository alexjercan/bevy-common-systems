# check-comment-tags: flag a /// on a test fn guarding an unexplained literal

- PRIORITY: 40
- TAGS: chore, tooling, lessons
- ACTIVITY: COMPOUNDING
- GATES: PLAN REVIEW RETRO
- RESOLUTION: DONE

Promotion of the ledger lesson `state-what-the-checker-cannot-see` (x3,
disposition taken 2026-08-01 during the close of epic 20260731-172116).

The recurring shape: a `///` doc comment on a test fn inside `#[cfg(test)]` is
the correct home for what the test PROVES, but rustdoc never renders it and
`check-comment-tags.sh` exempts it. Three consecutive authors used it correctly
in the main and then, once per outing, misused it to move a VALUE guard out of
the test body -- where the tag rule would have demanded a `NOTE:`.

Prose cannot see this; the checker can.

## Definition of Done

- The checker gains a second rule flagging a `///` test fn whose body holds an
  unexplained magic literal, reported as a distinct error class from the
  untagged-block rule (cmd: `./scripts/check-comment-tags.sh
  scripts/fixtures/comment_tags/violating.rs` exits 1 and prints a line naming
  the literal, under a header distinct from the untagged-block one).
- The rule is probed all three ways -- match, no-match, tool error (cmd:
  `./scripts/test-check-comment-tags.sh` exits 0, covering the violating
  fixture at exit 1, the compliant fixture at exit 0, and a no-argument run, a
  missing path, an empty directory and an unbalanced file at exit 2).
- A parse desync is loud, not silent: rule 2 exits 2 rather than reporting a
  file clean when it has lost brace depth (cmd: the `unbalanced file is a tool
  error` probe inside `test-check-comment-tags.sh`).
- The epic-wide gate is clean under both rules, so every base-tree hit is fixed
  in this change (cmd: `./scripts/check-comment-tags.sh src
  bevy_common_systems_macros/src` exits 0).
- The probes run in CI, so the fixtures cannot decay silently (cmd: `nix run
  nixpkgs#actionlint -- .github/workflows/ci.yml` exits 0, and the workflow has
  a `Comment-tag checker probes` step).
- The eight base-tree hits are fixed by moving the value guard back into the
  body, not by deleting the `///` (manual: `git diff master -- src/camera/shake.rs
  src/physics/pd_controller.rs` shows each fn keeping an intent-only `///` and
  gaining a tagged `NOTE:` block).
- The AGENTS.md own-line-comment bullet states the new rule (manual: read the
  bullet; it names the `///`-on-a-test-fn case and the checker that enforces it).
- Plain-ASCII rule holds (cmd: `./scripts/check-ascii.sh`).
- Formatting and lints clean (cmd: `nix develop --command cargo fmt --check`,
  `... cargo clippy --all-targets`, `... cargo clippy --all-targets --features debug`).
- Tests pass in both feature configurations (cmd: `nix develop --command cargo
  test`, `... cargo test --features debug`).
- Task artifacts lint clean (cmd: `tatr check`).

## Steps

1. Fix the eight hits the tuned rule finds on the base tree, since they are the
   misuse itself: `src/camera/shake.rs`
   (`offset_scales_with_amount_and_max_offset`,
   `shake_offset_stays_within_the_configured_bound`) and
   `src/physics/pd_controller.rs` (the four `1.5`/`0.7` rad/s roll tests).
   Each keeps an intent-only `///` and gains a body `NOTE:` naming why the
   value is that value.
2. Add `scripts/fixtures/comment_tags/violating.rs` and `compliant.rs`. Both
   must be clean under the EXISTING untagged-block rule, so the only axis under
   test is the new one. Use fictional identifiers (see the
   `a-tree-scanner-scans-itself` lesson: this tree is scanned by its own
   checkers).
3. Extend `scripts/check-comment-tags.sh` with the second awk pass. Definition,
   tuned by measurement rather than the naive one (see Notes): a hit is a
   numeric literal that appears BOTH in the `///` block and in the fn body,
   where the body has no tagged `NOTE:`/`FIXME:`/`BUG:`/`TODO:` block and the
   literal's line carries no end-of-line comment; bare `0`, `1`, `2` are
   excluded. Report file:line:literal. Both rules run; the script exits 1 if
   either fires, 2 on a usage error.
4. Add `scripts/test-check-comment-tags.sh` running the three probes above.
5. Update the AGENTS.md own-line-comment bullet and the script's header prose
   to state the rule and its definition.
6. Verify: run every `cmd:` proof, plus `cargo clippy --all-targets`.

## Notes

- The DoD's original parenthetical ("any literal with no tagged block and no
  end-of-line comment") was measured on the base tree and rejected: 183 hits in
  `src` alone, 253 including `examples`. The task's own `manual:` clause
  sanctions tuning over shipping a noisy check. Measured alternatives:
  "`///` block contains any literal" = 26 hits, mostly legitimate intent
  ("Yawing 90 degrees"); the shipped doc-and-body correlation rule = 8 hits
  across 6 test fns in 2 files, 0 in `examples`.
- The correlation is the actual signal: the value migrated out of the body, so
  it shows up in both places at once. It reproduces the historical occurrence
  (`ui/health_display.rs`, literals `0.4`/`2.29`/`2.3`) and stays green on its
  fixed form, which now carries a body `NOTE:`.
- Known and accepted gap: prose that justifies a value WITHOUT writing its
  digits ("the sliver value") is invisible to this rule. KISS -- all three
  recorded occurrences wrote the digits.
- The task body's claim that the script "already parses `#[cfg(test)]`
  regions" is wrong; it does not. Step 3 adds that parsing.
- `#[cfg(all(test, not(target_arch = "wasm32")))]` must match too
  (`probe-a-new-checker-both-ways`, 20260731-172208).
- Origin occurrences: 20260731-172224, 20260731-172232, 20260731-172233.

## Close-out

Evidence numbers here come from re-running their command at write time, with
the unit named; a bare integer sits adjacent to the command that printed it.

### What and why

`scripts/check-comment-tags.sh` gains a second rule closing the hole in its own
rustdoc exemption: a numeric literal written in BOTH a test fn's `///` block
and its body is reported, because a doc spelling out a number the body uses is
guarding a value. Full design record and the measurements behind the tuning:
`tasks/20260801-093730/NOTES.md`.

### Difficulties

The DoD as planned named `examples` in the epic-wide gate. That was wrong: rule
1 has never been applied to `examples/`, which carries 769 untagged blocks
(`./scripts/check-comment-tags.sh src bevy_common_systems_macros/src examples`
on the base tree, rule-1 header). Cleaning those is a separate task. Rule 2
alone reads 0 hits under `examples/`, so nothing is lost by scoping the gate to
`src bevy_common_systems_macros/src`; the DoD was corrected rather than the
scope quietly widened.

### Evidence

- `./scripts/test-check-comment-tags.sh` -> exit 0, "all probes pass". Probed
  the prober: with rule 2's reporting branch disabled it exits 1 on 2 probes.
- `./scripts/check-comment-tags.sh scripts/fixtures/comment_tags/violating.rs`
  -> exit 1, 4 literals (0.35, 4.49, 4.51, 12), rule-2 header only.
- `./scripts/check-comment-tags.sh scripts/fixtures/comment_tags/compliant.rs`
  -> exit 0.
- `./scripts/check-comment-tags.sh src bevy_common_systems_macros/src` -> exit
  1 with 8 literals across 6 test fns in 2 files on the base tree; exit 0 after
  the fixes.
- `nix develop --command cargo fmt --check` -> exit 0.
- `... cargo clippy --all-targets` and `... --features debug` -> exit 0, only
  the expected `proc-macro-error2` future-incompat note.
- `... cargo test` -> 148 passed (lib), 59 passed 1 ignored (doc), 0 failed.
- `... cargo test --features debug` -> 155 passed (lib), 66 passed 1 ignored
  (doc), 0 failed.
- `./scripts/check-ascii.sh` -> exit 0.
- `nix run nixpkgs#actionlint -- .github/workflows/ci.yml` -> exit 0.

### Round 2 (after review round 1)

One BLOCKER, one MAJOR and one MINOR, all in the region parser, all fixed with
fixture coverage. Fixing the BLOCKER's stated cause was not enough: the canary
that exposed it -- a violation injected into `src/modding/registry.rs`'s real
test module -- stayed green through three fixes, surfacing a new desync cause
each time. Four in total, every one silent. The class now fails loud: an
unbalanced file is exit 2, not a clean run. See `NOTES.md`.

Evidence, re-run at write time:

- `./scripts/test-check-comment-tags.sh` -> exit 0. Reverting the lifetime fix,
  the raw-string cross-line state, or the balance guard each turns it red.
- canary (`src/modding/registry.rs` + an injected violation) -> exit 1, reports
  the injected literal. It was exit 0 at round 1.
- `/tmp/probe4.rs` (public rustdoc under a latched `#[cfg(test)] use`) -> exit
  0. It was exit 1 at round 1.
- `./scripts/check-comment-tags.sh src bevy_common_systems_macros/src` -> exit
  0, and no desync line anywhere in `src`, the macros crate or `examples`.
- `./scripts/check-ascii.sh` -> exit 0.
- No Rust source changed this round (`git diff --name-only 4e06cc6 -- '*.rs'
  ':!scripts'` is empty), so the cargo results above still stand.

### Reflection

The task body asserted the checker "already parses `#[cfg(test)]` regions"; it
did not, and the parsing was the bulk of the work. Reading the script before
trusting the plan's premise is what caught it. The measurement-first pass was
the load-bearing step: the DoD's own definition of an unexplained literal was
unshippable at 183 hits, and only running three candidate definitions over the
tree made the tuned one defensible rather than a guess.
