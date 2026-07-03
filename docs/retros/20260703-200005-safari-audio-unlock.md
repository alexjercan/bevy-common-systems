# Retro: Fix mobile Safari silent audio in 06_fruitninja web build

- TASK: 20260703-200005
- BRANCH: fix/safari-audio-unlock
- REVIEW ROUNDS: 1 (APPROVE)

See tasks/20260703-200005/TASK.md for what changed and why. This is about how
the cycle went.

## What went well

- **Read the prior decision before re-deciding.** The retro for the earlier
  audio task (20260703-163329) concluded "no unlock code needed." Rather than
  treating that as settled, the investigation asked *for which browsers* it was
  true. It was true only for Chrome/Firefox (they auto-resume a suspended
  AudioContext on `start()`); the reported failure was mobile Safari, which
  WebKit does not. The old decision was not wrong -- it was scoped narrower than
  its wording. Catching that scoping was the whole diagnosis.
- **Verified the risky mechanism against the emitted artifact, not the source.**
  The shim only works if it runs before Bevy constructs its AudioContext. That
  claim was checked by reading the BUILT `dist` HTML (shim at line ~27, trunk's
  deferred `type="module"` loader at line 86), not by assuming trunk's injection
  point. Same discipline the crate already applies to "verify through the real
  entry point."
- **Standard pattern, not invention.** The constructor-wrap + resume-on-gesture
  + silent-buffer-kick is the well-worn Web Audio unlock; picking it over a
  bespoke scheme kept the diff small and the review to one round.

## What went wrong

- **First `npm run build` half-failed on a fresh worktree.** `build:games`
  (trunk) succeeded, but `build:web` died with `webpack: command not found`
  (exit 127): a sprout worktree starts with no `node_modules` (git-ignored, not
  copied from the main checkout). Root cause: assumed the worktree inherited the
  main checkout's installed deps. Fixed by running `npm ci` in the worktree
  first. Cost was one wasted build cycle, not a wrong result -- the piped-exit
  discipline (`WEBPACK_EXIT=$?` to a file) surfaced the 127 instead of hiding
  it behind a `| tail`.

## What to improve next time

- On any web/JS task in a fresh sprout worktree, run `npm ci` (or `nix develop
  -c bash -lc 'cd web && npm ci'`) BEFORE the first `npm run build`. Worktrees
  never carry `node_modules`.
- When documenting or deciding on browser behavior, state which engines the
  claim covers. "Browsers auto-resume" hid a WebKit exception for a whole
  cycle; "Chrome/Firefox auto-resume; WebKit needs explicit resume()" would not
  have. The doc rewrite now does this.

## Action items

- [x] docs/wasm-web-builds.md: autoplay section rewritten to be engine-specific
      and to document the shim (done in this task).
- [x] AGENTS.md Gotchas: add the "fresh worktree has no node_modules, run
      npm ci before web builds" note (proposed below / added with this retro).
- [ ] Future: when a second web game gains sound, lift the inline shim into a
      shared snippet (web/games/_shared/audio-unlock.js via trunk). Noted in the
      docs; no tatr task yet since only one game has sound.
