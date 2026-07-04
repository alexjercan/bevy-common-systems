# assets: sound/handle registry + opt-in ready-gate (Wave 2)

- STATUS: CLOSED
- PRIORITY: 55
- TAGS: spike,feature

> Spike: docs/spikes/20260704-175058-dev-harness-and-app-scaffolding.md (read
> first). Wave 2 -- clean leaf harvest, fits beside `audio`.

## Goal

Replace the hand-rolled `SfxAssets` handle-bag that all six games (06-11) copy
with a reusable named-handle registry. Every game declares a flat struct of
named `Handle<AudioSource>` fields loaded inline in `setup` (06:558, 08:439,
09:514, 11:308), sharing the same placeholder `.wav` files and the same
file-path strings. Provide a sound/handle registry (an enum or key -> path map
+ loader) that deduplicates the struct and the paths, plus an OPTIONAL
`AssetsReady` state-gate for games that want to defer `Menu` until handles load
(today every game boots straight to `Menu` and gets away with no gate, so the
gate must be opt-in, not mandatory -- see the spike's open question).

Prove it by refactoring a couple of games onto the registry. This task is
stepless on purpose (spike output); run /plan to break it into steps before
/work.
