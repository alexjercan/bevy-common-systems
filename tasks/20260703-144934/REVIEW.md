# Review: enable wasm example builds + trunk build of fruit ninja

- TASK: 20260703-144934
- BRANCH: web-showcase (sprout worktree)

## Round 1

- VERDICT: APPROVE

Self-review. The getrandom fix is scoped correctly: the dependency is
`[target.'cfg(target_arch = "wasm32")'.dependencies]` and the rustflag is under
`[target.wasm32-unknown-unknown]`, so native builds are untouched (only a
feature toggle on an existing transitive dep). Verified: native `cargo test`
path unaffected; wasm example compiles (exit 0, captured directly - not through
`tail`); trunk emits a working release dist (index.html + JS + 42 MB wasm-opt'd
wasm). Build script stages to web/build/games so webpack's dist clean can't wipe
it, and is PUBLIC_PATH-aware. 04_status_item excluded (shells out). Browser smoke
run is the user's; artifacts confirmed present and non-trivial.

- Note: 42 MB wasm is heavy (Bevy) but expected; acceptable for a showcase.
