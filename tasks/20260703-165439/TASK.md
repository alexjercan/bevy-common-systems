# modding: JSON-authored EventHandler registry for the event bus

- STATUS: CLOSED
- PRIORITY: 50
- TAGS: feature,modding

Surfaced by the 01-05 games spike (see
`tasks/20260703-165138/NOTES.md`). The `modding/events` module can
already carry event payloads as `serde_json::Value` across a scripting
boundary, but there is no way to build an `EventHandler` (its filter + action
trait objects) from data -- the constructors are all Rust. Add a registry that
maps event-name / filter-name / action-name strings to registered constructors
so handlers can be authored in JSON and loaded at runtime.

This is the missing half of the module's stated purpose (crossing a
modding/scripting boundary), useful independently of any game, and the
precondition for a real "03 Reactor" modding game later. Watch the known
gotcha: the `EventKind` derive's default `event_info` path is stale
(`tasks/20260703-095509`). Grows out of `examples/03_modding`.

## Steps

- [x] Add `src/modding/registry.rs`: an `EventHandlerRegistry<W>` mapping
      event/filter/action name strings to registered constructors, plus a
      `HandlerSpec` / `HandlerComponentSpec` serde schema and a `RegistryError`.
- [x] Registration API: `register_event::<E>()`, `register_filter(name, ctor)`
      / `register_action(name, ctor)`, and `_de` convenience variants that
      deserialize the params JSON straight into a `DeserializeOwned` type.
- [x] `build_handler(&HandlerSpec)`, `parse_specs(&str)` and `parse_handlers(&str)`
      that produce `EventHandler<W>` values (with the `Name` from the spec
      available via `parse_specs`), with clear errors for unknown
      event/filter/action names and bad params.
- [x] Wire the registry into the `modding` prelude; init an empty registry in
      `GameEventsPlugin` so game code can populate it from a startup system.
- [x] Unit-test the pure logic: successful build (asserting params drive
      behaviour), unknown-name errors, bad params, invalid JSON, a
      params-ignoring custom constructor, plus a module doctest.
- [x] Rework `examples/03_modding` to author both handlers from an inlined JSON
      string through the registry (keeping the Rust filter/action types); ran
      it -- boots to the render loop and the filter gates the action correctly.
- [x] `cargo fmt`, `cargo clippy --all-targets` (+`--features debug`),
      `cargo test` (+`--features debug`), ASCII check all green; decision noted
      in `tasks/20260703-165439/NOTES.md`.

