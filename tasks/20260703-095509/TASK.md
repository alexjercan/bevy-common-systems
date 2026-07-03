# Fix stale default event_info path in EventKind derive macro

- STATUS: OPEN
- PRIORITY: 10
- TAGS: bug

## Goal

`#[derive(EventKind)]` without an `#[event_info(...)]` attribute generates
`impl EventKind` with `type Info = modding::events::game_event::GameEventInfo`,
but no `game_event` module exists; the real path is
`modding::events::GameEventInfo` (or the crate prelude re-export). The
attribute-less derive therefore fails to compile in user code.

## Steps

- [ ] Change the default `event_info` path in
      bevy_common_systems_macros/src/lib.rs to a path that resolves for
      downstream users (decide: fully qualified
      `bevy_common_systems::modding::events::GameEventInfo` vs relying on
      the user's imports; check how other Bevy derive macros reference their
      home crate).
- [ ] Add a compile test or extend examples/03_modding.rs with a
      `#[derive(EventKind)]` that uses no attributes, proving the default
      path resolves.
- [ ] Run the full check suite (fmt, clippy --all-targets, test) plus
      `cargo run --example 03_modding` briefly to confirm behavior.

## Notes

- Found during 20260703-094842 while reading the macro for documentation.
- Example 03_modding.rs never hits this because it always passes
  #[event_name] and #[event_info].
- Out of scope for the current docs flow (goal: AGENTS.md/CLAUDE.md);
  backlog item for a future session. Not a dependency of anything.
