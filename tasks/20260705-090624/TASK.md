# Build examples/13_glide -- UI-forward slide-merge (2048-style) puzzle

- STATUS: CLOSED
- PRIORITY: 80
- TAGS: spike,feature,example,ui

## Goal

Add `examples/13_glide`: a UI-forward slide-merge (2048-style) puzzle. It is the
canonical headline demo of `tween` (tile slide/pop animations on Bevy UI nodes)
and of `persist` + `scoring/high_score` (a saved best score, which a 2048 lives
on). 4x4 board, swipe + arrow-key input, standard 2048 rules,
menu/playing/game-over states, `ui/popup` / `ui/menu` / `SfxPlugin`, `Camera2d`,
wasm/trunk build -- following the `06_fruitninja` shape.

Per user steer: treat UI quality as a first-class goal and produce reusable,
copy-pastable UI patterns. Core structure: a static responsive CSS-grid `Node`
underlay for the cells + a separate absolutely-positioned tile layer whose
`left`/`top` is driven from a `Tween<Vec2>`. Ship UI-juice helpers game-local
(node color flash / pop, animated number) -- do NOT promote to the crate here;
that is the follow-up.

## Steps

- [x] **Scaffold `examples/13_glide.rs`** following the `11_overload`/`09_reactor`
  skeleton: `//!` doc header (game + controls + `cargo run --example 13_glide`),
  clap `Cli` parsed first in `main`, `Window` with wasm `canvas: Some("#game-canvas")`
  + `fit_canvas_to_parent`, `DefaultPlugins.set(WindowPlugin{..})`,
  `PhysicsPlugins::default()`, `ClearColor`, `GameState { #[default] Menu, Playing,
  GameOver }` + `init_state`, `Startup` `setup` spawning `Camera2d` and loading the
  `SoundBank<Sfx>`. Add crate plugins: `SfxPlugin`, `TweenPlugin`, `PopupPlugin`,
  `UnifiedPointerPlugin`, `MenuPlugin`, `PersistPlugin::<HighScore<u32>>::new("13_glide.high_score")`,
  `FrameTimeDiagnosticsPlugin` (guarded). Boots to an empty Menu state.
- [x] **Board model + geometry constants.** A `Board` resource: `grid: [[u32; 4]; 4]`
  (0 = empty) for logic plus `tiles: [[Option<Entity>; 4]; 4]` mapping cells to tile
  entities. Constants: `BOARD_N = 4`, cell size / gap / board origin in px, and
  `cell_to_px(row, col) -> Vec2` + `tile_color(value) -> Color` + `text_color(value)`
  helpers. `Score(u32)` resource; `init_resource::<HighScore<u32>>()`.
- [x] **Pure move logic + unit tests.** `fn slide_merge_line(line: [u32;4]) -> ([u32;4],
  u32 /*gained*/)` (compact toward index 0, merge equal neighbours once, return gained
  score) and `fn apply_move(grid, Direction) -> (new_grid, Vec<TileMove{from,to,merged}>,
  gained, changed: bool)`. Add `#[cfg(test)]` tests for slide/merge edge cases (empty
  line, single, `[2,2,2,2]->[4,4]`, no-merge-twice `[4,4,8,0]`, full-no-move) per the
  crate's pure-logic testing convention. Direction enum `{Up, Down, Left, Right}`.
- [x] **Board rendering.** On `OnEnter(Playing)`: reset `Board`, spawn a static
  responsive CSS-grid `Node` (`display: Display::Grid`, percentage tracks) as the
  16-cell underlay, and a separate `position_type: Absolute` tile-layer container.
  Spawn a tile = Absolute `Node` at `cell_to_px` with `BackgroundColor(tile_color)`,
  `border_radius`, and a centered `Text` child. Seed two starting tiles. Tag overlays
  `DespawnOnExit(GameState::Playing)`.
- [x] **Input -> Move intent.** Detect a swipe from `UnifiedPointer` (it exposes only
  `screen_pos` / `pressed` / `just_pressed`, so keep a `SwipeTracker` resource: store
  origin on `just_pressed`, keep `prev_pressed`, and on the pressed->released edge
  compute `delta`, dominant axis, sign, gated on a min magnitude) plus arrow keys /
  WASD. Emit a `MoveIntent(Direction)` event; ignore input while `MoveAnim` is active.
- [x] **Apply move + slide animation (tween headline).** On a `MoveIntent`, run
  `apply_move`; if `changed`, for each `TileMove` attach `Tween<Vec2>` (start = current
  px, end = `cell_to_px(to)`, `MOVE_DURATION`, `EaseFunction::QuadraticOut`) to the tile
  entity and start a `MoveAnim` timer. A system after `TweenSystems::Advance` writes
  `node.left/top = Px(tween.value())`. Merged-source tiles slide onto their target then
  get resolved next step.
