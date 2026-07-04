# persist: cross-platform save/load resource (Wave 2)

- STATUS: OPEN
- PRIORITY: 28
- TAGS: spike,feature,persist

> Spike: docs/spikes/20260704-134035-game-juice-and-scaffolding-kit.md (read
> first). Wave 2 -- the one primitive every game lacks.

## Goal

Add a `persist` primitive that serializes a `Resource` to durable storage and
loads it on startup, so high scores and settings survive a restart. Today
every game's "high score" is a plain in-memory `Resource` that resets on
launch (e.g. `examples/06_fruitninja.rs` `struct HighScore(usize)`), because
the crate has no persistence and nobody wants to hand-roll native+wasm storage
per game.

Must work on both targets: a project directory via serde/ron on native, and
`localStorage` on wasm (mirror how the wasm-web build already handled the
`getrandom` split, see `docs/wasm-web-builds.md`). RESOLVE FIRST (spike open
question): build a thin wrapper vs depend on `bevy-persistent`; a 30-minute
follow-up spike or a user call before implementing. Prove it by persisting one
game's high score across two launches.
