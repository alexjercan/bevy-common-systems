# NOTES: KISS pass over modding/ + persist/ + macros subcrate

Design/fix record for task 20260731-172232. Fourth child of epic
20260731-172116, after 20260731-172208 (convention + checker), 20260731-172223
(integrity/physics) and 20260731-172224 (mesh/meth/camera).

## Baseline

`./scripts/check-comment-tags.sh src/modding src/persist bevy_common_systems_macros/src`
reported **24** untagged non-doc comment BLOCKS. Per file: `events.rs` 8,
`registry.rs` 8, `persist/mod.rs` 5, `persist/backend.rs` 2, macros `lib.rs` 1.

(The Story's "34 comments between them" and "14 more" count comment LINES,
which is a different unit. Recording both, per the `record-numbers-from-a-rerun`
ledger lesson from the previous cluster.)

Bare tatr HUIDs in non-doc comments in scope: **0**.

Code before `#[cfg(test)]`:

| File | total | code | concerns | call |
| --- | --- | --- | --- | --- |
| `modding/events.rs` | 543 | 404 | one (the event bus), 4 layers sharing a type | keep |
| `modding/registry.rs` | 494 | 320 | one (JSON -> handler construction) | keep |
| `modding/mod.rs` | 9 | 9 | one | keep |
| `persist/mod.rs` | 200 | 200 | one (plugin + load/save systems) | keep |
| `persist/backend.rs` | 160 | 80 | one (platform split behind load/save) | keep |
| macros `lib.rs` | 43 | 43 | one (the derive) | keep |

## Split decisions: all KEEP

`events.rs` at 404 code lines is the largest remaining body in the crate and
was the obvious candidate. It was measured and kept.

The test that carried the previous two splits - **do the parts have disjoint
dependency sets?** - fails here. `events.rs` holds four layers:

1. the traits (`EventWorld`, `EventKind`, `EventAction`, `EventFilter`),
2. the `EventHandler` component and its builder methods,
3. `GameEvent` / `GameEventInfo` / `Commands::fire`,
4. the plugin: queue, `EventHandlerIndex`, `maintain_handler_index`,
   `queue_system`.

Layer 4 needs every one of 1-3 (`EventHandlerIndex` stores `EventHandler<W>`
clones and is generic over `W: EventWorld`; the dispatcher reads `GameEvent`
and calls both trait objects). There is no cut that reduces what any resulting
file imports - unlike `mesh/slice.rs`, whose kernel imported only
`bevy::prelude` after the move. Splitting here would move code between files
while leaving `use super::*` behind, which is churn, not structure.

`registry.rs` (320 code) is one concern end to end: parse `HandlerSpec` JSON,
resolve names against the registry, build handlers. Its bulk past line 320 is
the test rig that exercises exactly that.

`persist/` is already split along the one seam it has - `backend.rs` hides
native vs wasm behind `load`/`save`, `mod.rs` owns the plugin - and neither
half is large.

## The macros subcrate: guidance verified, and it had drifted

The task said to verify the `EventKind` guidance rather than trim it. It was
wrong, and had been for some time.

AGENTS.md's Gotchas carried: *"`EventKind` derive's default `Info` path does
not resolve (`tasks/20260703-095509`). Always pass `#[event_info(...)]`."*
That was true when written. The derive was fixed since - the default is now
`quote! { () }`, the unit type, which satisfies the `EventKind::Info` bounds
(`Serialize + Default + Clone + Debug`) and needs no import at the derive site.
The in-code comment at `macros/src/lib.rs:12` already recorded the fix; the
repo-level guidance never caught up, so it kept steering every caller away from
a path that works.

Nothing in the repo derives `EventKind` without `#[event_info(...)]`
(`examples/03_modding.rs` is the only derive site and it passes one), so the
default was live but **unexercised** - which is why the stale warning was
invisible. Added
`modding::events::tests::attribute_less_derive_defaults_to_no_payload`: it
derives an attribute-less `EventKind`, asserts the name defaults to the
lowercased struct name, and binds `<OnQuiet as EventKind>::Info` to `()`. That
binding is the real guard - it fails to COMPILE, not to run, if the default
payload type ever changes back to something that does not resolve, which is
exactly the original defect.

