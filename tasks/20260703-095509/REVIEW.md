# Review: Fix stale default event_info path in EventKind derive macro

- TASK: 20260703-095509
- BRANCH: fix/eventkind-default-path

## Round 1

- VERDICT: APPROVE

Verified independently against master:

- Root cause correctly diagnosed and fully fixed. The original default
  `modding::events::game_event::GameEventInfo` was broken twice over: the
  `game_event` module does not exist, and `GameEventInfo` does not
  implement `serde::Serialize` (required by `EventKind::Info`). Defaulting
  to `()` resolves both - `()` satisfies `Serialize + Default + Clone +
  Debug + Send + Sync + 'static` and needs no import at the derive site.
  Confirmed the rejected alternative (fully-qualified path to
  `GameEventInfo`) does indeed still fail the `Serialize` bound, so `()` is
  the right call, not just a convenient one.
- Goal delivered: the attribute-less `#[derive(EventKind)]` now compiles
  and works. `examples/03_modding.rs` `OnTick` is a genuine end-to-end
  guard, not just a compile probe - it is fired via `fire::<OnTick>(())`
  and handled, and `cargo run --example 03_modding` shows the counter
  double-incrementing (OnTick every tick + OnUpdate when value >= 0.5).
- Suite clean: `cargo fmt --check`, `cargo clippy --all-targets` (no crate
  warnings), `cargo test` (13 unit + 11 doctests) all pass. Diff
  introduces no non-ASCII characters.
- Behavior change is safe: since no attribute-less derive could compile
  before, changing the default payload type breaks no existing code; the
  change is purely additive.
- `().into()` path checks out: `GameEventInfo::from_data(())` serializes to
  `Value::Null`, giving `{ data: Some(Null) }` - sane "no payload"
  semantics for filters that inspect `info.data`.

- [ ] R1.1 (NIT) src/modding/events.rs:32 - the `EventKind` trait doc does
  not mention what the derive defaults to when `#[event_info(...)]` /
  `#[event_name(...)]` are omitted (`Info = ()`, name = lowercased struct
  name). The example now documents it inline, so this is discretionary;
  a one-line note on the trait or the derive would aid discoverability.
  - Response:

No BLOCKER/MAJOR/MINOR findings; the NIT is optional. APPROVE.
