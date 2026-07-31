# Fix clippy manual_contains in completion.rs

- STATUS: OPEN
- PRIORITY: 30
- TAGS: chore,lint
- KIND: TASK
- FLOW STEP: BACKLOG
- PLAN STATUS: DRAFT

`cargo clippy --all-targets` emits `clippy::manual_contains` at
`src/completion.rs:88`:

```
self.pending.iter().any(|p| *p == name)
   help: try: `self.pending.contains(&name)`
```

Pre-existing on `master` (confirmed at 8500161), not introduced by the KISS
epic. Surfaced while verifying task 20260731-172208, whose scope covers
`completion.rs` but only its comments -- folding a lint fix into a
comment-hygiene pass would have muddied that diff.

Exit code is 0 (the lint is warn-level), so no `cmd:` proof in the epic
currently fails; this is about keeping `clippy` output actually clean, which
AGENTS.md asks for.

Note `is_pending` at the same site takes `&str` while `pending` holds
`&'static str`, so check the suggested `contains(&name)` actually compiles
before assuming it is a one-word change.

## Steps

- [ ] Apply the clippy suggestion at `src/completion.rs:88`, adjusting for the
      `&str` / `&'static str` mismatch if it does not compile as suggested.
- [ ] Check `others_pending` (same file) for the same pattern.
- [ ] Re-run the suite.

## Definition of Done

- No `manual_contains` warning in either feature configuration (cmd: `nix develop --command cargo clippy --all-targets 2>&1 | grep -c manual_contains` -> 0, and the same with `--features debug`).
- Behavior unchanged (cmd: `nix develop --command cargo test` and `... cargo test --features debug` pass).
