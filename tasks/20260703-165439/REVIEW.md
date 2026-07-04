# Review: modding JSON-authored EventHandler registry

- TASK: 20260703-165439
- BRANCH: feature/modding-json-registry

## Round 1

- VERDICT: REQUEST_CHANGES

The core paths are sound and well covered (unknown event/filter/action names,
bad params, invalid JSON, null/missing params, filter gating), clippy and the
full test suite are green, prelude/plugin wiring and ASCII conventions are
correct, and the stale `event_info` derive gotcha does not bite. Findings below
are polish plus one test-honesty issue.

- [x] R1.1 (MAJOR) src/modding/registry.rs:369-377 - the test comment claims
  "the filter blocks below the threshold and the action adds the configured
  amount", but the test only calls `handler.filter(...)`; the action is never
  run and `world.counter` is never asserted to change by 5. The Steps checklist
  explicitly asks to assert the deserialized params "drive filter/action
  behaviour", and only the filter's `min` param is actually verified. Exercise
  the built action too (the `actions` field is `pub(super)`, so the test module
  can iterate `handler.actions` and call `action.action(&mut world, &info)`, or
  add a small `EventHandler` helper) and assert the counter advances by the
  JSON-supplied `amount`. Otherwise soften the comment - do not claim behaviour
  the test does not exercise.
  - Response: Fixed. `builds_handler_with_filter_and_action` now iterates the
    built handler's `actions` (reachable via `pub(super)`) and asserts the
    counter advances by the JSON `amount: 5` (0/3 -> 8); the comment was
    reworded to describe only what the test verifies.
- [x] R1.2 (MINOR) src/modding/registry.rs:243 - `parse_specs` takes no `&self`
  and never uses `W`, yet lives on `EventHandlerRegistry<W>`, forcing the
  pointless turbofish at examples/03_modding.rs:148
  (`EventHandlerRegistry::<CustomEventWorld>::parse_specs(...)`). Move it to a
  free module function `pub fn parse_specs(json: &str) -> Result<Vec<HandlerSpec>,
  RegistryError>` (re-exported from the prelude) so deserializing specs does not
  drag an unrelated world type through the call site.
  - Response: Fixed. `parse_specs` is now a free module function, re-exported
    from the prelude; the example calls plain `parse_specs(HANDLERS_JSON)`.
- [x] R1.3 (MINOR) src/modding/registry.rs:73,93 - neither `HandlerComponentSpec`
  nor `HandlerSpec` sets `#[serde(deny_unknown_fields)]`, so a mod author's typo
  (`"acions"`, `"parms"`) deserializes to a default and the handler silently does
  nothing. For a data-authoring surface, add `deny_unknown_fields` so a typo
  becomes a `RegistryError::Parse`, and add a test for it.
  - Response: Fixed. Both specs carry `#[serde(deny_unknown_fields)]`; new test
    `unknown_json_fields_are_rejected` asserts a mistyped field is a
    `RegistryError::Parse`.
- [x] R1.4 (MINOR) src/modding/registry.rs:208,235 - the `_de` helpers call
  `serde_json::from_value::<F>(params.clone())`, deep-cloning the params on every
  build. `&serde_json::Value` implements `Deserializer`, so `F::deserialize(params)`
  deserializes without the clone. Drop the clone.
  - Response: Fixed. Both helpers now call `F::deserialize(params)` /
    `A::deserialize(params)` by reference, no clone.
- [x] R1.5 (MINOR) src/modding/registry.rs:181,195,222 - `register_*` use
  `HashMap::insert`, so re-registering a name silently clobbers the previous
  constructor. Document the last-wins behaviour in the method docs and emit a
  `trace!`/`debug!` on overwrite so a mod name collision is at least visible.
  - Response: Fixed. Each `register_*` documents last-wins and emits a `trace!`
    when `insert` overwrites an existing name.
- [x] R1.6 (NIT) src/modding/registry.rs:153 - the registry is a `Resource` but
  not `Reflect` (it holds boxed closures, so it genuinely cannot be). Add a
  one-line comment on the struct noting `Reflect` is intentionally absent, so a
  future session does not try to add it against the crate convention.
  - Response: Fixed. Added a struct doc line explaining `Reflect` is
    intentionally absent because it holds boxed constructor closures.
- [x] R1.7 (NIT) src/modding/registry.rs:180-183 - `register_event::<E>()` always
  inserts `E::name() -> E::name()` (key and value identical), so the `events` map
  is effectively a `HashSet<&'static str>` used only to recover the `'static`
  lifetime from the JSON string. The doc ("an event name resolves to the static
  name") reads as if an alias were possible; reword to make clear the JSON name
  must equal `E::name()`.
  - Response: Fixed. `register_event` doc now states the JSON `event` must be
    exactly `E::name()` with no aliasing.

## Round 2

- VERDICT: APPROVE

All seven Round 1 findings verified fixed against the new diff:

- R1.1: the build test now runs the built actions and asserts `counter == 8`
  from the JSON `amount: 5`, so both filter and action params are exercised; the
  comment matches. The action path is no longer untested.
- R1.2: `parse_specs` is a free function in `registry.rs`, re-exported from the
  modding prelude; the example uses it without a turbofish.
- R1.3: `deny_unknown_fields` on both specs; `unknown_json_fields_are_rejected`
  confirms a typo becomes `RegistryError::Parse`.
- R1.4: `F::deserialize(params)` / `A::deserialize(params)`, no clone.
- R1.5: last-wins documented; `trace!` on every overwrite.
- R1.6, R1.7: doc comments added/reworded as requested.

`cargo fmt --check`, `cargo clippy --all-targets` (+`--features debug`),
`cargo test` (27 unit + 13 doctests) and `--features debug` all green;
`scripts/check-ascii.sh` clean; `examples/03_modding` still boots to the render
loop and the JSON-authored handlers fire. Clean, in-convention, well tested.
