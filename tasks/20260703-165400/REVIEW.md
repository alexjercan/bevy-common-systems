# Review: 11_overload -- dashboard-survival game on the status bar

- TASK: 20260703-165400
- BRANCH: feature/11-overload

## Round 1

- VERDICT: APPROVE

No BLOCKER or MAJOR findings. The diff delivers the Goal: four coupled gauges on
the `status_bar` with a green -> amber -> red `color_fn`, vent keys, a
`HealthPlugin` lose condition with alarm sounds, difficulty ramp, and the
06_fruitninja shape (states, sounds, wasm gallery). The run is genuinely losable
(uncapped climb ramp vs. bounded vent throughput, and inattention drains the hull
in ~11 s per red gauge). The status-bar downcast machinery matches how
`src/ui/status.rs` invokes `value_fn` / `color_fn`, the reactor entity lifecycle
and `On<Add, HealthZeroMarker>` observer are correctly guarded, `HealthApplyDamage`
is dt-scaled, and every Bevy 0.19 idiom (FontSize::Px, TextLayout literal,
DespawnOnExit, Camera2d-only UI) matches the reference examples. Full check suite
verified independently: build, clippy `--all-targets` (+ `--features debug`), fmt,
ascii, `cargo test` (lib 20 + doctests 12 + example 7), a clean native boot to the
render loop, and a scoped release `trunk` wasm build with sounds staged.

The findings below are all NIT-level and would not block a merge; a few are cheap
honesty/quality wins worth taking now.

- [x] R1.1 (MINOR) `examples/11_overload.rs` `vent_input` + the TASK note --
  the vent arithmetic (subtract `VENT_AMOUNT`, add `COUPLING` to the partner,
  clamp) is untested. The `coupling_forms_a_cycle_touching_every_gauge` test only
  asserts the *static* `GAUGES[].couples_to` graph is a self-free cycle, yet the
  TASK note claims "a unit test pins the coupling ... so no future edit
  reintroduces a free vent" -- which overstates what is covered (a regression that
  dropped the `+ COUPLING` term or mis-clamped would still pass). Extract the vent
  math into a pure helper and unit-test it, so the note is honest.
  - Response: Fixed in the round-1 fixup commit. Extracted `apply_vent(gauges, i)` (pure, used by
    `vent_input`) and added `vent_lowers_gauge_and_raises_its_partner` +
    `vent_and_coupling_clamp_at_bounds` tests exercising the subtract / couple /
    clamp behavior. Reworded the TASK note to say the tests pin the vent math and
    the cycle.

- [x] R1.2 (NIT) `examples/11_overload.rs` `GaugeReading` / `HullReading` -- the
  two newtypes have byte-identical `Display` impls. Defensible as two named types
  for gauge-vs-hull intent, but the duplicated `fmt` body can drift. Optional:
  collapse to one `Reading(f32)` with the two color closures.
  - Response: Kept two types (they document gauge-vs-hull semantics and downcast
    distinctly) but removed the duplication by giving both a shared
    `fmt_percent()` helper, and added a `HullReading` Display test. The fmt body
    now lives in one place.

- [x] R1.3 (NIT) `examples/11_overload.rs` `ReactorState::reset` -- the comment
  says gauges "start scattered around amber's lower edge so the console already
  needs attention," but the range is `18.0..40.0`, comfortably below `AMBER =
  60.0`; the console actually starts green. Fix the comment (or the range).
  - Response: Fixed in the round-1 fixup commit. Reworded the comment to say the gauges start in
    the green but already climbing, which is what `18.0..40.0` does.

- [x] R1.4 (NIT) `examples/11_overload.rs` `setup` -- the status bar is spawned in
  `Startup` and never despawned, so it renders (reading the default `ReactorState`:
  gauges 0, hull 100, level 1, time 0) behind the menu and meltdown overlays.
  Plausibly intended ("the game is the status bar, always visible") but worth a
  deliberate call rather than an accident.
  - Response: Intended -- the whole conceit is that the console is always on. Made
    it deliberate: added a comment at the status-bar spawn, and `reset` now runs
    once at startup so the idle bar shows a plausibly-scattered console instead of
    all-zero gauges.

- [ ] R1.5 (NIT, informational) `examples/11_overload.rs` `spawn_game_over` --
  `new_best` is evaluated after `record_high_score` has already raised `high.0`, so
  any tie (including the first run) prints "New best!". Harmless and arguably
  correct (ties count as bests); flagged only in case exact-tie wording matters.
  - Response: Left as-is by design -- treating the first run and ties as a "new
    best" is the intended, friendlier wording.
