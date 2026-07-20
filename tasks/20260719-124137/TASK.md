# GameEvent public read accessors (name/info) so downstream recorders can observe events

- STATUS: CLOSED
- PRIORITY: 50
- TAGS: modding,api,historical

## Record

Driver: nova-protocol's run-timeline recorder (its task 20260719-112238) needs
to observe scenario events from outside this crate, and `On<GameEvent>` is the
only observable hook - but `name`/`info` were `pub(super)`, so an external
observer could see an event pass by yet read nothing off it.

Change: two read accessors on `GameEvent` (`name()`, `info()`), fields stay
module-private, construction stays `GameEvent::new`-only. A doctest pins the
PUBLIC visibility (doctests compile as an external crate), and a behavior test
pins the recorder pattern: an observer reads name+payload of fired events
WITHOUT draining the dispatch queue (the queue still holds both events for
handlers afterward).

Verified: fmt, clippy --all-targets (only the known proc-macro-error2
future-incompat note), modding tests 10/10, doctests 2/2, check-ascii. Version
0.19.2 (both crates), CHANGELOG entry added.
