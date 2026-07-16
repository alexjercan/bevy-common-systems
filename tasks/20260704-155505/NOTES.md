# feedback: full-screen screen-flash overlay module

- DATE: 2026-07-04
- TASK: tasks/20260704-155505
- SPIKE: tasks/20260704-134035/SPIKE.md (Wave 1)
- FOLLOWS: tasks/20260704-134600/NOTES.md (the premise correction that
  spawned this task)

## What changed

Added `src/feedback/screen_flash.rs`: `ScreenFlashPlugin` drives a full-screen UI
damage/hit overlay whose alpha spikes to a peak and decays back to transparent --
the red flash a game slams across the whole screen on a hit or a death. It is the
real duplication the material-`Flash` task's premise was pointing at: 06, 07 and
10 each hand-rolled this overlay. All three are now refactored onto the module,
deleting their local copies.

## The duplication, and the one primitive that covers it

The three copies split into two shapes:

- **06_fruitninja / 10_asteroids** -- spawn a short-lived `RedFlash { age,
  lifetime }` overlay on a death; a per-frame system fades its `BackgroundColor`
  alpha over the lifetime and despawns it. A one-shot spawn-and-fade.
- **07_orbit** -- a persistent `DamageFlashOverlay` node plus a `DamageFlash(f32)`
  intensity resource; a hazard hit spikes the intensity to 1 and a per-frame
  system decays it at a fixed rate, driving the overlay alpha. A reusable
  spike-and-decay.

These look different but are the *same* linear-decay primitive. A lifetime `L` is
just a decay rate of `1/L` per second, so both are "intensity starts at 1, loses
`decay` per second, alpha = `peak_alpha * intensity`". The only genuine
differences are (a) despawn-at-zero vs persist, and (b) fresh-spawn-per-event vs
re-spike-in-place. Both fall out of one component:

```rust
pub struct ScreenFlash {
    pub peak_alpha: f32,      // alpha at full intensity
    pub decay: f32,           // intensity lost per second; full -> 0 in 1/decay s
    pub despawn_on_end: bool, // despawn at intensity 0 (one-shot) or persist
}
```

- Spawn-and-fade (06/10): `commands.spawn(screen_flash(color, peak, decay))` --
  the builder sets `despawn_on_end: true`, so it fades and despawns itself.
- Spike-and-decay (07): spawn a persistent transparent overlay once
  (`screen_flash_node()` + a transparent `BackgroundColor` + a marker +
  `GlobalZIndex(-1)`, no `ScreenFlash`), then insert `ScreenFlash { despawn_on_end:
  false, .. }` on each hit to re-spike it. Re-insert re-triggers because the
  insert observer uses `On<Insert>` (resets intensity to 1), mirroring the
  material `Flash` re-flash semantics.

### Color lives in `BackgroundColor`, not in `ScreenFlash`

The plugin only writes the alpha channel (`background.0.with_alpha(alpha)`),
preserving the RGB. So the tint is whatever the overlay's `BackgroundColor` is
set to, and `ScreenFlash` carries no color field. This keeps the animation and
the color cleanly separated and lets 07 keep its slightly different tint
(`0.85/0.05/0.05`) vs 06/10 (`0.9/0.1/0.1`) with no API change.

## Why `feedback`, not `ui`

It is a UI overlay, but conceptually it is hit feedback, and it sits beside the
material `Flash` in the same module (the two are the "screen says you got hit" and
"the thing you hit flashed" halves of the same juice). Leaning `feedback` keeps
the game-feel primitives together.

## Tests

Pure `flash_alpha(intensity, peak)` (scale + clamp), plus three ECS tests driving
the plugin: insert spikes intensity to full and preserves the RGB tint;
spawn-and-fade decays and despawns itself; a persistent overlay survives a full
decay to transparent and re-spikes to peak on a re-insert. The two usage shapes
are both pinned.

## Notes / difficulties

- The persistent-07 case needed care around the initial frame: a one-shot spawns
  *at* peak (the spawn is the trigger), but 07 must start transparent and only
  flash on a hit. Solved by spawning 07's overlay without `ScreenFlash` at all
  and inserting it lazily on the first hit, rather than adding an
  `initial_intensity` field. This also means 07 needs no per-run reset -- the
  overlay is respawned transparent by `spawn_hud` on every `Playing` entry.
- No initial-flash regression to worry about: `screen_flash()` (06/10) is spawned
  exactly at the death moment, so starting at peak is correct there.
