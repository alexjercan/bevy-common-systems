# Retro: harvest 13_glide UI juice into ui/animate

- TASK: 20260705-090557
- BRANCH: spike/glide-ui-juice-harvest (squash-merged to master)
- REVIEW ROUNDS: 1 (APPROVE, four informational notes)

A harvest-after-proof spike: 13_glide had shipped with its UI juice inline, and
this task decided which patterns become crate API. Three candidates in, two
promoted, one kept local -- a clean split.

## What went well

- Let the concrete reference draw the line. Because 13_glide already existed and
  worked, "does this generalize" was answerable by reading real code, not
  guessing. The slide/pop/flash appliers were literally 3-line systems keyed off
  a `Tween<T>` -- obviously reusable -- while the score readout was tangled with
  the game's `Score` resource and text format -- obviously not. The proof-first
  ordering the task insisted on paid off.
- Chose opt-in markers over an auto-applier and could say why. The `tween` module
  is deliberately target-agnostic (its doc has the game write the applier), so a
  crate plugin that grabbed every `Tween<Vec2>` and forced it into `Node.left/top`
  would over-reach. Markers keep tween generic and match the crate's other opt-in
  shapes (`feedback`'s `Flash`, status items). This kept the harvest additive, not
  a behaviour change to tween.
- Deleted more than I added at the call site. The refactor removed ~45 lines from
  13_glide (three appliers, `flash_tween`, two colour helpers) AND a now-dead
  `TileFace` marker whose only purpose was to scope those appliers. Noticing the
  marker went dead -- rather than leaving it -- is the kind of cleanup a harvest
  should include.
- Verified the visual layer the reliable way. `scrot` grabbed the terminal (the
  known-unreliable X-root path), so I switched to `ScreenshotPlugin` (the app
  framebuffer) and got a clean board render confirming the appliers drive the Node
  fields -- exactly the "lean on the framebuffer capture, not scrot" lesson the
  glide-example retro had just recorded.

## What went wrong

- Wrote a `node_flash` test that called `Tween::advance`, which is private to the
  tween module, so it did not compile from `ui/animate`'s test. Root cause: I
  assumed the tween's tick was public without checking. Rewrote the test to assert
  the start-white convention (the part I can observe without advancing) and left
  the end value to 13_glide's integration. Minor, caught at first `cargo build
  --lib`, but a reminder to check the API surface of a sibling module before
  leaning on it in a test.

## What to improve next time

- When a harvested helper wraps another module's type, check which of that type's
  methods are `pub` before writing tests against them. If the useful observation
  (here: the tweened end value) needs a private method, either test the
  observable part or reach for an integration test with the driving plugin --
  do not assume `advance`/`tick`-style methods are public.

## Action items

- [x] `ui/animate` shipped (markers + plugin + colour helpers + `node_flash`);
  13_glide refactored onto it; rolling-number kept local with reasoning.
- No follow-up seeded: the rolling-number readout waits for a second concrete
  user before a `RollingNumber` type is worth extracting (the same two-user rule
  the bastion-catalog and Wave 2 harvests applied).
