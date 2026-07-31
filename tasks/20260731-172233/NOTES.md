# NOTES: KISS pass over feedback/ tween/ ui/ transform/ + small modules

Design/fix record for task 20260731-172233. Fifth and last child of epic
20260731-172116, after 20260731-172208 (convention + checker), 20260731-172223
(integrity/physics), 20260731-172224 (mesh/meth/camera) and 20260731-172232
(modding/persist/macros).

## Baseline (re-run on the work branch)

`./scripts/check-comment-tags.sh src/feedback src/tween src/ui src/transform
src/audio src/health src/helpers src/input src/scoring src/time`: **exit 1,
106 untagged non-doc comment BLOCKS** across 21 files. Per file:

| File | Blocks | | File | Blocks |
| --- | --- | --- | --- | --- |
| `feedback/flash.rs` | 24 | | `time/cooldown.rs` | 3 |
| `feedback/screen_flash.rs` | 11 | | `scoring/high_score.rs` | 3 |
| `ui/touchpad.rs` | 10 | | `helpers/wasd.rs` | 3 |
| `tween/mod.rs` | 8 | | `helpers/pointer.rs` | 3 |
| `health/mod.rs` | 8 | | `audio/registry.rs` | 3 |
| `transform/point_rotation.rs` | 7 | | `ui/animate.rs` | 2 |
| `ui/popup.rs` | 4 | | `ui/status.rs` | 1 |
| `ui/health_display.rs` | 4 | | `ui/objectives.rs` | 1 |
| `transform/random_sphere_orbit.rs` | 4 | | `transform/smooth_look_rotation.rs` | 1 |
| `scoring/streak.rs` | 4 | | `helpers/temp.rs` | 1 |
| | | | `audio/mod.rs` | 1 |

14 further files in scope carry zero. (Blocks, not lines -- the Story text's
"28 / 24 / 13 / 10" counted comment LINES. Per `record-numbers-from-a-rerun`,
both units are named and the per-file breakdown is pasted so the total is
reconstructible.)

Bare tatr HUIDs in non-doc comments in scope: **0**, before and after.

Result: **106 -> 0 untagged; 37 blocks kept and tagged**, the rest dropped or
promoted to rustdoc. 24 files changed, +109 / -153 lines.

(Both numbers are post-review: round 1 added `src/material.rs` to the scope --
one more kept block, one more file -- and restored a value guard in
`ui/health_display.rs` that had been folded into a `///`. The pre-review
figures were 35 blocks over 23 files. Re-derived, not adjusted by hand:
`grep -rcE '^\s*//\s*(NOTE|FIXME|BUG|TODO):'` over the scope sums to 37, and
`git diff --cached --shortstat master -- src` reports the file/line counts.)

## Comment calls, all 106 blocks

Legend, inherited from 20260731-172224: **drop** = restates the code;
**compact** = kept, rewritten as one tagged `NOTE:` block; **doc** = kept as
intent, promoted to a `///` doc comment on the fn it explains.

### `feedback/flash.rs` (24: 8 compact, 1 doc, 15 drop)

| Line | Call | Reason |
| --- | --- | --- |
| 119 | compact | guards the conditional `TweenPlugin` add (double-add panics) |
| 195 | compact | non-obvious: removing `TweenFinished` is what re-arms the tween on a re-flash |
| 206 | drop | restates `remove::<Flash>()`; the `Flash` rustdoc already states the no-material case |
| 212 | compact | borrow hazard: `.cloned()` must end the immutable borrow before `add` |
| 273 | compact | same class: read the original channel (a Copy) before `get_mut` |
| 300, 303, 306 | drop | each restates the assert below it; `flash_mix_endpoints_and_midpoint` names the case |
| 316 | drop | restates `flash_mix_clamps_and_keeps_alpha` |
| 342 | drop | restates the `add(StandardMaterial { .. })` two lines down |
| 354 | drop | restates the `insert(Flash { color: WHITE, .. })` |
| 360 | compact | non-obvious: ONE step must flush the observer AND run animate |
| 363, 374, 381 | drop | narrate the three asserts, which all carry messages |
| 428 | compact | value guard: 5 x 100ms vs the 0.2s duration |
| 433, 448 | drop | restate asserts that carry messages |
| 473 | compact | value guard: 4 x 100ms ages elapsed to ~0.4 of 0.5 |
| 484 | doc | the test's INTENT (re-insert resets elapsed, reuses the clone) -> `///` on `reflashing_restarts_the_animation` |
| 493 | drop | restates the assert |
| 501 | compact | value guard: why 0.8 discriminates a restarted flash from an aged one |
| 519 | drop | restates `flash_without_material_is_dropped` |
| 554 | drop | restates `despawn_mid_flash_frees_clone` |

