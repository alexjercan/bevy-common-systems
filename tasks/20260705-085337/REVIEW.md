# Review: Bastion on-screen build + upgrade buttons

- TASK: 20260705-085337
- BRANCH: feature/bastion-build-buttons

## Round 1

- VERDICT: APPROVE

Reviewed `git diff master...HEAD` (~446 lines added to `examples/12_bastion.rs`).
The feature delivers the Goal and is well-tested.

Correctness (verified by reading + running):

- Region-owns-tap is sound: `build_bar_input` and `place_or_select` both read
  (never mutate) `DragState`, so no ordering between them is needed; they
  partition the tap by `build_bar_zone` (the bar owns in-zone taps via
  `build_bar_hit`, which returns `Some` for every in-zone point, so nothing falls
  through to the world), and `place_or_select` early-returns for in-zone taps.
  Space is exempt from the guard, so keyboard placement still works.
- Multiple systems access `Tower` (`build_bar_input` `&mut`, `update_build_bar`
  `&`, `place_or_select`/`aim_and_fire_towers`); Bevy serializes the conflicting
  ones -- no panic, no data race.
- The bar is always visible (plain `Visibility`, no `RevealOnTouch`), meeting the
  "standalone game too" requirement; mouse and touch both reach it via
  `UnifiedPointer`. Upgrade logic is shared by the U key and the button via
  `try_upgrade_selected` (no duplicated cost/credit math).
- `BUILD_BAR_H_FRAC` is the single source of truth for both the strip height and
  the hit-test zone (`build_bar_zone_matches_strip_height` asserts they agree).

Tests are meaningful, not tautological:

- `build_bar_hit_maps_columns_and_misses` checks column mapping + out-of-zone
  miss.
- `build_bar_tap_arms_tower_and_upgrades` is a real integration test: it drives
  the actual `build_bar_input` system through a minimal App with a sized Window
  and asserts the OBSERVABLE effects (a tower tap arms `build.spec = Some(0)`; an
  Upgrade tap raises a selected tower's level 1->2 and its damage). This directly
  answers the follow-up retro's "verify the advertised control's effect, not a
  proxy" -- and covers the pointer path the keyboard-only autopilot cannot.

Checks re-run in the worktree: plain `cargo build --example` clean (no dead-code,
per the packs-task lesson), `clippy --all-targets` clean, `fmt --check` clean,
`check-ascii` clean, `cargo test --examples` green (13 bastion unit tests). A
`BCS_SHOT=390x844` screenshot confirmed all four buttons render on one row with
readable keybind/name/cost, tinted per tower.

Findings:

- [x] R1.1 (NIT) examples/12_bastion.rs:1019,1099 - the Upgrade button's gold
  accent `Color::srgb(1.0, 0.85, 0.35)` is written literally in both
  `spawn_build_bar` and `update_build_bar`; if one is retuned they drift. Extract
  a `const UPGRADE_ACCENT: Color` (or a small helper) and use it in both.
  - Response: Done -- added `const UPGRADE_ACCENT: Color` and used it in both
    `spawn_build_bar` and `update_build_bar`.

Observation (not a finding): the always-visible bar occupies the bottom
`BUILD_BAR_H_FRAC` (14%) of the screen and owns taps there, so a tower cannot be
placed by tapping that band. This is a deliberate, documented tradeoff (orbit
repositions the ground), consistent with `11_overload`'s pad model; no change
needed.
