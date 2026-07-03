# Fix stale default event_info path in EventKind derive macro

- STATUS: CLOSED
- PRIORITY: 10
- TAGS: bug

## Goal

`#[derive(EventKind)]` without an `#[event_info(...)]` attribute generates
`impl EventKind` with `type Info = modding::events::game_event::GameEventInfo`,
but no `game_event` module exists; the real path is
`modding::events::GameEventInfo` (or the crate prelude re-export). The
attribute-less derive therefore fails to compile in user code.

## Steps

- [x] Change the default `event_info` in
      bevy_common_systems_macros/src/lib.rs. REVISED during work: the goal
      was "a path that resolves", but the real default type must also
      satisfy the `EventKind::Info` bound `serde::Serialize + Default +
      Clone + Debug`. `GameEventInfo` fails that (it has no `Serialize`
      impl), so no path to it could ever compile. Correct fix: default to
      `()`, the unit type - it satisfies every bound, needs no import, and
      means "no payload".
- [x] Extend examples/03_modding.rs with an attribute-less
      `#[derive(EventKind)]` (`OnTick`), fire it and handle it, proving the
      default `name()` ("ontick") and default `Info` (`()`) work end to end.
- [x] Run the full check suite (fmt, clippy --all-targets, test) plus
      `cargo run --example 03_modding` briefly to confirm behavior. Verified
      OnTick fires every tick (counter double-increments alongside
      OnUpdate).

## Notes

- Found during 20260703-094842 while reading the macro for documentation.
- Example 03_modding.rs never hits this because it always passes
  #[event_name] and #[event_info].
- Out of scope for the current docs flow (goal: AGENTS.md/CLAUDE.md);
  backlog item for a future session. Not a dependency of anything.

## Close-out

What changed and why:
- bevy_common_systems_macros/src/lib.rs: default `event_info` changed from
  the nonexistent `modding::events::game_event::GameEventInfo` to `()`.
- examples/03_modding.rs: added `OnTick` (attribute-less derive), a handler
  for it, and a `fire::<OnTick>(())` call, as an end-to-end regression
  guard consistent with the examples-as-integration-tests convention.

Alternatives considered:
- Fully-qualified `bevy_common_systems::modding::events::GameEventInfo`:
  the path resolves, but compilation still fails because `GameEventInfo`
  does not implement `serde::Serialize` (required by `EventKind::Info`).
  Rejected: does not deliver the Goal.
- Deriving `Serialize`/`Deserialize` on `GameEventInfo` so it could stay
  the default: possible (its field is `Option<serde_json::Value>`), but
  `GameEventInfo` is the event *wrapper*, not a payload; using it as the
  payload double-wraps on `.into()` (`GameEventInfo { data: serialize(
  GameEventInfo) }`). Rejected as semantically confusing.
- `()` chosen: it is the payload for "this event carries no data", which
  is exactly what an attribute-less event means; zero new code, no import.

Difficulties:
- The task framed this as a one-line path fix. Building the test
  immediately surfaced the deeper `Serialize` bound failure - the path was
  only half the bug. Diagnosed straight from the compiler error
  (`GameEventInfo: serde::Serialize is not satisfied`). Writing the test
  first (rather than trusting the one-line framing) is what caught it.

Self-reflection:
- Good: implemented the test before declaring the fix done, which turned a
  plausible-but-wrong one-line change into the correct fix. Reinforces the
  work skill's "tests alongside, not after".
- The planning note assumed the fix was purely a path; a quick
  `cargo build` against a throwaway bare derive during planning would have
  revealed the Serialize bound and set a more accurate Step. Cheap probe,
  worth doing when a "trivial" bug touches a trait bound.