- [x] **Resolve after slide.** When `MoveAnim` elapses: despawn merged-source tiles,
  bump the merged target tile's value (+ pop), spawn the new random `2`/`4` tile (spawn
  pop), add `gained` to `Score`, spawn a `ui/popup` "+N" at the merged cell's screen px,
  play merge/spawn `SfxPlugin` one-shots, and clear `MoveAnim` to re-enable input.
- [x] **UI-juice helpers (game-local, the copy-pastable patterns).** (a) tile **pop**
  on spawn/merge; FIRST resolve whether a UI `Node` honors `Transform` scale in Bevy
  0.19 with a `ScreenshotPlugin` grab -- if yes, tween `Transform::scale`; if not, tween
  `Node` `width`/`height` (+ font) or a wrapping layer; document the choice. (b) merge
  **color flash** via `Tween<Vec4>` applied to the tile `BackgroundColor`. (c) **animated
  number**: the score readout rolls old->new via a `Tween<f32>` each change. Keep these
  as small local systems/helpers -- do NOT promote to the crate (that is task
  20260705-090557).
- [x] **HUD.** A top bar showing current `Score` (animated number) and best
  (`HighScore::best()`), plus a small "swipe / arrows" hint. Plain `Node` bar or
  `ui/status`; keep it responsive at phone width.
- [x] **Game over + high score.** Detect no legal move (grid full AND no equal adjacent
  pair) -> `NextState(GameOver)`. `OnEnter(GameOver)`: `record_high_score`
  (`HighScore::record(score)`) chained BEFORE `spawn_game_over` so the overlay can read
  `is_new_best()`; overlay via `centered_screen()` + `screen_text(...)` showing final
  score, "New best!" when `is_new_best()` else the best line; play game-over sfx. Dismiss
  on tap / any key / touch -> `Menu`.
- [x] **Menu.** `OnEnter(Menu)` spawn overlay: title (`screen_text` + `TitlePulse` via
  `MenuPlugin`), best-score line, "tap / swipe / arrows to play". Start on tap / any key
  / touch -> `Playing`.
- [x] **Harness wiring** (behind `#[cfg(feature = "debug")]`): `InspectorDebugPlugin`;
  `AutopilotPlugin::new().hold(Menu, 0.6).hold(Playing, 3.0).hold(GameOver, 0.8)` with an
  `.input(|world, elapsed| ...)` closure that, while in `Playing`, presses arrow keys to
  drive real moves (use `ButtonInput` reset_all + press, per the verify-input-with-
  autopilot lesson); and `ScreenshotPlugin::new(Playing).settle_frames(30)`.
- [x] **Verify (full gate).** `cargo fmt`, `cargo clippy --all-targets` and
  `--features debug` (clean), `cargo test` (the pure-logic unit tests), `cargo test
  --examples`, `scripts/check-ascii.sh`. Boot under `timeout` to confirm the render loop.
  `BCS_AUTOPILOT=1 ... --features debug` under `timeout` -> confirm each `autopilot: ->
  State` transition and `autopilot: cycle complete, no panic`. `BCS_SHOT=390x844` grab
  to confirm the board is responsive and readable at phone width (reactor mobile lesson).
  Verify persistence: run once with `BCS_PERSIST_DIR` set, confirm best is reloaded.
- [x] **Wasm/web registration.** Append `"13_glide web/games/13_glide"` to the `games`
  array in `web/scripts/build-games.sh`; add `web/games/13_glide/index.html` (copy
  `09_reactor`'s, retitle, keep the `#game-canvas` + audio-unlock + `copy-dir` sounds
  links); add a `13_glide` entry to `GAMES` in `web/src/games.ts`. Run `npm ci` then
  `npm run build` in `web/` if the devshell allows (fresh worktree has no node_modules).
- [x] **Docs.** Add `tasks/20260705-090624/NOTES.md` (design decisions: tween-on-UI-
  node pattern, the static-grid-underlay + absolute-tween-layer split, the UI-pop
  resolution, move/merge animation coordination). Update the `AGENTS.md` example list
  and module-map coverage to mention `13_glide` as the `tween` / `persist` headline.

## Notes

Spike: tasks/20260705-090421/SPIKE.md

Kept as ONE task (single cohesive example file); the UI-pattern harvest is the
separate follow-up 20260705-090557.

### API facts (verified, so the implementer does not re-search)

- Examples are **auto-discovered** -- no `Cargo.toml` `[[example]]` entry needed;
  `.cargo/config.toml` already sets the wasm rustflags. `Cargo.toml` already enables
  the bevy `wav` feature for examples.
- All items via `use bevy_common_systems::prelude::*;` (harness via
  `bevy_common_systems::debug::harness::prelude::*`).
- `tween`: `Tween::new(start, end, dur, EaseFunction)` (+ `.with_on_complete(Keep|
  Remove|Despawn)`), read `Tween::value()` in a system `.after(TweenSystems::Advance)`;
  `TweenValue` impl for `f32/Vec2/Vec3/Vec4` (color = linear-RGBA `Vec4`); `TweenFinished`
  marker is per-entity, so keep one tween per entity when a completion observer must be
  unambiguous. No built-in `Node` adapter -- apply the value to `Node`/`Transform`
  yourself.
