# data-driven towers/enemies for 12_bastion + evaluate a SpecCatalog module (modding hook)

- STATUS: OPEN
- PRIORITY: 1
- TAGS: spike,examples,modding

> Spike: docs/spikes/20260704-220530-new-prototype-game-ideas.md (read first).
> Follow-up to the tower-defense game; DEPENDS ON 12_bastion shipping first
> (tasks/20260704-220736).

## Goal

Make `12_bastion`'s towers and enemies **data-driven** so new tower/enemy types
(different HP, damage, range, fire-rate, turn-rate, speed, cost) can be added
without recompiling -- the user's `modding` stretch goal -- and decide whether
the mechanism should become a crate module.

Shape note (important): the crate's existing `modding` is an **event bus**
(`EventWorld` + `EventHandler` + `registry`, demoed by 03/09). A tower-defense
wants a **stat catalog** (TowerSpec / EnemySpec loaded from JSON, spawned by
name), which is closer to how `modding/registry` maps name-strings to
constructors than to the event bus. So this is NOT just "add the event bus to
the TD".

Path:

1. Start from the game-local serde catalog that 12_bastion ships with; move the
   tower/enemy stat tables to an external JSON file loaded at startup, spawned
   by name. Prove it by adding a new tower and a new enemy purely in JSON.
2. Then evaluate (this is the spike-y half): does this generalize into a crate
   module -- a `SpecCatalog<T>` sibling to `EventHandlerRegistry` -- and can it
   reuse anything from `modding/registry`? If yes, that likely wants its own
   plan/spike; if it stays game-shaped, say so and keep it local.

Do not over-build the abstraction before two concrete users exist; the reactor
(09) and this game are the candidate pair to compare.

This task is stepless on purpose (spike output); run /plan to break it into
steps before /work.
