# Add non-ASCII guard (pre-commit or CI) enforcing plain-ASCII rule

- STATUS: OPEN
- PRIORITY: 10
- TAGS: chore

## Goal

AGENTS.md mandates plain ASCII in all code, comments and docs, but nothing
enforces it - non-ASCII typographic chars sat in chase.rs from the initial
commit until a grep happened to find them (20260703-101712). Add a
mechanical guard so the rule is enforced, not remembered.

## Steps

- [ ] Decide the mechanism: a lightweight CI step is preferred over a local
      pre-commit hook (hooks are not shared via the repo and this project
      has no hook setup today). If CI config does not yet exist, a small
      shell check is enough.
- [ ] Add a check that fails when `grep -rnP '[^\x00-\x7F]'` matches under
      `src/`, `bevy_common_systems_macros/src/` and `examples/`. It should
      print the offending file:line so the fix is obvious.
- [ ] Confirm it passes on the current tree (which is clean as of
      20260703-101712) and fails on a deliberately introduced en-dash.
- [ ] Document the check in AGENTS.md under the build/verify commands so
      agents can run it locally.

## Notes

- Discovered during 20260703-101712; see
  docs/retros/20260703-101712-chase-ascii.md.
- Scope note: the repo currently has no CI workflow file. Adding one is a
  reasonable part of this task, but if the user prefers a different
  enforcement point (e.g. a justfile/make target or a rustfmt-adjacent
  tool), adjust accordingly. Not a dependency of anything.
