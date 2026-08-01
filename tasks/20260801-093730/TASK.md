# check-comment-tags: flag a /// on a test fn guarding an unexplained literal

- STATUS: OPEN
- PRIORITY: 40
- TAGS: chore,tooling,lessons
- KIND: TASK
- FLOW STEP: BACKLOG
- PLAN STATUS: DRAFT

Promotion of the ledger lesson `state-what-the-checker-cannot-see` (x3,
disposition taken 2026-08-01 during the close of epic 20260731-172116).

The recurring shape: a `///` doc comment on a test fn inside `#[cfg(test)]` is
the correct home for what the test PROVES, but rustdoc never renders it and
`check-comment-tags.sh` exempts it. Three consecutive authors used it correctly
in the main and then, once per outing, misused it to move a VALUE guard out of
the test body -- where the tag rule would have demanded a `NOTE:`.

Prose cannot see this; the checker can. `check-comment-tags.sh` already parses
`#[cfg(test)]` regions.

## Done Means

- cmd: `./scripts/check-comment-tags.sh` flags a `///` on a test fn whose body
  contains an unexplained numeric literal (a literal with no tagged `NOTE:`
  block and no end-of-line comment on its line)
- cmd: the rule has a fixture proving both directions -- a violating test fn is
  reported, a compliant one is not
- cmd: `./scripts/check-comment-tags.sh src bevy_common_systems_macros/src`
  still exits 0 on the current tree, or every new hit is fixed in the same
  change
- cmd: `cargo test` and `cargo test --features debug` pass
- manual: the rule's false-positive rate is acceptable on the current tree --
  if it is not, tune the "unexplained literal" definition rather than shipping
  a noisy check

## Notes

- Prose alternative, rejected in favour of the checker: one AGENTS.md line
  stating a `///` may carry what a test PROVES and never why a literal has its
  value.
- Origin occurrences: 20260731-172224, 20260731-172232, 20260731-172233.
