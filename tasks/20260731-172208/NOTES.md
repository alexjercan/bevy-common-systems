# NOTES -- 20260731-172208 (KISS pass: debug/ + lib.rs + completion.rs)

First child of epic 20260731-172116. Sets the inline-comment convention the
four sibling clusters inherit, and ships the checker that enforces it.

## What changed

- `scripts/check-comment-tags.sh` (new): fails on any non-doc comment block
  whose FIRST line lacks `NOTE:` / `FIXME:` / `BUG:` / `TODO:`. Takes paths so
  each sibling task can scope it to its own cluster.
- `AGENTS.md`: one Conventions bullet stating the rule and naming the script.
- Comment pass over the eight in-scope files: 37 untagged blocks (74 lines) ->
  0 untagged, 35 lines. 13 blocks kept and tagged, 21 dropped, 3 promoted to
  rustdoc.
- `autopilot.rs:51`: fixed the unresolved intra-doc link `loop_while_pending`
  -> `[...](AutopilotPlugin::loop_while_pending)`.

## Keep / compact / drop, all 37 blocks

`src/debug/inspector.rs` (14 blocks)

| Line | Call | Reason |
| --- | --- | --- |
| 35 | drop | restates `insert_resource(DebugEnabled(true))` |
| 38 | drop | restates the two `add_plugins` lines |
| 42 | drop | restates the `run_if(resource_equals(...))` |
| 48 | keep | guards `auto_create_primary_context: false`; compacted to one NOTE naming the system that owns placement instead |
| 55 | promote | six lines of rationale duplicating `keep_inspector_on_window_camera`'s own rustdoc; the unique part (the replaced first-camera-wins observer + the two nova HUIDs) moved INTO that rustdoc, clearing the epic HUID proof |
| 63 | drop | restates the avian plugin tuple |
| 70 | drop | restates `add_systems(Update, ...)` |
| 98 | drop | restates `ui_for_world` |
| 101 | drop | restates the `CollapsingHeader::new("Materials")` |
| 106 | drop | restates `ui.heading("Entities")` |
| 142 | keep | THE hazard: removing `PrimaryEguiContext` alone leaves `EguiContext` + `EguiMultipassSchedule` and bevy_egui panics. Tagged NOTE, trimmed 6 lines -> 5 |
| 208 | promote | test provenance (nova task 20260710-104421) -> `///` on the test fn; keeps the HUID out of a non-doc comment |
| 232 | drop | test narration; the fn name plus `keep_inspector_on_window_camera`'s rustdoc already say it |
| 303 | drop | restates the `insert(RenderTarget::Image(..))` on the next line |

`src/debug/wireframe.rs` (4 blocks: 44, 47, 50, 56) -- all dropped, each a
literal restatement of the single line beneath it.

`src/debug/harness/autopilot.rs` (6 blocks) -- all kept, all tagged, all
compacted; nothing here restates code.

| Line | Guards |
| --- | --- |
| 198 | the `.after(InputSystems)` ordering (Bevy clears `just_pressed` there) |
| 215 | the early return -- do not index past the schedule end |
| 228 | skipping the first `NextState` set, to avoid a spurious OnExit/OnEnter |
| 250 | finishing mid-cycle while looping, so a slow cycle cannot straddle the deadline |
| 286 | holding the final step and zeroing BOTH clocks on a loop restart |
| 295 | expired runway is an error exit, not a negotiated completion |

`src/debug/harness/screenshot.rs` (11 blocks)

| Line | Call | Reason |
| --- | --- | --- |
| 127 | keep | why the screenshot harness stands down when autopilot is armed |
| 137 | drop | restates `parse_resolution(&env_value).or(self.resolution)` |
| 159 | drop | `hide_debug_overlay`'s own rustdoc says exactly this, two lines below |
| 187 | keep | guards `window.resizable = false` against a reflowing WM |
| 206 | drop | restates the `if !config.advanced` branch |
| 216 | drop | `MAX_WAIT_FRAMES`'s rustdoc already states the bound and the reason |
| 243 | keep | why a SECOND observer is needed (`save_to_disk` is synchronous in its observer) |
| 278, 280 | drop | test narration; the inputs `1024X768` / `" 640 x 480 "` are self-evident |
| 286 | drop | folded into the `assert_eq!` message, where a failure will actually show it |
| 297 | promote | `///` on the test fn -- it explains the rule, not the line |

`src/lib.rs` (1 block, line 7) -- keep. Why `completion` is ungated while the
harness plugins are not. Tagged, 4 lines -> 3.

`src/completion.rs` (1 block, line 110) -- keep. Why the watcher is added once
per registrant rather than once. Tagged, 3 lines -> 3.

`src/debug/mod.rs`, `src/debug/harness/mod.rs` -- no non-doc comments; rustdoc
audited, nothing stale.

## Structure: no splits (confirms DECISION.md D3)