AGENTS.md's gotcha now states current behaviour and points at that test. The
in-code `NOTE:` at the default keeps the "do not name a concrete type here"
hazard, which is the part still worth guarding.

## Comment calls (24 blocks)

Legend as in the previous cluster: **drop** = restates the code; **compact** =
kept as one tagged `NOTE:`; **doc** = kept as intent, promoted to `///` on the
item it describes (per the AGENTS.md convention this epic added).

### `modding/events.rs` (8)

| Line | Comment | Call |
| --- | --- | --- |
| 79 | hand-written `Clone` so the bound is `EventWorld`, not `Clone` | compact - explains a non-obvious impl the index depends on |
| 256 | index maintenance is ungated and ordered before dispatch | compact - a real ordering hazard, and an instance of the AGENTS.md empty-set rule |
| 388 | "only the handlers for this event name, walked contiguously" | drop - the `EventHandlerIndex` rustdoc above already explains the design at length, and `index.handlers(event.name)` says it |
| 408 | `bevy::prelude::*` arrives via `super::*` | compact - non-obvious import provenance |
| 481 | an unregistered event must be a no-op | compact - names what the bare `fire` + unchanged count is testing |
| 497 | despawn pruning must happen on an idle frame | compact - the point of the step |
| 509 | the observer read-accessor contract | doc - describes the whole test |
| 538 | observing must not starve dispatch | compact - explains why a second assertion follows |

### `modding/registry.rs` (8)

| Line | Comment | Call |
| --- | --- | --- |
| 345, 357 | "an action that carries params" / "a filter that reads a threshold" | doc - they describe the fixture types |
| 394 | the filter's `min: 3` blocks below, passes at | drop - the three asserts below say it |
| 401 | the action's `amount: 5` drives behaviour; `actions` is `pub(super)` | compact - records WHY the test reaches a `pub(super)` field |
| 413 | a typo must be a parse error | doc |
| 422 | `EventHandler` is not `Debug`, so use `.err().unwrap()` | compact - explains a non-obvious idiom repeated below |
| 460 | `amount` is an integer, a string fails | compact - names the mutation under test |
| 486 | a constructor with no params | compact |

### `persist/mod.rs` (5)

| Line | Comment | Call |
| --- | --- | --- |
| 94 | load synchronously, not in a system | compact - a deliberate design choice a reader would otherwise "fix" |
| 145 | native-only test, hermetic via `BCS_PERSIST_DIR` | compact - explains the `cfg` |
| 169 | the ONLY test that sets the env var; new ones must share or serialize | compact - a genuine cross-test hazard |
| 174, 187 | "First launch" / "Second launch" | drop - the assertion messages ("default on a clean store", "restored across launches") already carry it |

### `persist/backend.rs` (2)

| Line | Comment | Call |
| --- | --- | --- |
| 99 | "overwriting replaces the stored value" | drop - restates the save-then-assert below |
| 115 | kept pure so it never races the env-based test | doc - a property of the whole test |

### macros `lib.rs` (1)

| Line | Comment | Call |
| --- | --- | --- |
| 12 | why the default `Info` is `()` and what the old default broke | compact, and extended with a pointer to the new regression test |

## Rustdoc audit

`EventHandlerIndex`'s doc linked `[`queue_system`]` and
`[`maintain_handler_index`]`, both private - the two in-scope `cargo doc`
warnings. Fixed the way `integrity/plugin.rs` did it: name them in prose. The
other four warnings `cargo doc` reports are in `helpers/` and `input/`, which
belong to the last cluster (20260731-172233), not this one; the DoD was
corrected mid-task to say so after it initially miscounted them as in scope.