### `feedback/screen_flash.rs` (11: 8 compact, 3 drop)

| Line | Call | Reason |
| --- | --- | --- |
| 106 | compact | conditional `TweenPlugin` add |
| 139 | compact | THE hazard: a non-positive decay must hold at the peak, hence `INFINITY` |
| 158 | compact | why `remove::<TweenFinished>()` -- a stale marker leaves the re-spike landed |
| 235 | compact | value guard on `decay: 0.0` inside the spawn literal |
| 242 | compact | one step flushes the observer and runs animate |
| 245, 252 | drop | restate the two assert groups |
| 268 | compact | value guard: 250ms is half the 0.5s life (decay 2.0) |
| 281 | compact | value guard: 250 + 400ms is past the life, so despawn_on_end fires |
| 305 | compact | value guard: 10 x 100ms fully decays the 1/3s fade |
| 316 | drop | restates the re-insert below it |

### `ui/touchpad.rs` (10: 2 doc, 8 drop)

All ten blocks (246, 255, 261, 271, 291, 313, 317, 321, 325, 329) labelled a
group of asserts and all ten left the bodies: each restates the assert beneath
it, and none guards a value. The two facts the fn names do NOT carry were
folded into `///` docs instead --
`button_grid_maps_columns_and_rejects_misses` gets edge clamping and the four
degenerate misses (off-window x, zero-size window, zero-column grid, point
outside the zone); `stick_deflection_maps_and_clamps` gets the dead zone, the
0..1 mapping and the unit-disc invariant. The `///` docs are new text on the
fns, not any of the ten lines moved.

### `tween/mod.rs` (8: 5 compact, 1 doc, 2 drop)

| Line | Call | Reason |
| --- | --- | --- |
| 228 | compact | THE invariant: guard on `completed`, not `finished` |
| 238 | compact | 6 lines -> 1: `try_*` only, plain commands panic on a stale entity. The fn rustdoc already states the despawn-safety contract |
| 307 | drop | restates `easing_bends_the_value_off_the_linear_line` |
| 313 | compact | value guard: 0.4 sits between QuadraticIn(0.5) = 0.25 and linear 0.5 |
| 329 | doc | it documents the `app_with_tween` HELPER (why zero duration is deterministic) -> `///` on that fn |
| 352, 424 | compact | value guard: no `with_on_complete` because `Tween::new` defaults to Remove |
| 368 | drop | 7 lines of P100-breach provenance and race narration, duplicated by the `///` on `app_with_doomed_tween` directly below, which states the ordering, the flush interleaving and the old panic. Provenance belongs in `tasks/` |

### `health/mod.rs` (8: 1 compact, 7 drop)

| Line | Call | Reason |
| --- | --- | --- |
| 98 | drop | restates `add_observer(on_damage)` |
| 128 | compact | real invariant: this branch leaves `amount` UNCHANGED, unlike the two zero-health branches below |
| 146 | drop | restates the three lines under it; the `on_damage` rustdoc explains the clamp at length |
| 177, 185, 189, 228, 237 | drop | narrate a trigger or an assert; all three tests already carry `///` docs stating the intent |

### `transform/point_rotation.rs` (7: 1 compact, 6 doc)

| Line | Call | Reason |
| --- | --- | --- |
| 26 | doc | "identity means facing -Z, up +Y, right +X" is API surface -> promoted onto the `initial_rotation` field |
| 143 | compact | the columns are `(right, up, -forward)`; sharpened into the `from_mat3` handedness hazard from AGENTS.md |
| 159, 173, 187, 206, 220 | doc | the fn names are `test_compute_point_rotation_1..5` and carry nothing; each comment WAS the test name -> `///` on each |

