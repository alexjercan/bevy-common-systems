# Retro: ship game audio assets into the wasm web build

- TASK: 20260703-163328 (CLOSED, review APPROVE round 1)
- BRANCH: feature/web-audio
- SCOPE: make 06_fruitninja's SFX load in the browser build by staging
  `assets/sounds/` into the trunk dist.

## What went well

- **Investigation before code.** A read-only Explore agent mapped the terrain
  first (trunk directives, build-games.sh, webpack copy, the audio module, the
  autoplay risk) and correctly isolated the dominant break -- assets never
  copied -- from the secondary risk (autoplay). The fix was then a one-line
  directive, not a hunt.
- **Applied the prior retro directly.** `tasks/20260703-000001/NOTES.md`
  had two lessons that paid off immediately: (1) never `| tail` a build whose
  exit code matters -- logged to a file and checked `$?`; (2) verify through
  the real entry point. That second lesson turned into actually running
  `npm run build:web` (not just `build-games.sh`) and confirming the sounds
  reach `dist/`, which is the artifact that is actually served. Trusting the
  staging dir alone would have repeated the exact class of miss that shipped a
  broken build last time.

## What was tricky

- **trunk copy-dir target-path nesting was ambiguous** from the docs: would
  `href=".../assets/sounds"` + `data-target-path="assets/sounds"` produce
  `assets/sounds/*.wav` or `assets/sounds/sounds/*.wav`? Resolved by building
  and listing the output rather than guessing -- trunk copies the *contents* of
  the href dir into the target-path. Now captured in `docs/wasm-web-builds.md`
  so the next game does not re-derive it.

## What to do differently / notes for future sessions

- The wasm release build is a multi-minute compile; kick it off in the
  background early and do docs/other work while it runs, rather than blocking.
  Do NOT run a competing native `cargo build` at the same time -- CPU
  contention made the native build time out at 2 min (it was redundant anyway,
  since this task changed no Rust).
- copy-dir also ships `assets/sounds/README.md` into the dist. Left as-is
  (harmless, keeps the directive a one-liner); flagged as a NIT in the review.
  If a game ever ships assets whose directory holds files you do NOT want
  published, prefer per-file `copy-file` directives.
- Deferred correctly: browser *audibility* + the autoplay gesture policy are
  task 20260703-163329, not this one. Splitting "assets load" from "sound is
  audible" kept this task fully verifiable without a graphical session.
