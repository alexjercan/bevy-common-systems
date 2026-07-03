# Fruit ninja: slice pop flash on fruit

- STATUS: OPEN
- PRIORITY: 88
- TAGS: feature,example

## Goal

When a fruit is sliced, make the cut read as impactful: briefly flash/scale the
object right before it bursts, instead of it vanishing instantly into fragments.

## Steps

- [ ] Decide the beat: the sliced shell currently gets `ExplodeMesh` and is
      despawned by `on_fragments_spawned` the same frame. To show a flash, delay
      the explode by ~0.05-0.1s: on slice, instead of inserting `ExplodeMesh`
      immediately, insert a `SlicePop { timer }` (and remove `Sliceable` /
      `Projectile`), swap the material to a bright/white flash and bump the
      scale up.
- [ ] Add a `resolve_slice_pop` system (Update, `Playing`) that ticks
      `SlicePop`; while active it can keep the pop scaled, and when it elapses
      inserts `ExplodeMesh` so the existing explosion path runs.
- [ ] Keep it cheap: a single extra material handle (white) in `FruitAssets`
      for the flash, or tint via a new StandardMaterial; reuse the existing
      explode/ fragment flow unchanged after the pop.
- [ ] Make sure a bomb still triggers game over on the slice frame (bombs should
      not wait for the pop, or the pop is fine but the health hit is immediate);
      note the chosen behavior.
- [ ] Verify: `cargo fmt --check`, `cargo clippy --all-targets` (+ `--features
      debug`), `./scripts/check-ascii.sh`, real boot (auto-slice; confirm the
      pop then explosion, no panic).

## Notes

- Slice site: `slice_objects` fruit/bomb branches (~:660-720); it currently does
  `.remove::<Sliceable>().remove::<Projectile>().insert(ExplodeMesh {...})`.
- `on_fragments_spawned` despawns the shell after fragments spawn; the pop just
  inserts a short delay before `ExplodeMesh`.
- Alternative simpler beat if the delayed-explode is fiddly: spawn a brief
  expanding translucent "slash" quad/gizmo at the slice point and keep the
  instant explode. Note which approach was taken.
- Depends on nothing, but coexists with the golden-fruit and variety tasks
  (shared spawn/slice code) -- expect light merge overlap.
- No new dependencies.
