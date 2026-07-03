# modding: JSON-authored EventHandler registry for the event bus

- STATUS: OPEN
- PRIORITY: 50
- TAGS: feature,modding

Surfaced by the 01-05 games spike (see
`docs/2026-07-03-example-games-ideation.md`). The `modding/events` module can
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

