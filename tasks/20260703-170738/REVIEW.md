# Review: 09_reactor -- rules-as-machine incremental on the modding bus

- TASK: 20260703-170738
- BRANCH: example/09-reactor

## Round 1

- VERDICT: APPROVE

An independent skeptical review (fresh eyes on `git diff master...HEAD`, the
`modding` sources, and the 03/11 patterns) plus my own pass. The diff is clean:
the modding bus is genuinely the mechanic (shop = list of `HandlerSpec`s, buying
= `build_handler`, TAP/SELL/tick all route through `commands.fire` -> the queue),
the tick(Update)->enqueue->queue_system(PostUpdate) ordering is correct, the 9
tests pass and are honest (including the end-to-end `fuel_rod_handler_runs_...`
which drives a live `GameEventsPlugin`), no panics on user-reachable paths, and
the web/docs wiring mirrors 11_overload. No BLOCKER or MAJOR findings.

The MINOR/NIT findings below are addressed in this round (I am running the whole
cycle) rather than deferred, except the LoC note which is a judgment call.

- [x] R1.1 (MINOR) examples/09_reactor.rs:113-185,82-86 - heat tension is
  trivially opt-out: a pure Solar + Market strategy makes zero heat, and ambient
  heat capped at tier 7 is cancelled by one free Heat Sink, so meltdown is
  effectively opt-in and infinite risk-free scaling exists. This undercuts the
  task's core "without letting heat run away" and the 06 shape (runs should end).
  - Response: Addressed (balance pass). Two changes: (1) tiers are now uncapped
    (geometric `tier_for_score`), so ambient heat scales with everything you have
    earned and never plateaus -- there is no set-and-forget equilibrium; you must
    keep expanding cooling as you grow, and a lapse melts you down. (2) Solar
    Array nerfed (+2.5 -> +1.4 energy) so fuel rods (which add heat) are the real
    scaling engine and clean generation is only a slow bootstrap, not a risk-free
    infinite. Re-verified headless with the autopilot: it now scales via fuel rods
    and must cool continuously as tiers climb (tier 12 by ~50 s), with heat the
    central pressure. Honest caveat: this makes heat the permanent central concern
    rather than forcing meltdown -- a superhuman APM can still hold an equilibrium
    (the autopilot buys ~10 parts/s), but the trivial zero-heat infinite is gone
    and meltdown is readily reachable (verified separately by a fuel-only run).
- [x] R1.2 (MINOR) examples/09_reactor.rs:1176-1190 - "New best!" shows on ties
  because `record_high_score` bumps `high.0` before `spawn_game_over` reads it,
  so `final_score >= high.0` is true even when merely equalling a prior best.
  - Response: Fixed. Reordered the OnEnter chain so `spawn_game_over` runs before
    `record_high_score` and compares with strict `final_score > high.0` against
    the still-old best; the else branch then shows the correct previous best.
- [x] R1.3 (NIT) examples/09_reactor.rs:1258-1269 - `setup_registry_on` (tests)
  duplicates the 9 registrations in `setup_registry` (the system); they can
  silently desync.
  - Response: Fixed. Both now delegate to a single `register_all(&mut registry)`
    helper.
- [x] R1.4 (NIT) examples/09_reactor.rs:1037 - redundant
  `#[allow(clippy::too_many_arguments)]`; the crate `[lints.clippy]` already
  allows it crate-wide.
  - Response: Fixed. Removed the attribute; clippy stays clean.
- [x] R1.5 (NIT) modding bus, inherent - intra-tick handler order is
  `Query<&EventHandler>` iteration order, so energy/heat-gated parts can see
  slightly different intermediate state depending on install order. Worth a docs
  sentence.
  - Response: Fixed. Added a note to the module doc and the docs writeup that
    intra-tick handler order is unspecified (spawn/archetype order).
- [ ] R1.6 (MINOR) examples/09_reactor.rs (~1416 lines) - ~40% over the task's
  "~1000 LoC" guidance. Defensible (smaller than 06/07/08/10, heavy on the
  crate's doc-comment convention) but a stretch of the stated scope.
  - Response: Acknowledged, left as-is. The bulk is the shop/heat-bar UI spawning
    and the 9 tests; trimming doc comments would fight the crate convention. It
    is the smallest of the game examples.
