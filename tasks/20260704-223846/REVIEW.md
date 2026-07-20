# Review: migrate games onto the Wave 2 harvest modules

- TASK: 20260704-223846
- BRANCH: feat/migrate-wave2-harvests (cleanup/migrate-wave2-harvests)

## Round 1

- VERDICT: APPROVE

Reviewed `git diff master...HEAD` (examples only, net -264 lines, no crate
source touched -- correct for a migration onto already-shipped modules). Ran the
full suite in the worktree: `cargo fmt --check`, `cargo clippy --all-targets`
(clean), `--features debug` (clean), `cargo test` (94 unit + 51 doctests),
`cargo test --examples` (green), `check-ascii.sh`. Exercised gameplay:
08/11/12 ran the full `AutopilotPlugin` cycle (menu -> playing -> game-over) with
no panic; 06/07/09/10 booted to the render loop; 10's menu confirmed by
screenshot (TitlePulse title + FPS item + centered layout).

Verified the coverage matrix against the four task parts and spot-checked the
judgement calls. Findings are all informational.

- [x] R1.1 (NIT) Verified, not a finding: The one real runtime regression the
  SoundBank harvest introduced is `get()` panicking on a key absent from the load
  list (where the old struct was a compile error). Statically diffed each game's
  `get(Sfx::X)` keys against its `SoundBank::load` list: every used key is loaded
  in all five migrated games, so the panic path is unreachable.
- [x] R1.2 (NIT) 11_overload's best-score test changed from `>=` (tie counts as a
  new best, guarded by `> 0.0`) to `HighScore::record`'s strict `>`. For a
  continuous f32 survival time an exact tie is unreachable, so this is
  behaviour-equivalent in practice; the `> 0.0` guard is subsumed by the strict
  compare (a 0s run: `record(0.0)` -> not a new best). Documented in the retro.
- [x] R1.3 (NIT) 08/09/11's menu column gap unifies 14px -> 16px by adopting the
  crate `centered_screen()`. A 2px cosmetic change, the price of dedup; 06/07/10/12
  were already 16px.
- [x] R1.4 (NIT) `AnyStartPress` was applied only where the advance check already
  matched its click/tap/Space/Enter shape (06's tap-only, 09/11's "any key"
  menus were deliberately left inline -- converting the any-key menus would
  *narrow* accepted keys). `manual_controls` (09, in-game TAP/SELL) and the Space
  tower-placement paths (12) were correctly left untouched -- they are gameplay,
  not menu advance.
- [x] R1.5 (NIT) `glowing_material` was applied to lit emissive materials only;
  08's deliberately `unlit: true` blended streak material was left as-is
  (converting it would drop `unlit` and change its rendering). 12's Blend ghost
  and roughness/metallic variants keep their extra fields via the
  `..glowing_material(..)` spread.
