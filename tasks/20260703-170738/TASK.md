# 09_reactor: rules-as-gameplay incremental on the modding bus

- STATUS: CLOSED
- PRIORITY: 20
- TAGS: feature,example,modding

Low-priority pick from the 01-05 games spike (see
`tasks/20260703-165138/NOTES.md`). An idle/incremental game where
the rules ARE the mod system: the `EventWorld` holds a few resources (energy,
heat, credits), ticks and clicks `fire` events, and the player builds their
machine by placing `EventHandler` entities (filter + action pairs, exactly as
`examples/03_modding` shows). Goal: compose handlers into an escalating loop
without letting heat run away.

Deferred behind the games worth building first: it is the least visual and the
hardest to make legibly fun, and to be a game rather than a demo it really
wants the JSON handler-registry feature (20260703-165439) so handlers can be
data-authored. Build that first, or accept a Rust-authored handler palette.

Scope: this is a library example, not a product. Keep it small (~1000 LoC),
basic but fun for ~15 minutes. No tech trees, no meta-progression, no
save-game sprawl -- just enough to show the modding bus driving a real loop.
Follow the 06_fruitninja shape (states, sounds, wasm gallery build). Grows out
of `examples/03_modding`.