### `ui/popup.rs` (4: 3 compact, 1 doc)

| Line | Call | Reason |
| --- | --- | --- |
| 95 | compact | conditional `TweenPlugin` add |
| 224 | compact | value guard: one 100ms step is mid-lifetime |
| 240 | compact | value guard: 100 + 6 x 100ms is past the lifetime |
| 252 | doc | the reason the test exists (Node/TextColor are optional queries) -> `///` on `bare_popup_without_node_or_text_despawns` |

### `ui/health_display.rs` (4: 1 compact, 2 doc, 1 drop)

135 and 146 are the two tests' reasons for existing (0.17% must not read 0%; a
`{ current: 0, max: 0 }` root must not render "NaN%") -> `///` on
`living_sliver_ceils_to_one_percent` and `non_positive_max_reads_zero_not_nan`.
139 restates its assert and was dropped.

137 was the round-1 correction. The first pass folded ALL of 135/137/139 into
the `///`, which lost the justification for three magic inputs (`0.4`, `2.29`,
`2.3`) from the body where they sit -- the same `///`-misuse this cluster was
explicitly guarding against, applied in the one file where it was not caught
during the pass. A body `NOTE:` now names what `2.29` vs `2.30` straddles, and
the `///` keeps only the intent sentence.

### `transform/random_sphere_orbit.rs` (4: 4 drop)

143, 147, 150, 157 each restate the statement below (`delta = next - state`,
`max_delta = speed * dt`, and the two identical clamp branches). Nothing here
guards a value or explains a choice.

### `scoring/streak.rs` (4: 1 compact, 2 doc, 1 drop)

194 is a value guard (0.6 + 0.6 straddles the 1.0 window, so only the second
tick ends the streak) -> compact. 199 and 213 are extra coverage the fn names
miss (ticking / extending an INACTIVE streak) -> folded into `///` docs on
`tick_past_window_ends_with_final_count` and
`extend_to_lengthens_but_never_shortens`. 210 restates the latter's name.

### `time/cooldown.rs` (3: 3 doc)

151 (overshoot clamps at zero) and 155 (ticking a ready cooldown is a no-op)
fold into one `///` on `tick_counts_down_and_becomes_ready`; 173 (a negative
window clamps to zero) into one on `trigger_for_sets_a_custom_window`. All
three are coverage the fn names do not carry.

### `scoring/high_score.rs` (3: 1 doc, 2 drop)

124 (a lower score is not a new best and does not lower the best) -> `///` on
`record_updates_and_reports_a_new_best`. 129 and 165 restate their asserts,
the latter alongside a literal `assert_eq!(json, "{\"best\":42}")`.

### `helpers/wasd.rs` (3: 3 drop)

