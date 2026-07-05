# data-driven towers/enemies for 12_bastion + evaluate a SpecCatalog module (modding hook)

- STATUS: CLOSED
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

## Steps (plan)

- [x] 1. Serde-ify `TowerSpec`/`EnemySpec` (`String` name, `[f32;3]` color,
  `#[derive(Deserialize)]`) and add a `Catalog { towers, enemies }` resource.
- [x] 2. Extract the stat tables to `assets/bastion/catalog.json`; add
  `load_catalog()` that reads the file via `std::fs` on native (edit JSON,
  re-run, no recompile) and falls back to a compiled-in `include_str!` default
  on wasm or a missing file.
- [x] 3. Replace `tower_specs()`/`enemy_specs()` with catalog reads; generalize
  `select_build` (loop `Digit1..` over `catalog.towers`) and `spawn_enemy`
  (weighted pick over `catalog.enemies` via a new data field) so new JSON
  entries actually participate.
- [x] 4. Prove: add a 3rd tower and a 3rd enemy in `catalog.json` ONLY (no code)
  and verify they appear (boot / autopilot / screenshot).
- [x] 5. Spike write-up in `docs/`: evaluate promoting a `SpecCatalog<T>` crate
  module (sibling to `EventHandlerRegistry`); compare to 09's `HandlerSpec`
  registry, decide crate-module vs game-local, and seed a follow-up task only if
  a second concrete user justifies it.

## Close-out

Towers/enemies are data-driven from `assets/bastion/catalog.json` (native reads
the file with no recompile; wasm/missing-file falls back to an embedded copy).
`select_build` and `spawn_enemy` iterate the catalog, so a Sniper tower and a
Swarm enemy were added purely in JSON (now shipped as the living proof).
No-recompile proven by running one binary against 2/3/4-tower JSON with an
unchanged mtime.

Spike verdict (`docs/2026-07-05-bastion-data-catalog.md`): keep the catalog
game-local -- it shares nothing with `modding/registry` (data vs behavior), the
only reusable part is the ~25-line fs-or-embedded JSON loader, and there is no
second concrete user yet. No `SpecCatalog<T>` module and no speculative
follow-up seeded (two-user rule). Reviewed APPROVE in one round.
