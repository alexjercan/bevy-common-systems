# JSON-authored EventHandler registry for the modding event bus

Task `tasks/20260703-165439`. Adds `src/modding/registry.rs`: an
`EventHandlerRegistry<W>` that maps event / filter / action name strings to
registered constructors, so `EventHandler`s can be authored as data (JSON) and
built at runtime instead of only in Rust.

## Why

The `modding/events` module already carries event payloads as
`serde_json::Value`, which is the crate's stated "cross a modding / scripting
boundary" purpose. But the other half was missing: an `EventHandler` (its
filter and action trait objects) could only be built with Rust constructors
(`EventHandler::new::<E>().with_filter(..).with_action(..)`). A mod or a data
file had no way to say "when event X fires and filter Y passes, run action Z".
This registry closes that gap and is the precondition for a future data-driven
"Reactor" modding game.

## Design

The trait objects (`EventFilter<W>`, `EventAction<W>`) cannot be deserialized
directly -- serde has no way to pick a concrete Rust type from a name. So the
registry inverts it: the game registers, under string names, the concrete types
it understands, and JSON only ever refers to those names.

- `EventHandlerRegistry<W>` holds three maps:
  - `events: name -> &'static str` -- an event name resolves to the static
    name an `EventHandler` matches on. Populated with `register_event::<E>()`.
  - `filters` / `actions: name -> constructor`. A constructor is a boxed
    closure `Fn(&serde_json::Value) -> Result<Arc<dyn ..>, String>`; it turns
    the spec's `params` JSON into a shared trait object.
- Registration:
  - `register_filter(name, ctor)` / `register_action(name, ctor)` for full
    control (the closure gets the raw params and can ignore them).
  - `register_filter_de::<F>(name)` / `register_action_de::<A>(name)` for the
    common case: the type is `DeserializeOwned` and is built with
    `serde_json::from_value`.
- Data schema (serde `Deserialize`): `HandlerSpec { name?, event, filters[],
  actions[] }` and `HandlerComponentSpec { type, params }`. `name` is an
  optional display name carried through for a Bevy `Name`; the registry does
  not use it.
- Building: `build_handler(&HandlerSpec)` resolves every name and calls the
  constructors, returning an `EventHandler<W>` or a `RegistryError` that says
  exactly which name or param was wrong. `parse_handlers(&str)` parses a JSON
  array and builds all of them; `parse_specs(&str)` just parses (so the caller
  can keep the display name next to the built handler).

Two small support pieces made this clean instead of hacky:

- `events.rs` gained `EventHandler::from_event_name(&'static str)` plus
  `add_filter_arc` / `add_action_arc`, so the registry builds handlers through
  the public API rather than reaching into private fields. `new::<E>()` now
  delegates to `from_event_name`.
- `GameEventsPlugin` inits an empty `EventHandlerRegistry<W>` resource, so game
  code can grab it with `ResMut` in a startup system, register its types, and
  spawn handlers -- see the reworked `examples/03_modding`.

### Errors, not panics

`RegistryError` is a plain enum (`Parse`, `UnknownEvent`, `UnknownFilter`,
`UnknownAction`, `Params { component, message }`) implementing `Display` +
`Error`. Constructors return `Result<_, String>` (a message); `build_handler`
wraps that into `Params { component, .. }` so the error names the offending
`type`. No `thiserror` -- it is not a dependency and the manual `Display` is
five lines.

## Alternatives considered

- **Deserialize trait objects via a tag/registry crate (typetag, erased-serde).**
  Rejected: a new dependency, and it forces every filter/action to be
  `Serialize` too. The name-plus-params indirection keeps the payload types
  free to be anything `DeserializeOwned`, and keeps the modding surface (the
  set of legal names) explicit and game-controlled.
- **Store constructors keyed by `TypeId` instead of strings.** Rejected: the
  whole point is that the authoring side only has strings.
- **Make the registry own spawning (a `Commands` extension).** Rejected for
  now: building and spawning are separable, and the caller usually wants to
  attach its own components (a `Name`, marker components). `parse_specs` +
  `build_handler` covers it without coupling the registry to `Commands`.

## Testing

- Unit tests in `registry.rs` cover a successful build (and assert the
  deserialized params actually drive filter/action behaviour), unknown
  event/filter/action names, bad params, invalid JSON, and a custom
  params-ignoring constructor.
- A module doctest shows the minimal end-to-end path.
- `examples/03_modding` was reworked to author both its handlers from an inlined
  JSON string through the registry (keeping the Rust `MinValueFilter` /
  `IncrementCounterAction` types), which is the de facto integration test.

## Gotchas hit

- `EventHandler` is not `Debug` (it holds `Arc<dyn ..>`), so `Result::unwrap_err`
  (which needs the `Ok` type to be `Debug`) does not compile on
  `parse_handlers` results. Tests use `.err().unwrap()` instead.
- The AGENTS.md gotcha about the `EventKind` derive's default `event_info` path
  being stale is out of date on this HEAD -- the macro already defaults `Info`
  to `()`, and the attribute-less derive compiles (the doctest relies on it).