72, 75, 79 are section headers ("Add input context", "Observers for setup and
teardown", "Observers for input actions") over `add_input_context` and
`add_observer` calls that say the same thing.

### `helpers/pointer.rs` (3: 3 compact)

76 guards the conditional `EnhancedInputPlugin` add; 89 is the real ordering
constraint (`after(InputSystems).before(EnhancedInputSystems::Prepare)`, or the
action sees last frame's touch); 97 is why the clear runs in `Last`
(edge-triggering `just_pressed`). All three earn their keep.

### `audio/registry.rs` (3: 1 compact, 2 drop)

207 explains why a deliberately missing path (`"does_not_exist"`) is a valid
stand-in for a slow load -- a freshly requested handle is pending either way.
That is not visible from the code, so it stays. 179 and 199 restate asserts.

### `ui/animate.rs` (2: 1 compact, 1 doc)

145 explains why the round-trip is compared in LINEAR space -> compact. 156 is
four lines of intent plus a stated coverage gap (the tween's END value is
exercised by `13_glide`, not here) -> `///` on `node_flash_starts_bright_white`,
gap included.

### `ui/status.rs` (1: 1 doc) -- the performance contract

The single block was generic prose lifted from Bevy's own docs ("If you really
need full, immediate read/write access ... WARNING: These will block all
parallel execution"), sitting above `update_status_bar_item_values`.

**Verified, not trimmed on sight**, as the Story required. `update_status_bar_item_values(world: &mut World)` really is an exclusive system, and it really does run every frame (`add_systems(Update, ...)` at `status.rs:120`); the closures are `Fn(&World) -> ...` (lines 54, 75, 83), which is exactly why it must be exclusive. The claim holds.

One claim did NOT survive the check and was corrected before commit: the first
draft of the module doc said the `value_fn` *and* `color_fn` closures take
`&World`. `color_fn` is `Fn(Box<&dyn Any>) -> Option<Color>` and runs in the
ordinary parallel `update_status_bar_item_ui`; only `value_fn` is exclusive.
The module doc now says so explicitly, since "keep it cheap" applies far more
weakly to the parallel half.

Two changes rather than one, because the warning was in the wrong place. The
prose was rewritten as a `///` on the system, naming THIS system's cost instead
of exclusive systems in general -- and the contract was added to the module
`//!` doc, which is where a caller writing a `value_fn` will actually read it.
It was absent there, even though AGENTS.md's Module Map cites `ui/status` as
documenting it. Same edit fixed the module doc's "ststaus" typo and a missing
full stop.

### `src/material.rs` (1 block, added in review round 1)

Not in the baseline 106, because the baseline was measured over the ten
directories the Story enumerates. `src/material.rs` is a bare file at the top
of `src/`, and review round 1 established that NO epic child claims it: the
five children cover debug/lib/completion, integrity/physics, mesh/meth/camera,
modding/persist/macros and these ten directories. This task's own scope
sentence -- "everything not claimed by the other children" -- therefore takes
it, and as the last child there is nobody else to take it. Left alone, the
epic-wide gate `./scripts/check-comment-tags.sh src
bevy_common_systems_macros/src` would still have exited 1 with this one block
after the epic closed.

`material.rs:58` is a genuine hazard (an `unlit` emissive material skips the
lighting pass and does not bloom -- AGENTS.md lists it among the things that
render wrong *silently*), so it was compacted and tagged, never dropped. Both
DoD checker proofs now name the file, and the epic-wide invocation was added
alongside so the gap cannot recur by scoping.

### `ui/objectives.rs`, `transform/smooth_look_rotation.rs`, `helpers/temp.rs`, `audio/mod.rs` (1 each)

| File | Call | Reason |
| --- | --- | --- |
| `ui/objectives.rs:164` | doc | the shrink case is coverage `the_panel_renders_one_line_per_objective` does not name -> `///` |
| `transform/smooth_look_rotation.rs:74` | compact | a real schedule choice (PostUpdate, so the target angle is current); first-person voice dropped |
| `helpers/temp.rs:66` | compact | why `Update` (timers tick with the frame delta) |
| `audio/mod.rs:126` | compact | value guard: rodio rejects a non-positive playback rate |

## Split decisions: all KEEP

Code before the test module, after the pass (measured with
`^#\[cfg\((all\()?test`, the form that read `persist/mod.rs` wrong last
cluster):

| File | total | code | concerns | call |
| --- | --- | --- | --- | --- |
| `ui/status.rs` | 328 | 328 | bar machinery + built-in fps/version providers | keep |
| `feedback/flash.rs` | 548 | 288 | one (material hit flash) | keep |
| `tween/mod.rs` | 419 | 256 | one (a value tween + its plugin) | keep |
| `ui/touchpad.rs` | 331 | 232 | reveal-on-touch + hit-test primitives | keep |
| `helpers/wasd.rs` | 231 | 231 | one (enhanced-input bindings for the WASD camera) | keep |
| `feedback/screen_flash.rs` | 325 | 205 | one (full-screen overlay fade) | keep |
| `ui/popup.rs` | 274 | 189 | one | keep |
| `input/pointer.rs` | 210 | 188 | one | keep |

Every other file in scope is under 180 code lines.

**Nothing here reaches the bar the previous clusters set.** The one split in
the epic so far was `mesh/builder.rs` at 521 code lines with a
dependency-disjoint geometry kernel inside it; `modding/events.rs` (404) and
`camera/shake.rs` (313) were both measured and KEPT. The largest body in this
cluster is 328.

Two files were examined properly rather than waved past on size:

**`ui/status.rs` (328, no test module)** is the size candidate and does hold
two things: the bar/item machinery, and the built-in providers
(`status_fps_value_fn`, `status_fps_color_fn`, `status_version_*`). The
dependency sets are NOT disjoint: every provider's return type is
`impl Fn(&World) -> Option<Arc<dyn StatusValue>>`, so the provider half imports
the core half's trait; and `status_bar_with_fps` sits astride the cut, calling
both. A split would leave one file importing the other and one function
belonging to neither. KEEP.

**`ui/touchpad.rs` (232)** is the closest call in the cluster, and the only
file that passes the *concerns* half of the test outright: reveal-on-first-touch
(`TouchSeen`, `RevealOnTouch`/`HideOnTouch` -- ECS observers) and the hit-test
primitives (`button_grid_at`, `stick_deflection` -- pure `Vec2`/`Rect` math with
no ECS at all). Those two dependency sets genuinely are disjoint. It is kept on
the *size* half: 232 code lines is below every keep precedent in this epic, let
alone the 521 that justified the one split, and the halves would be ~120 and
~110 lines. `split-along-the-test-seam` also argues against it -- the file has
one test module, and it exercises only the primitives, so there is no
author-found seam here, just a size that does not hurt. Recorded so the next
reader does not have to re-derive it: if `touchpad.rs` grows a real pad widget,
the cut is already located.

## Alternatives weighed

- **Promote the `13_glide` / `14_breach` provenance in `tween/mod.rs` and
  `ui/animate.rs` into rustdoc instead of dropping it.** Done for
  `ui/animate.rs` (it states a real coverage GAP, which a future reader needs)
  and rejected for `tween/mod.rs:368` (it states WHERE a bug once came from,
  which `tasks/` holds).
- **Simplify `helpers/temp.rs`'s `(update_temp_entities,).chain()`** -- a
  one-element tuple with a no-op `chain()`. Left alone: this task is comments
  and structure, and a behaviour-neutral code edit outside that remit is
  exactly the churn the epic forbids. Noted here instead.
- **Use `///` on test fns more aggressively** to clear the checker faster.
  Rejected per `state-what-the-checker-cannot-see`: `///` carries what a test
  PROVES, and every value guard inside a body (the `0.8` threshold in
  `flash.rs`, `0.6 + 0.6` in `streak.rs`, the eight step-count guards) stayed a
  tagged `NOTE:` in the body where it guards the number. That is the misuse the
  previous cluster's review caught, so the split was applied deliberately here:
  37 kept blocks are `NOTE:`, and every promotion names a fn whose behaviour it
  describes.

## Rustdoc audit

Four warnings existed in scope on the base branch and are gone (they were
called out as this cluster's in 20260731-172232's DoD):

| Warning | Fix |
| --- | --- |
| `` `pointer` is both a module and a primitive type`` (`helpers/mod.rs:5`) | `[`pointer`](mod@pointer)` |
| same (`input/mod.rs:4`) | `[`pointer`](mod@pointer)` |
| unresolved link to `UnifiedPointerPlugin` (`helpers/pointer.rs:17`) | full path `crate::input::pointer::UnifiedPointerPlugin` |
| redundant explicit link target (`helpers/pointer.rs:16`) | `active_pointer_pos` is already imported, so the explicit target was dropped |

`cargo doc --no-deps --features debug` now emits no `lib doc` warning at all.
The only remaining line is the crate-wide `proc-macro-error2` future-incompat
note, which AGENTS.md lists as expected.

Other stale-claim checks that came back clean: the `Flash` / `ScreenFlash`
`On<Insert>` restart claims match the observers; `helpers/pointer.rs`'s "both
write `UnifiedPointer` every frame, so adding both would have them fight"
matches `input/pointer.rs`; `ui/status.rs`'s exclusive-system contract is
verified above (and was the one real gap -- it was missing from the module
doc).

## Difficulties

None of substance. The one judgement call that took real work was
`ui/touchpad.rs`, recorded above rather than resolved silently. The one
correction to the plan: `TASK.md` was first written with a `## Done Means`
heading and prose-style proofs, which `tatr flow --to PLANNED` rejected
(`bad-record-schema`, then eight `bad-proof-syntax`); the sibling task's
`## Definition of Done` with trailing `(cmd: ...)` / `(manual: ...)` parens is
the accepted shape.
