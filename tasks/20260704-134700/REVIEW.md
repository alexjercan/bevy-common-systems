# Review: persist - cross-platform PersistPlugin<T>

- TASK: 20260704-134700
- BRANCH: feat/persist

## Round 1

- VERDICT: APPROVE

Delivers the "one primitive every game lacks" cleanly, and the build-vs-depend
fork is resolved the right way (hand-roll): serialization reuses the existing
serde/serde_json, the wasm plumbing follows the crate's own getrandom precedent,
and a game lifting the module pulls only small target-gated deps -- true to the
copy-pastable charter, where `bevy-persistent` would bind the crate to a specific
bevy version.

The load-in-`build` decision is the highlight. Booting 06 caught a real panic --
`spawn_menu` (`OnEnter(Menu)`) reads `Res<HighScore>` before a `PreStartup` load
system would have inserted it -- and moving the load into `Plugin::build`
(one small synchronous file read) fixes it correctly: the resource now exists
before any system or state transition. This is exactly the "an example is not
done until it has actually been run" rule paying off.

Design/correctness:
- Backend is cleanly cfg-gated behind a shared `load(key)`/`save(key,data)` pair;
  native uses `dirs` + `serde_json` + `std::fs`, wasm uses `web-sys` localStorage.
  Both handle the "storage unavailable" path gracefully (return `None` / warn),
  and `load` falls back to `Default` on a missing or unparseable blob.
- The `$BCS_PERSIST_DIR` override is a genuinely useful escape hatch (portable
  installs, sandboxes) that also makes the plugin hermetically testable.

Tests are meaningful and, notably, cover the cross-platform surface as far as is
possible here: a native save/load round-trip (hermetic temp dir via `save_in`/
`load_in`), a key->path mapping check, and -- the task's actual proof -- a
two-`App`-run test where a fresh app loads the value the first one saved. The
wasm backend is compile-checked on `wasm32-unknown-unknown` (verified; can't run
a browser headlessly). 06 persists its `HighScore` and boots with no panic.

Verified in the worktree: `cargo fmt --check`, `cargo clippy --all-targets`
(default and `--features debug`), `cargo test` (79 unit + 39 doctests),
`cargo check --target wasm32-unknown-unknown` and `scripts/check-ascii.sh` all
pass.

- [x] R1.1 (MINOR) src/persist/backend.rs:33 - `path_in` joins `"{key}.json"`
  without sanitizing the key, so a key containing `/` or `..` escapes the
  namespace directory (`path_in(root, "../evil")` -> `root/../evil.json`). The
  doc says "filename-safe key" but nothing enforces it. Keys are developer
  constants today (low risk), but a typo silently writes to the wrong place;
  reject or sanitize keys with path separators (a `debug_assert!` or stripping
  them).
  - Response: Fixed. Added `is_safe_key` (rejects empty keys and keys containing
    `/` or `\`); native `load`/`save` warn and no-op on an unsafe key. Covered by
    the pure `unsafe_keys_are_rejected` test. wasm needs no guard (localStorage
    keys are flat, no filesystem traversal).
- [x] R1.2 (NIT) src/persist/mod.rs:98 - `build` inserts the loaded resource
  (marks it added), so the first `Update` fires `save_persisted` (added counts as
  changed) and re-writes the file on every launch even when nothing changed.
  Harmless for a high score; skip the first-frame save if you want to avoid the
  redundant write.
  - Response: Accepted as-is. One tiny idempotent write per launch for an
    intentionally infrequently-changing resource; skipping it would need a
    per-type "just loaded" flag that is not worth the complexity here.
- [x] R1.3 (NIT) src/persist/mod.rs:167 - `value_survives_across_two_app_runs`
  sets a process-global `BCS_PERSIST_DIR`. Safe today (it is the only test that
  exercises `root()`; the backend tests use explicit roots), but a future
  plugin-level test touching storage concurrently would race on the env var --
  note it so they serialize or share the fixture.
  - Response: Acknowledged and kept it the sole env-setting test: the new
    `unsafe_keys_are_rejected` test was deliberately written pure (no env/storage)
    with a comment saying why, so it cannot race. A note on the two-app test
    records the constraint for future tests.
