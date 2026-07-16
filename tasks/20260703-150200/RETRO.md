# Retro: bevy 0.19 / avian 0.7 build migration

- TASKS: 20260703-145725 (FontSize, ui/status.rs), 20260703-145730 (Skybox.image
  Option + image borrow, camera/skybox.rs), 20260703-145707 (EventWorld Mutable
  bound + deref + or_else, modding/events.rs), 20260703-150132 (FontSize in
  examples/06_fruitninja.rs). All CLOSED, committed on feat/update-bevy-version.

## Context

The dep bump (bevy 0.18 -> 0.19, avian3d 0.6 -> 0.7, bevy_enhanced_input 0.25 ->
0.26, bevy-inspector-egui 0.36 -> 0.37, plus a new bevy_asset_loader 0.27
dev-dep) was already staged uncommitted in the worktree. It surfaced 8 lib +
4 example compile errors. Committed the bump first so each fix built on a stable
base, then fixed by module.

## What went well
- Grouped the errors by module up front (status/skybox/events/example) and made
  one tatr task per cluster; independent files meant no dependencies and the
  fixes never collided.
- The gnarly-looking `.chain()` E0599 (a full screen of unsatisfied Curve/
  CurveExt bounds) was a pure knock-on of the `ResMut<W>` failure. Fixing the
  root cause -- the `EventWorld: Resource<Mutability = Mutable>` bound -- made it
  vanish. Reading the first real error instead of the scariest one saved time.
- Verified behavior, not just compilation: booted `03_modding` and watched the
  filter/action/counter output to confirm the event bus still works after the
  deref and or_else changes.

## Difficulties / bugs
- The 0.19 mutability split is the theme: `Component`/`Resource` now carry an
  associated `Mutability` type and `ResMut<T>` requires `Mutability = Mutable`.
  Generic resource params (`ResMut<W>`) need the bound stated explicitly; deref
  coercion does not fire for a generic `W` at a `&W`/`&mut W` call site, so
  `&*world` / `&mut *world` were required.
- Two errors only appeared after earlier ones were fixed: the skybox
  `reinterpret_stacked_2d_as_array` borrow conflict (needed `let mut` and
  hoisting `height()/width()` into a `let layers` local) surfaced only once the
  `Some(..)` error cleared, and the whole 06_fruitninja example (4 more FontSize
  sites) only compiled after the lib did. Compile errors mask downstream ones;
  budget for more than one pass.
- tatr `new` uses a second-resolution timestamp ID, so three `new` calls fired
  in one Bash line all got the same ID and clobbered each other -- only the last
  survived. Create tasks in separate calls (or one per second) to get distinct
  IDs.

## Improve next time
- When doing a version bump, grep the whole tree (src AND examples) for the
  changed API before declaring the task list; the example FontSize task was
  discovered mid-flow only because examples compile after the lib. A single
  `grep 'font_size:'` up front would have found all 7 sites at once.
- Expect cascade errors on engine bumps: run `cargo build --all-targets` between
  each fix rather than assuming the first error list is complete.
