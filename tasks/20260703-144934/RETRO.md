# Retro: enable wasm example builds + trunk build

- TASK: 20260703-144934 (committed on web-showcase, 1 self-review round, APPROVE)

## What went well
- De-risked the riskiest unknown first: kicked off a real wasm compile before
  planning the site, which surfaced the getrandom-0.3-needs-wasm_js failure
  immediately instead of after building a whole gallery around a broken build.
- Verified trunk's example support and the full pipeline with a fast DEV trunk
  build (reused cache, ~seconds) before committing to the slow release build -
  cheap validation of the invocation shape.
- Guarded the native build after a Cargo.toml/.cargo/config change (cargo build
  exit 0) so a wasm-only fix couldn't silently break native.

## What went wrong
- First background "de-risk" build reported exit 0 while actually failing,
  because I piped `cargo build ... | tail` and got tail's exit code. Re-ran
  capturing the real code. Lesson: never judge a build by a piped tail's exit
  status.
- Chose the games output dir (dist/games) before realizing webpack's dist clean
  would wipe it; switched to a web/build staging dir that webpack copies from.
  Caught while planning task 2's integration, not after a clobbered build.

## Improve next time
- For any build whose exit code matters, redirect to a file and check `$?`
  directly; reserve `| tail` for interactive peeking only.