Code-before-tests, after the pass:

| File | Before pass | After | Concerns |
| --- | --- | --- | --- |
| `harness/autopilot.rs` | 320 | 319 | one: the scripted state driver |
| `harness/screenshot.rs` | 270 | 263 | one: advance-settle-capture |
| `inspector.rs` | 189 | 178 | one: the inspector plugin + its context reconcile |
| `completion.rs` | 147 | 147 | one: the exit-negotiation protocol |
| `harness/mod.rs` | 85 | 85 | module doc + env-var consts |
| `wireframe.rs` | 73 | 66 | one: the wireframe toggle |
| `lib.rs` | 45 | 44 | module list + prelude |
| `debug/mod.rs` | 40 | 40 | module doc + prelude |

The epic's split rule needs measured size AND more than one concern. Every
file here is single-concern, and the largest (319) sits well under the epic's
flagged outliers (`mesh/builder.rs` 521, `modding/events.rs` 404). No split.

The closest call is `inspector.rs`, which carries the plugin AND the
primary-context reconcile. They are kept together deliberately: the reconcile
exists only because this plugin disables egui's auto-creation, and splitting
would separate the `auto_create_primary_context: false` line from the system
that compensates for it -- the exact coupling the kept NOTE at line 48 exists
to make visible.

## Difficulties

- The two nova HUIDs (`20260710-104421`, `20260712-201603`) point at ANOTHER
  repo's tasks, so "HUID only when it points at a live task record" could not
  be satisfied locally. Resolved by moving both into rustdoc, which the epic's
  HUID proof does not scan and which is the right home for provenance a
  downstream reader benefits from.
- First cut of the checker treated `///` as a continuation of a preceding
  `//` run, which would let a doc comment mask an untagged comment beneath it.
  Fixed by having any non-`//` line, rustdoc included, reset the block.
- A `///` doc placed AFTER `#[test]` compiles but is not a doc comment on the
  item; caught by re-reading the file, not by the tool result.

## Evidence

| Proof | Base | After |
| --- | --- | --- |
| `check-comment-tags.sh src/debug src/lib.rs src/completion.rs` | 37 untagged, exit 1 | exit 0 |
| scoped bare-HUID grep | 2 lines | none |
| `cargo doc --no-deps --features debug`, in-scope warnings | 1 (`autopilot.rs:51`) | 0 |
| same, crate total | 11 | 10 |
| `cargo fmt --check` | clean | clean |
| `cargo clippy --all-targets` (+ `--features debug`) | exit 0 | exit 0 |
| `cargo test` / `--features debug` / `--examples` | 59 / 66 / 1 pass | 59 / 66 / 1 pass |
| `./scripts/check-ascii.sh` | clean | clean |

Test counts are unchanged in both configurations, which is the intended
result: this pass touches comments, one rustdoc link, and no behavior.

Clippy exits 0 in both configurations but is not silent: `src/completion.rs:88`
carries a warn-level `clippy::manual_contains` that pre-exists on `master` and
sits outside this diff. Filed as task 20260731-180747 rather than folded in.

## Reflection

- The checker was worth writing before the first edit. Running it on the
  untouched tree produced the exact 37-block worklist the plan predicted, so
  "did I miss one" never became a judgement call.
- The strongest signal for DROP was proximity to rustdoc: five of the 21
  dropped comments were restating a `///` block within ten lines
  (`screenshot.rs:159` and `:216` most starkly). Worth telling the sibling
  tasks: check the item's own rustdoc before deciding a comment is
  load-bearing.
- Promoting to rustdoc, rather than deleting, is the right move whenever the
  content is provenance or a rule about the item. It satisfies the HUID proof
  as a side effect instead of forcing a delete-or-violate choice.

## Review round 1 fixes

Round 1 caught a scope gap the implementation had not considered: the
`AGENTS.md` bullet claimed to govern "inline (non-doc) comments" while the
checker only ever sees own-line ones. That mattered because `src/` already
carries 12 end-of-line comments and every one of them is in a SIBLING
cluster's files -- the four remaining tasks would have inherited a rule whose
stated scope and enforcement disagreed.

Resolved by narrowing the rule rather than widening the script: end-of-line
comments are exempt, stated in both the bullet and the script header. The
existing 12 are value labels (`// 1 neighbor`, `// 0..=1`, `// no clamp`);
tagging them `// NOTE: 0..=1` would be exactly the noise this epic removes.

Also fixed: the tag regex demanded exactly one space, so a correctly written
`//NOTE:` was reported as untagged with a message telling the author to add
the tag they had already added; and filenames were word-split into awk, which
broke on paths containing whitespace.

Lesson for the siblings: when a task's deliverable is a CONVENTION, the doc
and the enforcement are one artifact. A checker that is narrower than its
bullet is worse than no checker, because exit 0 reads as compliance.
