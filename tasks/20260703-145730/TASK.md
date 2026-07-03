# Fix bevy 0.19 build: Skybox.image is now Option in camera/skybox.rs

- STATUS: CLOSED
- PRIORITY: 90
- TAGS: bug,bevy-migration

## Goal

After the bevy 0.18 -> 0.19 bump, `Skybox::image` changed type from
`Handle<Image>` to `Option<Handle<Image>>`. The skybox setup in
`src/camera/skybox.rs` assigns a bare handle and no longer compiles. Wrap the
handle in `Some(...)` so the crate compiles and the skybox still attaches.

## Steps

- [x] In `src/camera/skybox.rs` (~line 132), change
      `image: config.cubemap.clone()` to
      `image: Some(config.cubemap.clone())` inside the `Skybox { .. }` insert.
- [x] Confirm no other `Skybox { image: .. }` construction sites exist in the
      crate (grep `Skybox {`); this is the only one today.
- [x] Verify the module compiles: `cargo build` (the E0308 error gone).
- [x] Full check suite: `cargo fmt --check`, `cargo clippy --all-targets`
      (+ `--features debug`), `cargo test`, `./scripts/check-ascii.sh`.
- [x] Boot the skybox path if an example exercises it (grep examples for
      `SkyboxPlugin`/`SkyboxConfig`); if none does, note that it is covered by
      compile-only.

## Notes

- Error: `E0308 mismatched types: expected Option<Handle<Image>>, found
  Handle<Image>` at `src/camera/skybox.rs:132`.
- `..default()` on `Skybox` already fills `brightness` etc.; only `image`
  needs the `Some` wrapper.
- No new dependencies; pure API-shape fix.

## Close-out

Wrapped `image` in `Some(..)`. Bevy 0.19 also broke the surrounding
`reinterpret_stacked_2d_as_array` call in the same function (folded in, since it
is the same file and migration): `images.get_mut(..)` now needs a `let mut`
binding, and the `image.height()/image.width()` args had to be hoisted into a
`let layers` local to avoid an immutable-borrow-while-mutably-borrowed conflict
(E0596/E0502). No other `Skybox { image: .. }` sites exist. No example boots the
skybox, so it is covered compile-only (doctest in `skybox.rs` passes). Full
suite green.
