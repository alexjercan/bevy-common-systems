# Data-driven towers/enemies for 12_bastion, and the `SpecCatalog<T>` question

- TASK: tasks/20260704-220719 (spike)
- SPIKE INPUT: docs/spikes/20260704-220530-new-prototype-game-ideas.md

## What shipped

`12_bastion`'s tower and enemy stats now live in
`assets/bastion/catalog.json`, deserialized once at startup into a `Catalog`
resource (`Vec<TowerSpec>` + `Vec<EnemySpec>`). Towers and enemies are
referenced by their index in those `Vec`s, and the two places that used to hard
-code the roster now iterate the catalog:

- `select_build` binds tower `i` to `Digit(i + 1)` (via `digit_key`), so a new
  tower is buildable with the next number key.
- `spawn_enemy` picks a type by weight (`weighted_enemy_index`) using each
  enemy's `spawn_weight` + `wave_weight`, so a new enemy joins the spawn mix by
  data alone (replacing the old hard-coded Runner-vs-Brute coin flip).

Loading is native-first: on native it reads the on-disk file (edit the JSON,
re-run, no recompile), falling back to a compiled-in `include_str!` copy if the
file is missing/unparseable or on wasm (which has no filesystem). A parse error
logs and falls back rather than panicking.

**Proof it is data-driven (no recompile):** the binary was built once, then run
twice against different JSON with an unchanged mtime -- the startup log went from
`2 towers` (embedded default) to `3 towers ["Gun","Cannon","Sniper"], 3 enemies
["Runner","Brute","Swarm"]` (on-disk) to `4 towers [... "Tesla"]` after another
edit, with no rebuild. A `Sniper` tower and a `Swarm` enemy were added purely in
JSON. Unit tests cover the embedded catalog parsing, the level-scaled upgrade
cost, and `weighted_enemy_index` (including that an appended enemy participates).

## Should this become a crate module (`SpecCatalog<T>`)?

**No -- keep it game-local.** The reasoning, against the two-user test the task
set:

1. **It reuses nothing from `modding/registry`, because it solves a different
   problem.** `EventHandlerRegistry` maps name-strings to *constructor closures*
   so JSON can name *behavior* (filters/actions) that gets built into trait
   objects at runtime. The TD catalog names *pure data* (stat structs) that
   serde deserializes directly -- there is no name->constructor indirection, no
   trait objects, no behavior. A `SpecCatalog<T>` would sit beside the registry
   in the file tree but share none of its machinery.

2. **The genuinely reusable nugget is the loader, not a "catalog" type.**
   Strip away the game-specific parts (the `TowerSpec`/`EnemySpec` fields, the
   weighted spawn pick, the digit-key bindings, the per-wave stat scaling) and
   what is left is ~25 lines: "deserialize `T` from an on-disk JSON file, with an
   embedded `include_str!` fallback for wasm / missing / unparseable, into a
   resource, and log what loaded." That is generic over `T: DeserializeOwned`
   and is the only part another game would want verbatim. A `SpecCatalog<T>`
   wrapper type around a plain `Vec<T>` would add almost nothing over that
   loader.

3. **There is no second concrete user yet.** The task named the reactor (09) as
   the comparison candidate, but 09's shop is a `HandlerSpec` list built through
   the *event registry* -- it is a user of `modding/registry`, not of a stat
   catalog. So the count of real "load a `Vec<Spec>` from JSON" users is one
   (this game). The spike's own rule is: do not build the abstraction before two
   concrete users exist.

**Recommendation.** Keep the catalog game-local. If a second game later wants the
same "edit-JSON, no-recompile" asset load, promote just the *loader* as a small
generic helper (working name `data::load_json_asset::<T>(path, embedded) -> T`,
or a thin `JsonAsset<T>` resource wrapper) -- NOT a `SpecCatalog<T>` type, which
would bake in a `Vec`-of-named-specs shape that only this game happens to have.
The turret/enemy selection logic (`weighted_enemy_index`, `digit_key`) is
game-shaped and should stay in the example regardless.

No follow-up task is seeded: promoting the loader is cheap and mechanical once a
second user appears, and speculative extraction now would violate the two-user
rule. This paragraph is the negative result the spike was asked to produce.