- `persist`: `PersistPlugin::<T>::new(key)`, `T: Resource + Serialize +
  DeserializeOwned + Default`; loads synchronously in `Plugin::build`, auto-saves on
  `resource_changed`. Native JSON under `dirs::data_dir()` or `$BCS_PERSIST_DIR`; wasm
  `localStorage`.
- `scoring/high_score`: `HighScore<T>` (`T: PartialOrd + Copy`); `record(score) -> bool`
  (strict `>`), `best()`, `is_new_best()`, `clear_new_best()`. `new_best` is
  `#[serde(skip)]`, so `PersistPlugin::<HighScore<u32>>::new(...)` stores only `best`.
- `ui/popup`: `popup(screen_pos: Vec2, text, font_size, color)` -- position is a
  SCREEN/viewport point in logical px (becomes an absolute Node that rises + fades +
  self-despawns). Add `PopupPlugin`.
- `input/pointer`: `UnifiedPointer { screen_pos: Option<Vec2>, pressed, just_pressed }`
  (touch beats cursor). NO drag-delta/release field -- track swipe state yourself.
- `input/state`: `set_state_on_key(KeyCode, target)` as a `.run_if(in_state(..))` system.
- `audio`: `SoundBank::load(&assets, [(Sfx::Merge, "pickup"), ...])` loads
  `assets/sounds/<name>.wav`; `commands.play_sfx_volume(bank.get(key), vol)`. Available
  placeholder wavs: alarm, bomb, combo, game_over, golden, hurt, launch, level_up,
  menu_select, pickup -- reuse by semantic key (e.g. Merge=pickup, Spawn=menu_select,
  BigMerge=combo/golden, GameOver=game_over, MenuSelect=menu_select).
- `debug/harness`: `AutopilotPlugin::new().hold(state, secs)....input(|world, elapsed|)`
  (env `BCS_AUTOPILOT`); `ScreenshotPlugin::new(state).settle_frames(n).path(..)` (env
  `BCS_SHOT="WxH"`). Mutually exclusive at runtime; wire both behind `#[cfg(feature="debug")]`.
- Overlays: `centered_screen()` + `screen_text(text, size, color)` children, tagged
  `DespawnOnExit(state)`. `TitlePulse::new(color).with_speed(..).with_alpha_range(..)`
  needs `MenuPlugin`.

### Open decisions (resolve during impl, do not guess silently)

- **UI pop scaling** -- does a Bevy 0.19 UI `Node` honor `Transform` scale? Settle with
  a `ScreenshotPlugin` grab before committing to the pop approach.
- **Move/merge coordination** -- primary plan uses a fixed `MOVE_DURATION` timer then a
  resolve system (deterministic, testable); the `On<Add, TweenFinished>` observer is the
  alternative if the timer feels off.
- **New-tile spawn** -- uniform random empty cell (seed via `rand`); spawn only after the
  slide resolves, never mid-animation.

### Reuse / lessons applied

- Copy the visual layer (font_size `FontSize::Px`, `TextLayout` struct literal,
  `border_radius` on `Node`) from `11_overload`/`09_reactor`, not from memory
  (bevy-0.19 idiom gotcha).
- `cargo clippy --all-targets` is the real compile gate (bare `cargo build` skips
  examples).
- Responsive grid = percentage tracks, NOT fixed px + `flex_wrap` (reactor mobile lesson).
- Verify autopilot input by driving `ButtonInput`, not synthetic clicks.

### Work log (implementation)

- Implemented `examples/13_glide.rs` (~1030 lines): pure 2048 logic + 8 unit
  tests, wrapper/face tile UI, slide/pop/flash tweens, swipe + arrow input,
  rolling score, `ui/popup` "+N", persist + high score, menu/game-over, harness.
- Chose a fixed 346px centered board with absolute cell/tile layers over
  `Display::Grid` (unverified API; tiles must animate between cells anyway) and
  animate `Node`/`BackgroundColor` fields rather than `Transform` scale (UI owns
  the transform). Both documented in `tasks/20260705-090624/NOTES.md`.
- Bug fixed: tween-completion vs despawn race (merged-away tiles despawn exactly
  when their slide tween completes). Fix: order `tick_move_anim`
  `.before(TweenSystems::Advance)` + `TweenOnComplete::Keep` on all tweens.
- Verified: `cargo clippy --all-targets` clean (plain and `--features debug`);
  8 unit tests pass; `check-ascii` + `cargo fmt --check` clean; headless
  `BCS_AUTOPILOT` run reaches Menu->Playing->GameOver with "cycle complete, no
  panic" and zero runtime errors; `BCS_SHOT=390x844` confirms the board is
  responsive and readable at phone width and that the persisted best ("Best: 84")
  reloads. Web build registered (build-games.sh + index.html + games.ts) and a
  single-game `trunk build --example 13_glide` verified the wasm compile.
